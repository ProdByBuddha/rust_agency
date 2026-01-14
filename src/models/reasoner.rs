//! Reasoner Model (Qwen-2.5 Architecture)
//!
//! Deconstructed implementation of the Qwen-2.5 architecture, optimized for 
//! Reinforcement Learning (RL) and Group Relative Policy Optimization (GRPO).
//! This implementation provides direct access to logits and gradients.

use candle_core::{DType, Device, Result, Tensor, D};
use candle_nn::{embedding, Embedding, LayerNorm, Linear, Module, VarBuilder};

#[derive(Debug, Clone)]
pub struct Config {
    pub vocab_size: usize,
    pub hidden_size: usize,
    pub intermediate_size: usize,
    pub num_hidden_layers: usize,
    pub num_attention_heads: usize,
    pub num_key_value_heads: usize,
    pub layer_norm_std: f64,
    pub max_position_embeddings: usize,
    pub rope_theta: f32,
}

impl Config {
    pub fn qwen_7b() -> Self {
        Self {
            vocab_size: 151936,
            hidden_size: 3584,
            intermediate_size: 18944,
            num_hidden_layers: 28,
            num_attention_heads: 28,
            num_key_value_heads: 4, // GQA
            layer_norm_std: 1e-6,
            max_position_embeddings: 32768,
            rope_theta: 1000000.0,
        }
    }

    pub fn qwen_1_5b() -> Self {
        Self {
            vocab_size: 151936,
            hidden_size: 1536,
            intermediate_size: 8960,
            num_hidden_layers: 28,
            num_attention_heads: 12,
            num_key_value_heads: 2, // GQA
            layer_norm_std: 1e-6,
            max_position_embeddings: 32768,
            rope_theta: 1000000.0,
        }
    }

    pub fn qwen_0_5b() -> Self {
        Self {
            vocab_size: 151936,
            hidden_size: 896,
            intermediate_size: 4864,
            num_hidden_layers: 24,
            num_attention_heads: 14,
            num_key_value_heads: 2, // GQA
            layer_norm_std: 1e-6,
            max_position_embeddings: 32768,
            rope_theta: 1000000.0,
        }
    }
}

fn linear(in_dim: usize, out_dim: usize, vb: VarBuilder) -> Result<Linear> {
    let w = vb.get((out_dim, in_dim), "weight")?;
    // Qwen usually doesn't use bias in linears except for some specific layers, 
    // but we'll check for it to be safe.
    let b = if vb.contains_tensor("bias") {
        Some(vb.get(out_dim, "bias")?)
    } else {
        None
    };
    Ok(Linear::new(w, b))
}

#[derive(Debug)]
struct RotaryEmbedding {
    sin: Tensor,
    cos: Tensor,
}

impl RotaryEmbedding {
    fn new(cfg: &Config, device: &Device) -> Result<Self> {
        let dim = cfg.hidden_size / cfg.num_attention_heads;
        let inv_freq: Vec<_> = (0..dim)
            .step_by(2)
            .map(|i| 1f32 / cfg.rope_theta.powf(i as f32 / dim as f32))
            .collect();
        let inv_freq = Tensor::new(inv_freq, device)?;
        let t = Tensor::arange(0u32, cfg.max_position_embeddings as u32, device)?.to_dtype(DType::F32)?;
        let freqs = t.unsqueeze(1)?.matmul(&inv_freq.unsqueeze(0)?)?;
        let freqs = Tensor::cat(&[&freqs, &freqs], D::Minus1)?;
        Ok(Self {
            sin: freqs.sin()?,
            cos: freqs.cos()?,
        })
    }

    fn apply(&self, x: &Tensor, pos: usize) -> Result<Tensor> {
        let (_b_sz, _h, seq_len, _d) = x.dims4()?;
        let cos = self.cos.narrow(0, pos, seq_len)?.unsqueeze(0)?.unsqueeze(0)?;
        let sin = self.sin.narrow(0, pos, seq_len)?.unsqueeze(0)?.unsqueeze(0)?;
        
        // Rotate
        let x1 = x.narrow(D::Minus1, 0, x.dim(D::Minus1)? / 2)?;
        let x2 = x.narrow(D::Minus1, x.dim(D::Minus1)? / 2, x.dim(D::Minus1)? / 2)?;
        let x_rotated = Tensor::cat(&[&x2.neg()?, &x1], D::Minus1)?;
        
        Ok((x.broadcast_mul(&cos)? + x_rotated.broadcast_mul(&sin)?)?)
    }
}

