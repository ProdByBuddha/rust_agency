use anyhow::Result;
use candle_core::{Tensor};
use candle_nn::{Conv1d, Conv1dConfig, Module};
use std::collections::HashMap;
use crate::models::quantized::{UnifiedLinear, UnifiedWeight};

// HiFT-GAN constants for 24kHz
const UPSAMPLE_INITIAL_CHANNEL: usize = 512;
const RESBLOCK_DILATIONS: &[&[usize]] = &[&[1, 3, 5], &[1, 3, 5], &[1, 3, 5]];

struct WeightFinder {
    weights: HashMap<String, Tensor>,
}

impl WeightFinder {
    fn new(weights: HashMap<String, Tensor>) -> Self {
        Self { weights }
    }

    fn find_and_remove(&mut self, shape: &[usize]) -> Result<(String, Tensor)> {
        let mut found_key = None;
        for (key, tensor) in &self.weights {
            if tensor.shape().dims() == shape {
                found_key = Some(key.clone());
                break;
            }
        }
        
        if let Some(key) = found_key {
            let tensor = self.weights.remove(&key).unwrap();
            Ok((key, tensor))
        } else {
            anyhow::bail!("Could not find tensor with shape {:?}", shape)
        }
    }
    
    fn find_upsample(&mut self, in_channels: usize) -> Result<(String, Tensor)> {
        let mut found_key = None;
        for (key, tensor) in &self.weights {
            let dims = tensor.shape().dims();
            if dims.len() == 3 && dims[0] == in_channels {
                found_key = Some(key.clone());
                break;
            }
        }
        if let Some(key) = found_key {
            let tensor = self.weights.remove(&key).unwrap();
            Ok((key, tensor))
        } else {
            anyhow::bail!("Could not find upsample for in {}", in_channels)
        }
    }

    fn find_post(&mut self, out_channels: usize) -> Result<(String, Tensor)> {
        let mut found_key = None;
        for (key, tensor) in &self.weights {
            let dims = tensor.shape().dims();
            if dims.len() == 3 && dims[0] == out_channels {
                found_key = Some(key.clone());
                break;
            }
        }
        if let Some(key) = found_key {
            let tensor = self.weights.remove(&key).unwrap();
            Ok((key, tensor))
        } else {
            anyhow::bail!("Could not find post for out {}", out_channels)
        }
    }

    fn remove(&mut self, key: &str) -> Result<Tensor> {
        self.weights.remove(key).ok_or_else(|| anyhow::anyhow!("Key {} not found", key))
    }
}

#[derive(Debug)]
struct ResBlock {
    convs1: Vec<Conv1d>,
    convs2: Vec<Conv1d>,
}

impl ResBlock {
    fn load(channels: usize, dilation: &[usize], finder: &mut WeightFinder) -> Result<Self> {
        let mut convs1 = Vec::new();
        let mut convs2 = Vec::new();
        for i in 0..dilation.len() {
            let d = dilation[i];
            let (_, w1) = finder.find_and_remove(&[channels, channels, 11])
                .or_else(|_| finder.find_and_remove(&[channels, channels, 7]))
                .or_else(|_| finder.find_and_remove(&[channels, channels, 3]))?;
            let k = w1.dim(2)?;
            let b1 = finder.find_and_remove(&[channels]).map(|(_, t)| t).unwrap_or(Tensor::zeros(channels, w1.dtype(), w1.device())?);
            let cfg1 = Conv1dConfig { padding: (k * d - d) / 2, stride: 1, dilation: d, groups: 1, ..Default::default() };
            convs1.push(Conv1d::new(w1, Some(b1), cfg1));
            
            let (_, w2) = finder.find_and_remove(&[channels, channels, k])?;
            let b2 = finder.find_and_remove(&[channels]).map(|(_, t)| t).unwrap_or(Tensor::zeros(channels, w2.dtype(), w2.device())?);
            let cfg2 = Conv1dConfig { padding: (k - 1) / 2, stride: 1, dilation: 1, groups: 1, ..Default::default() };
            convs2.push(Conv1d::new(w2, Some(b2), cfg2));
        }
        Ok(Self { convs1, convs2 })
    }

    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let mut x = x.clone();
        for (c1, c2) in self.convs1.iter().zip(self.convs2.iter()) {
            let residual = x.clone();
            let xt = c1.forward(&candle_nn::ops::leaky_relu(&x, 0.1)?)?;
            let xt = c2.forward(&candle_nn::ops::leaky_relu(&xt, 0.1)?)?;
            x = (xt + residual)?;
        }
        Ok(x)
    }
}

pub struct HiFTGAN {
    pub head_proj1: UnifiedLinear,
    pub head_proj2: UnifiedLinear,
    pub encoder_proj: UnifiedLinear,
    pub spk_embed_affine_layer: UnifiedLinear,
    conv_pre: Conv1d,
    ups: Vec<candle_nn::ConvTranspose1d>,
    resblocks: Vec<ResBlock>,
    conv_post: Conv1d,
}

