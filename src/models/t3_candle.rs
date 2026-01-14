use candle_core::{Device, Result, Tensor, D, IndexOp, DType};
use candle_nn::{Embedding, LayerNorm, Module};
use crate::models::t3::Config;
use crate::models::quantized::{UnifiedLinear, UnifiedWeight};
use std::collections::HashMap;
use tokenizers::Tokenizer;

fn get_tensor(weights: &HashMap<String, Tensor>, name: &str) -> Result<Tensor> {
    weights.get(name).cloned().ok_or_else(|| candle_core::Error::Msg(format!("Weight {} not found", name)))
}

fn load_linear(weights: &HashMap<String, Tensor>, prefix: &str, _in_dim: usize, _out_dim: usize, device: &Device) -> Result<UnifiedLinear> {
    let bias_name = format!("{}.bias", prefix);
    let bias = weights.get(&bias_name).cloned();

    let qshape_name = format!("{}.weight.qshape", prefix);
    let qdata_name = format!("{}.weight.qdata", prefix);
    let qscales_name = format!("{}.weight.qscales", prefix);
    let qzp_name = format!("{}.weight.qzp", prefix);

    if weights.contains_key(&qshape_name) {
        let qshape_t = get_tensor(weights, &qshape_name)?;
        let qshape = qshape_t.to_vec1::<u32>()?;
        let n = qshape[0] as usize;
        let k = qshape[1] as usize;
        let data = get_tensor(weights, &qdata_name)?;

        let linear = if weights.contains_key(&qscales_name) && weights.contains_key(&qzp_name) {
            let scales = get_tensor(weights, &qscales_name)?;
            let zp = get_tensor(weights, &qzp_name)?;
            UnifiedLinear::load_quantized(data, scales, zp, bias, n, k, device)?
        } else {
            UnifiedLinear::load_q8_simple(data, bias, n, k, device)?
        };
        Ok(linear)
    } else {
        let weight_name = format!("{}.weight", prefix);
        let w = get_tensor(weights, &weight_name)?;
        Ok(UnifiedLinear::new(UnifiedWeight::F32(w), bias))
    }
}

pub fn load_embedding(weights: &HashMap<String, Tensor>, prefix: &str, _vocab_size: usize, n_embd: usize, device: &Device) -> Result<Embedding> {
    let prefix_full = if prefix.is_empty() { "weight".to_string() } else { format!("{}.weight", prefix) };
    let qshape_name = format!("{}.qshape", prefix_full);
    
    let weight = if weights.contains_key(&qshape_name) {
        let n = vocab_size_fix(weights, &qshape_name)?;
        let linear = load_linear(weights, prefix, n, n_embd, device)?;
        linear.weight.dequantize(device)?
    } else {
        get_tensor(weights, &prefix_full)?.to_dtype(DType::F32)?
    };
    Ok(Embedding::new(weight, n_embd))
}

fn vocab_size_fix(weights: &HashMap<String, Tensor>, qshape_name: &str) -> Result<usize> {
    let qshape_t = get_tensor(weights, qshape_name)?;
    let qshape = qshape_t.to_vec1::<u32>()?;
    Ok(qshape[0] as usize)
}

fn sanitize(t: &Tensor) -> Result<Tensor> {
    // SOTA 6.5: Precise per-element sanitization. 
    // We replace only the corrupted elements with 0.0, preserving the rest of the signal.
    let mask_nan = t.ne(t)?.to_dtype(DType::U8)?; // NaN != NaN is true
    let mask_inf = t.abs()?.gt(1e30f32)?.to_dtype(DType::U8)?; 
    let mask_u8 = (mask_nan + mask_inf)?; // Combine masks (any value > 0 means corrupted)
    
    // Check if any element needs sanitization
    let corruption_count = mask_u8.sum_all()?.to_scalar::<u8>()?;
    if corruption_count > 0 {
        let zeros = Tensor::zeros_like(t)?;
        let mask = mask_u8.gt(0u8)?;
        // Candle where_cond: mask.where_cond(on_true, on_false)
        mask.where_cond(&zeros, t)
    } else {
        Ok(t.clone())
    }
}