#[derive(Debug)]
struct Attention {
    q_proj: Linear,
    k_proj: Linear,
    v_proj: Linear,
    o_proj: Linear,
    num_heads: usize,
    num_kv_heads: usize,
    head_dim: usize,
    rope: RotaryEmbedding,
    kv_cache: Option<(Tensor, Tensor)>,
}

impl Attention {
    fn new(cfg: &Config, vb: VarBuilder) -> Result<Self> {
        let head_dim = cfg.hidden_size / cfg.num_attention_heads;
        let q_proj = linear(cfg.hidden_size, cfg.num_attention_heads * head_dim, vb.pp("q_proj"))?;
        let k_proj = linear(cfg.hidden_size, cfg.num_key_value_heads * head_dim, vb.pp("k_proj"))?;
        let v_proj = linear(cfg.hidden_size, cfg.num_key_value_heads * head_dim, vb.pp("v_proj"))?;
        let o_proj = linear(cfg.num_attention_heads * head_dim, cfg.hidden_size, vb.pp("o_proj"))?;
        let rope = RotaryEmbedding::new(cfg, vb.device())?;
        Ok(Self {
            q_proj,
            k_proj,
            v_proj,
            o_proj,
            num_heads: cfg.num_attention_heads,
            num_kv_heads: cfg.num_key_value_heads,
            head_dim,
            rope,
            kv_cache: None,
        })
    }

    fn forward(&mut self, x: &Tensor, pos: usize, mask: Option<&Tensor>) -> Result<Tensor> {
        let (b_sz, seq_len, _hidden) = x.dims3()?;
        let q = self.q_proj.forward(x)?;
        let k = self.k_proj.forward(x)?;
        let v = self.v_proj.forward(x)?;

        let q = q.reshape((b_sz, seq_len, self.num_heads, self.head_dim))?.transpose(1, 2)?;
        let k = k.reshape((b_sz, seq_len, self.num_kv_heads, self.head_dim))?.transpose(1, 2)?;
        let v = v.reshape((b_sz, seq_len, self.num_kv_heads, self.head_dim))?.transpose(1, 2)?;

        let q = self.rope.apply(&q, pos)?;
        let k = self.rope.apply(&k, pos)?;

        let (mut k, mut v) = if let Some((prev_k, prev_v)) = &self.kv_cache {
            (Tensor::cat(&[prev_k, &k], 2)?, Tensor::cat(&[prev_v, &v], 2)?)
        } else {
            (k, v)
        };
        self.kv_cache = Some((k.clone(), v.clone()));

        // GQA: Repeat K/V heads if needed
        if self.num_heads != self.num_kv_heads {
            let ratio = self.num_heads / self.num_kv_heads;
            // k is (b, h_kv, s, d). We want to repeat dim 1 (heads).
            // candle doesn't have repeat_interleave, so we simulate it:
            // 1. Unsqueeze at dim 2 to get (b, h_kv, 1, s, d)
            // 2. Repeat at dim 2 by ratio: (b, h_kv, ratio, s, d)
            // 3. Flatten dim 1 and 2 to merge them: (b, h_kv * ratio, s, d)
            
            k = k.unsqueeze(2)?.repeat((1, 1, ratio, 1, 1))?.flatten(1, 2)?;
            v = v.unsqueeze(2)?.repeat((1, 1, ratio, 1, 1))?.flatten(1, 2)?;
        }

        let mut att = (q.matmul(&k.transpose(D::Minus2, D::Minus1)?)? / (self.head_dim as f64).sqrt())?;
        if let Some(mask) = mask {
            att = att.broadcast_add(mask)?;
        }
        att = candle_nn::ops::softmax(&att, D::Minus1)?;
        let y = att.matmul(&v)?;
        let y = y.transpose(1, 2)?.reshape((b_sz, seq_len, self.num_heads * self.head_dim))?;
        self.o_proj.forward(&y)
    }

    fn clear_cache(&mut self) {
        self.kv_cache = None;
    }
}

#[derive(Debug)]
struct MLP {
    gate_proj: Linear,
    up_proj: Linear,
    down_proj: Linear,
}

impl MLP {
    fn new(cfg: &Config, vb: VarBuilder) -> Result<Self> {
        let gate_proj = linear(cfg.hidden_size, cfg.intermediate_size, vb.pp("gate_proj"))?;
        let up_proj = linear(cfg.hidden_size, cfg.intermediate_size, vb.pp("up_proj"))?;
        let down_proj = linear(cfg.intermediate_size, cfg.hidden_size, vb.pp("down_proj"))?;
        Ok(Self { gate_proj, up_proj, down_proj })
    }

    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        // SwiGLU
        let gate = self.gate_proj.forward(x)?.silu()?;
        let up = self.up_proj.forward(x)?;
        let activated = (gate * up)?;
        self.down_proj.forward(&activated)
    }
}