impl HiFTGAN {
    pub fn load_from_map(weights: HashMap<String, Tensor>, device: &candle_core::Device) -> Result<Self> {
        let mut finder = WeightFinder::new(weights);
        
        let spk_embed_affine_layer = {
            let w = finder.remove("spk_embed_affine_layer.weight")?;
            let b = finder.remove("spk_embed_affine_layer.bias").ok();
            if let (Ok(s), Ok(z)) = (finder.remove("spk_embed_affine_layer.scales"), finder.remove("spk_embed_affine_layer.zp")) {
                UnifiedLinear::load_quantized(w, s, z, b, 80, 192, device)?
            } else {
                UnifiedLinear::new(UnifiedWeight::F32(w), b)
            }
        };

        let head_proj1 = {
            let w = finder.remove("head_proj1.weight")?;
            let s = finder.remove("head_proj1.scales")?;
            let z = finder.remove("head_proj1.zp")?;
            UnifiedLinear::load_quantized(w, s, z, None, 512, 1024, device)?
        };

        let head_proj2 = {
            let w = finder.remove("head_proj2.weight")?;
            let s = finder.remove("head_proj2.scales")?;
            let z = finder.remove("head_proj2.zp")?;
            let b = finder.remove("head_proj2.bias").ok();
            UnifiedLinear::load_quantized(w, s, z, b, 512, 512, device)?
        };
        
        let encoder_proj = {
            let w = finder.remove("encoder_proj.weight")?;
            let s = finder.remove("encoder_proj.scales")?;
            let z = finder.remove("encoder_proj.zp")?;
            let b = finder.remove("encoder_proj.bias").ok();
            UnifiedLinear::load_quantized(w, s, z, b, 80, 512, device)?
        };

        let (_, cp_w) = finder.find_and_remove(&[UPSAMPLE_INITIAL_CHANNEL, 80, 7])?;
        let cp_b = finder.find_and_remove(&[UPSAMPLE_INITIAL_CHANNEL])?.1;
        let conv_pre = Conv1d::new(cp_w, Some(cp_b), Conv1dConfig { padding: 3, ..Default::default() });

        let mut ups = Vec::new();
        let mut resblocks = Vec::new();
        let mut curr_channels = UPSAMPLE_INITIAL_CHANNEL;
        for _ in 0..4 {
            let (uname, uw) = match finder.find_upsample(curr_channels) {
                Ok(v) => v,
                Err(_) => break,
            };
            let next_channels = uw.dim(1)?;
            let k = uw.dim(2)?;
            let ub = finder.find_and_remove(&[next_channels]).map(|(_, t)| t).unwrap_or(Tensor::zeros(next_channels, uw.dtype(), uw.device())?);
            let u = if uname.contains("18852") { 8 } else if uname.contains("18999") { 8 } else { 2 };
            let cfg = candle_nn::ConvTranspose1dConfig { padding: (k - u) / 2, stride: u, dilation: 1, groups: 1, output_padding: 0 };
            ups.push(candle_nn::ConvTranspose1d::new(uw, Some(ub), cfg));
            
            for _ in 0..3 {
                if let Ok(rb) = ResBlock::load(next_channels, RESBLOCK_DILATIONS[0], &mut finder) {
                    resblocks.push(rb);
                } else { break; }
            }
            curr_channels = next_channels;
        }

        let (_, cpo_w) = finder.find_post(18)?;
        let cpo_b = finder.find_and_remove(&[18])?.1;
        let conv_post = Conv1d::new(cpo_w, Some(cpo_b), Conv1dConfig { padding: 3, ..Default::default() });

        Ok(Self { head_proj1, head_proj2, encoder_proj, spk_embed_affine_layer, conv_pre, ups, resblocks, conv_post })
    }

    pub fn forward(&self, x: &Tensor, spk_emb: &Tensor) -> Result<Tensor> {
        let x = self.head_proj1.forward(x)?;
        let x = self.head_proj2.forward(&x)?;
        let head_cond = self.encoder_proj.forward(&x)?;
        
        let spk_cond = self.spk_embed_affine_layer.forward(spk_emb)?;
        let mut x = head_cond.broadcast_add(&spk_cond.unsqueeze(1)?)?;
        x = x.transpose(1, 2)?;
        
        let mut x_out = self.conv_pre.forward(&x)?;
        for i in 0..self.ups.len() {
            x_out = self.ups[i].forward(&candle_nn::ops::leaky_relu(&x_out, 0.1)?)?;
            let mut xs: Option<Tensor> = None;
            for j in 0..3 {
                let rb_idx = i * 3 + j;
                if rb_idx < self.resblocks.len() {
                    let res = self.resblocks[rb_idx].forward(&x_out)?;
                    xs = match xs { Some(sum) => Some((sum + res)?), None => Some(res) };
                }
            }
            if let Some(sum) = xs { x_out = (sum / 3.0)?; }
        }
        x_out = self.conv_post.forward(&candle_nn::ops::leaky_relu(&x_out, 0.1)?)?;
        Ok(x_out.tanh()?)
    }
}