#[derive(Debug)]
pub struct Attention {
    c_attn: UnifiedLinear,
    c_proj: UnifiedLinear,
    n_head: usize,
    head_dim: usize,
    kv_cache: Option<(Tensor, Tensor)>,
}

impl Attention {
    fn load(weights: &HashMap<String, Tensor>, prefix: &str, cfg: &Config, device: &Device) -> Result<Self> {
        let n_embd = cfg.n_embd;
        let n_head = cfg.n_head;
        let head_dim = n_embd / n_head;
        
        let c_attn = load_linear(weights, &format!("{}.attn.c_attn", prefix), n_embd, 3 * n_embd, device)?;
        let c_proj = load_linear(weights, &format!("{}.attn.c_proj", prefix), n_embd, n_embd, device)?;
        Ok(Self {
            c_attn,
            c_proj,
            n_head,
            head_dim,
            kv_cache: None,
        })
    }

    fn forward(&mut self, x: &Tensor, mask: Option<&Tensor>) -> Result<Tensor> {
        let (b_sz, seq_len, n_embd) = x.dims3()?;
        let x_dtype = x.dtype();
        let qkv = self.c_attn.forward(x)?;
        let qkv = qkv.to_dtype(DType::F32)?.reshape((b_sz, seq_len, 3, self.n_head, self.head_dim))?;
        let qkv = qkv.transpose(1, 3)?; 
        
        // SOTA: Clamp projections to prevent explosion before matmul
        let mut q = qkv.narrow(2, 0, 1)?.squeeze(2)?.contiguous()?; 
        let mut k = qkv.narrow(2, 1, 1)?.squeeze(2)?.contiguous()?; 
        let mut v = qkv.narrow(2, 2, 1)?.squeeze(2)?.contiguous()?; 
        
        // SOTA 6.2: Universal Articulation - wide activation guards for harmonics
        q = q.clamp(-150.0f32, 150.0f32)?;
        k = k.clamp(-150.0f32, 150.0f32)?;
        v = v.clamp(-150.0f32, 150.0f32)?;
        
        if let Some((prev_k, prev_v)) = &self.kv_cache {
            // SOTA: Secure concatenation with device-local contiguity
            k = Tensor::cat(&[prev_k, &k], 2)?.contiguous()?;
            v = Tensor::cat(&[prev_v, &v], 2)?.contiguous()?;
        }
        // SOTA 6.2: Lead Sanctuary 4.0 - articulate containment
        self.kv_cache = Some((k.clamp(-150.0f32, 150.0f32)?, v.clamp(-150.0f32, 150.0f32)?));

        // SOTA 6.2: Silicon Fortress - expressive matmul clamp
        let att_raw = q.matmul(&k.transpose(D::Minus2, D::Minus1)?.contiguous()?)?;
        let att = (att_raw.clamp(-500.0f32, 500.0f32)? / (self.head_dim as f64).sqrt())?;
        
        // SOTA 6.0: The Titanium Barrier - absolute protection for exp() before softmax
        let att_f32 = att.to_dtype(DType::F32)?.clamp(-60.0f32, 60.0f32)?;
        
        let mut att_f32 = if let Some(mask) = mask {
            att_f32.broadcast_add(&mask.to_dtype(DType::F32)?)?
        } else {
            att_f32
        };
        
        att_f32 = candle_nn::ops::softmax(&att_f32, D::Minus1)?;
        let att = att_f32.to_dtype(x_dtype)?;
        
        let y = att.matmul(&v)?; 
        let y = y.transpose(1, 2)?.reshape((b_sz, seq_len, n_embd))?;
        // SOTA 6.3: Harmonic Shield attention output cage
        let y = y.to_dtype(DType::F32)?.clamp(-200.0f32, 200.0f32)?;
        self.c_proj.forward(&y)
    }