#[derive(Debug)]
struct Block {
    input_layernorm: LayerNorm,
    self_attn: Attention,
    post_attention_layernorm: LayerNorm,
    mlp: MLP,
}

impl Block {
    fn new(cfg: &Config, vb: VarBuilder) -> Result<Self> {
        let input_layernorm = candle_nn::layer_norm(cfg.hidden_size, cfg.layer_norm_std, vb.pp("input_layernorm"))?;
        let self_attn = Attention::new(cfg, vb.pp("self_attn"))?;
        let post_attention_layernorm = candle_nn::layer_norm(cfg.hidden_size, cfg.layer_norm_std, vb.pp("post_attention_layernorm"))?;
        let mlp = MLP::new(cfg, vb.pp("mlp"))?;
        Ok(Self { input_layernorm, self_attn, post_attention_layernorm, mlp })
    }

    fn forward(&mut self, x: &Tensor, pos: usize, mask: Option<&Tensor>) -> Result<Tensor> {
        let residual = x;
        let x = self.input_layernorm.forward(x)?;
        let x = self.self_attn.forward(&x, pos, mask)?;
        let x = (x + residual)?;
        
        let residual = &x;
        let x = self.post_attention_layernorm.forward(&x)?;
        let x = self.mlp.forward(&x)?;
        x + residual
    }

    fn clear_cache(&mut self) {
        self.self_attn.clear_cache();
    }
}

#[derive(Debug)]
pub struct ReasonerModel {
    embed_tokens: Embedding,
    layers: Vec<Block>,
    norm: LayerNorm,
    lm_head: Linear,
    cfg: Config,
}

impl ReasonerModel {
    pub fn new(cfg: &Config, vb: VarBuilder) -> Result<Self> {
        let embed_tokens = embedding(cfg.vocab_size, cfg.hidden_size, vb.pp("model.embed_tokens"))?;
        let mut layers = Vec::new();
        let vb_l = vb.pp("model.layers");
        for i in 0..cfg.num_hidden_layers {
            layers.push(Block::new(cfg, vb_l.pp(i))?);
        }
        let norm = candle_nn::layer_norm(cfg.hidden_size, cfg.layer_norm_std, vb.pp("model.norm"))?;
        let lm_head = linear(cfg.hidden_size, cfg.vocab_size, vb.pp("lm_head"))?;
        Ok(Self { embed_tokens, layers, norm, lm_head, cfg: cfg.clone() })
    }

    pub fn forward(&mut self, input_ids: &Tensor, pos: usize) -> Result<Tensor> {
        let (_b_sz, seq_len) = input_ids.dims2()?;
        let mut x = self.embed_tokens.forward(input_ids)?;
        
        let mask = if seq_len > 1 {
            let mask: Vec<_> = (0..seq_len)
                .flat_map(|i| (0..seq_len).map(move |j| if j <= i { 0f32 } else { f32::NEG_INFINITY }))
                .collect();
            Some(Tensor::from_slice(&mask, (seq_len, seq_len), x.device())?.unsqueeze(0)?.unsqueeze(0)?)
        } else {
            None
        };

        for layer in self.layers.iter_mut() {
            x = layer.forward(&x, pos, mask.as_ref())?;
        }
        
        let x = self.norm.forward(&x)?;
        // Only return logits for the last token to save memory during inference, 
        // but we'll need the full sequence for training.
        let last_x = x.narrow(1, seq_len - 1, 1)?;
        self.lm_head.forward(&last_x)
    }

    pub fn forward_full(&mut self, input_ids: &Tensor, pos: usize) -> Result<Tensor> {
        let (_b_sz, seq_len) = input_ids.dims2()?;
        let mut x = self.embed_tokens.forward(input_ids)?;
        
        let mask = if seq_len > 1 {
            let mask: Vec<_> = (0..seq_len)
                .flat_map(|i| (0..seq_len).map(move |j| if j <= i { 0f32 } else { f32::NEG_INFINITY }))
                .collect();
            Some(Tensor::from_slice(&mask, (seq_len, seq_len), x.device())?.unsqueeze(0)?.unsqueeze(0)?)
        } else {
            None
        };

        for layer in self.layers.iter_mut() {
            x = layer.forward(&x, pos, mask.as_ref())?;
        }
        
        let x = self.norm.forward(&x)?;
        self.lm_head.forward(&x)
    }

    pub fn clear_cache(&mut self) {
        for layer in self.layers.iter_mut() {
            layer.clear_cache();
        }
    }
}