    fn clear_cache(&mut self) {
        self.kv_cache = None;
    }
}

#[derive(Debug)]
pub struct MLP {
    c_fc: UnifiedLinear,
    c_proj: UnifiedLinear,
}

impl MLP {
    fn load(weights: &HashMap<String, Tensor>, prefix: &str, cfg: &Config, device: &Device) -> Result<Self> {
        let n_embd = cfg.n_embd;
        let n_inner = n_embd * 4;
        let c_fc = load_linear(weights, &format!("{}.mlp.c_fc", prefix), n_embd, n_inner, device)?;
        let c_proj = load_linear(weights, &format!("{}.mlp.c_proj", prefix), n_inner, n_embd, device)?;
        Ok(Self { c_fc, c_proj })
    }

    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let x = self.c_fc.forward(x)?;
        // SOTA 6.3: Harmonic GELU guard
        let x = x.clamp(-150.0f32, 150.0f32)?.gelu()?;
        let x = self.c_proj.forward(&x)?;
        // SOTA 6.3: Harmonic MLP output cage
        x.clamp(-200.0f32, 200.0f32)
    }
}

#[derive(Debug)]
pub struct Block {
    ln_1: LayerNorm,
    attn: Attention,
    ln_2: LayerNorm,
    mlp: MLP,
}

impl Block {
    fn load(weights: &HashMap<String, Tensor>, prefix: &str, cfg: &Config, device: &Device) -> Result<Self> {
        let weight_dtype = candle_core::DType::F32;
        
        let ln_1_w = get_tensor(weights, &format!("{}.ln_1.weight", prefix))?.to_dtype(weight_dtype)?;
        let ln_1_b = get_tensor(weights, &format!("{}.ln_1.bias", prefix))?.to_dtype(weight_dtype)?;
        let ln_1 = LayerNorm::new(ln_1_w, ln_1_b, cfg.layer_norm_epsilon);
        
        let attn = Attention::load(weights, prefix, cfg, device)?;
        
        let ln_2_w = get_tensor(weights, &format!("{}.ln_2.weight", prefix))?.to_dtype(weight_dtype)?;
        let ln_2_b = get_tensor(weights, &format!("{}.ln_2.bias", prefix))?.to_dtype(weight_dtype)?;
        let ln_2 = LayerNorm::new(ln_2_w, ln_2_b, cfg.layer_norm_epsilon);
        
        let mlp = MLP::load(weights, prefix, cfg, device)?;
        Ok(Self { ln_1, attn, ln_2, mlp })
    }

    fn forward(&mut self, x: &Tensor, mask: Option<&Tensor>) -> Result<Tensor> {
        let x = x.to_dtype(DType::F32)?;
        // SOTA 6.3: Harmonic entry guard
        let x = x.clamp(-300.0f32, 300.0f32)?;
        let residual = &x;
        let x = self.ln_1.forward(&x)?;
        let x = self.attn.forward(&x, mask)?;
        let x = (x + residual)?.clamp(-400.0f32, 400.0f32)?; // SOTA: Harmonic Shield Addition
        let residual = &x;
        let x = self.ln_2.forward(&x)?;
        let x = self.mlp.forward(&x)?;
        let x = (x + residual)?.clamp(-400.0f32, 400.0f32)?; // SOTA: Harmonic Shield Addition
        // SOTA 6.3: Final block-level harmonic cage and sanitization
        let x = x.clamp(-400.0f32, 400.0f32)?;
        sanitize(&x)
    }
}

#[derive(Debug)]
pub struct T3Candle {
    pub wte: Embedding,
    wpe: Embedding,
    h: Vec<Block>,
    ln_f: LayerNorm,
    speech_head: UnifiedLinear,
    t3_cond_emb: Tensor,
}

impl T3Candle {
    pub fn load_from_map(weights: &HashMap<String, Tensor>, cfg: &Config, device: &Device) -> Result<Self> {
        let weight_dtype = candle_core::DType::F32;
        
        let wte = load_embedding(weights, "text_emb", cfg.vocab_size, cfg.n_embd, device)?;
        let wpe = load_embedding(weights, "t3.tfmr.wpe", cfg.n_positions, cfg.n_embd, device)?;
        
        let mut h = Vec::new();
        for i in 0..cfg.n_layer {
            h.push(Block::load(weights, &format!("t3.tfmr.h.{}", i), cfg, device)?);
        }
        
        let ln_f_w = get_tensor(weights, "t3.tfmr.ln_f.weight")?.to_dtype(weight_dtype)?;
        let ln_f_b = get_tensor(weights, "t3.tfmr.ln_f.bias")?.to_dtype(weight_dtype)?;
        let ln_f = LayerNorm::new(ln_f_w, ln_f_b, cfg.layer_norm_epsilon);
        
        // SOTA 4.1: Ensure condition embedding is properly dequantized
        let t3_cond_emb = if weights.contains_key("t3_cond_emb.weight.qshape") {
            let n = vocab_size_fix(weights, "t3_cond_emb.weight.qshape")?;
            let linear = load_linear(weights, "t3_cond_emb", n, cfg.n_embd, device)?;
            let w = linear.weight.dequantize(device)?;
            // SOTA 6.2: Ensure rank-3 for consistent concatenation, allowing multi-token
            match w.rank() {
                2 => w.unsqueeze(0)?,
                3 => w,
                _ => w.reshape((1, w.elem_count() / cfg.n_embd, cfg.n_embd))?,
            }
        } else {
            let w = get_tensor(weights, "t3_cond_emb")?.to_dtype(weight_dtype)?;
            match w.rank() {
                2 => w.unsqueeze(0)?,
                3 => w,
                _ => w.reshape((1, w.elem_count() / cfg.n_embd, cfg.n_embd))?,
            }
        };
        
        let speech_head = load_linear(weights, "speech_head", cfg.n_embd, 6563, device)?;
        Ok(Self { wte, wpe, h, ln_f, speech_head, t3_cond_emb })
    }

    pub fn forward(&mut self, text_tokens: &Tensor, mask: Option<&Tensor>) -> Result<Tensor> {
        let x_dtype = DType::F32;
        let text_embeds = self.wte.forward(text_tokens)?.to_dtype(x_dtype)?;
        let (_b_sz, _seq_len, _) = text_embeds.dims3()?;
        
        // SOTA 7.1: Global Positional Alignment
        let _n_cond = self.t3_cond_emb.dim(1)?;
        
        // Combine [Condition + Text]
        let x_joined = Tensor::cat(&[self.t3_cond_emb.clone().to_dtype(x_dtype)?, text_embeds], 1)?;
        let total_seq_len = x_joined.dim(1)?;
        
        // Apply positional encoding to the ENTIRE sequence starting from 0
        let pos = Tensor::arange(0u32, total_seq_len as u32, x_joined.device())?;
        let mut x = (x_joined + self.wpe.forward(&pos)?.to_dtype(x_dtype)?)?;
        
        for block in self.h.iter_mut() {
            x = block.forward(&x, mask)?;
        }
        let x = self.ln_f.forward(&x)?;
        // SOTA 4.2: Guard speech head logits
        let x = x.to_dtype(DType::F32)?.clamp(-50.0f32, 50.0f32)?;
        self.speech_head.forward(&x)
    }

    pub fn forward_embeds(&mut self, embeds: &Tensor, pos: &Tensor, mask: Option<&Tensor>) -> Result<Tensor> {
        let x_dtype = DType::F32;
        let embeds = embeds.to_dtype(x_dtype)?.clamp(-302.0f32, 302.0f32)?;
        let wpe = self.wpe.forward(pos)?.to_dtype(x_dtype)?;
        // SOTA 6.3: Ensure rank-3 for broadcasting if needed
        let wpe = if wpe.rank() == 2 { wpe.unsqueeze(0)? } else { wpe };
        let mut x = (embeds + wpe)?;
        
        let mask = mask.map(|m| m.to_dtype(x_dtype)).transpose()?;
        
        for block in self.h.iter_mut() {
            x = block.forward(&x, mask.as_ref())?;
        }
        let x = self.ln_f.forward(&x)?;
        // SOTA 6.3: Harmonic head logits
        let x = x.to_dtype(DType::F32)?.clamp(-300.0f32, 300.0f32)?;
        self.speech_head.forward(&x)
    }

    pub fn clear_cache(&mut self) {
        for block in self.h.iter_mut() {
            block.attn.clear_cache();
        }
    }

    pub fn generate_tokens_internal_static(
        t3_model: &mut crate::models::t3_candle::T3Candle,
        tokenizer: &Tokenizer,
        text: &str,
        speech_emb: &Embedding,
        device: &Device,
        start_token: i64,
        stop_token: i64
    ) -> Result<Vec<i64>> {
        let weight_dtype = speech_emb.embeddings().dtype();
        let mut clean_text = text.trim().to_string();
        if !clean_text.ends_with('.') && !clean_text.ends_with('!') && !clean_text.ends_with('?') {
            clean_text.push('.');
        }

        let encoding = tokenizer.encode(clean_text, true).map_err(|e| candle_core::Error::Msg(e.to_string()))?;
        let text_ids: Vec<u32> = encoding.get_ids().iter().map(|&x| x as u32).collect();
        // SOTA 7.1: Correct global positional alignment
        // [Condition (unindexed?)] + [Text (indexed 0..N)] + [Speech (indexed N..M)]
        // Actually, Python suggests EVERYTHING is indexed. 
        // Let's align text starting from n_cond.
        let n_cond = t3_model.t3_cond_emb.dim(1)?;
        
        // 1. Condition Positional Coating
        let cond_pos = Tensor::arange(0u32, n_cond as u32, device)?;
        let cond_wpe = t3_model.wpe.forward(&cond_pos)?.to_dtype(weight_dtype)?;
        let cond_ready = (t3_model.t3_cond_emb.clone().to_dtype(weight_dtype)? + cond_wpe.unsqueeze(0)?)?;

        // 2. Text Positional Coating
        let text_ids_tensor = Tensor::from_vec(text_ids.clone(), (1, text_ids.len()), device)?;
        let text_pos = Tensor::arange(n_cond as u32, (n_cond + text_ids.len()) as u32, device)?;
        let text_embeds = t3_model.wte.forward(&text_ids_tensor)?.to_dtype(weight_dtype)?;
        let text_wpe = t3_model.wpe.forward(&text_pos)?.to_dtype(weight_dtype)?;
        let text_ready = (text_embeds + text_wpe.unsqueeze(0)?)?;

        // Concatenate condition with ready text
        let prefix_embeds = Tensor::cat(&[cond_ready, text_ready], 1)?;
        let prefix_len = prefix_embeds.dim(1)?;

        t3_model.clear_cache();
        let mut speech_ids = vec![start_token as u32];
        
        // SOTA: Optimized loop
        for i in 0..1024 {
            let input_embeds = if i == 0 {
                let speech_start_id = Tensor::from_vec(vec![start_token as u32], (1, 1), device)?;
                // Apply positional encoding to speech start token correctly (at index prefix_len)
                let speech_pos = Tensor::from_vec(vec![prefix_len as u32], (1, 1), device)?;
                let speech_start_embeds = speech_emb.forward(&speech_start_id)?;
                // wpe.forward on (1,1) already returns (1,1,1024)
                let speech_start_wpe = t3_model.wpe.forward(&speech_pos)?.to_dtype(weight_dtype)?;
                let speech_ready = (speech_start_embeds + speech_start_wpe)?;
                Tensor::cat(&[&prefix_embeds, &speech_ready], 1)?
            } else {
                let last_id = Tensor::from_vec(vec![*speech_ids.last().unwrap()], (1, 1), device)?;
                // Step i means we are at position prefix_len + i
                let cur_pos = (prefix_len + i) as u32;
                let cur_pos_tensor = Tensor::from_vec(vec![cur_pos], (1, 1), device)?;
                let emb = speech_emb.forward(&last_id)?;
                let wpe = t3_model.wpe.forward(&cur_pos_tensor)?.to_dtype(weight_dtype)?;
                (emb + wpe)?
            };

            let seq_len = input_embeds.dim(1)?;
            let kv_len = prefix_len + i + 1;
            
            let mask = if seq_len > 1 {
                let m: Vec<_> = (0..seq_len).flat_map(|ii| (0..kv_len).map(move |jj| {
                    if jj <= ii + kv_len - seq_len { 0.0f32 } else { -1e9f32 }
                })).collect();
                Some(Tensor::from_vec(m, (seq_len, kv_len), device)?.to_dtype(weight_dtype)?)
            } else {
                None
            };

            // SOTA 6.1: forward_block bypasses forward_embeds to avoid double WPE application
            let mut x = input_embeds.to_dtype(DType::F32)?;
            for block in t3_model.h.iter_mut() {
                x = block.forward(&x, mask.as_ref())?;
            }
            let x = t3_model.ln_f.forward(&x)?;
            let logits = t3_model.speech_head.forward(&x.clamp(-300.0f32, 300.0f32)?)?;
            
            let next_token_logits = logits.i((0, logits.dim(1)? - 1, ..))?
                .contiguous()?;
            
            // SOTA: Sanitize logits before CPU argmax to prevent NaN-driven index jumps
            let next_token_logits = sanitize(&next_token_logits)?
                .to_device(&Device::Cpu)?;
            
            // SOTA: Early NaN/Inf detection - if still present after sanitization, bias towards silence
            let sum = next_token_logits.sum_all()?.to_scalar::<f32>()?;
            let next_token_logits = if sum.is_nan() || sum.is_infinite() {
                tracing::warn!("Speaker SOTA: Model collapsed (NaN/Inf) at token index {}, defaulting to SILENCE (4299)", i);
                let mut silence_logits = vec![-100.0f32; 6563];
                silence_logits[4299] = 100.0f32; // Strongly bias towards silence
                Tensor::from_vec(silence_logits, (6563,), &Device::Cpu)?
            } else {
                next_token_logits
            };

            let next_token = next_token_logits.flatten_all()?.argmax(D::Minus1)?.to_scalar::<u32>()?;
            
            // SOTA: Robust Range check - the decoder's voice vocab ends at 6561
            if next_token >= 6561 && next_token < 6563 && next_token != stop_token as u32 {
                tracing::warn!("Speaker SOTA: Model predicted meta-token {} at index {}, continuing...", next_token, i);
            }
            
            if next_token >= 6563 {
                tracing::error!("Speaker SOTA: Model collapsed to invalid index {} at token index {}", next_token, i);
                break;
            }

            if next_token == stop_token as u32 { break; }
            speech_ids.push(next_token);

            // SOTA: Trace the first few tokens to identify immediate EOS or collapse
            if i < 5 {
                tracing::debug!("Speaker SOTA: Step {}, predicted token {}", i, next_token);
            }
            
            // Heuristic breakage for repetition loops which are not SOTA
            if speech_ids.len() > 10 && speech_ids[speech_ids.len()-5..] == speech_ids[speech_ids.len()-10..speech_ids.len()-5] {
                break;
            }
        }
        
        Ok(speech_ids.into_iter().map(|x| x as i64).collect())
    }
}
