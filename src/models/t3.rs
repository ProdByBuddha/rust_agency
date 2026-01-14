use candle_core::{Result, Tensor, D};
use candle_nn::{embedding, Embedding, LayerNorm, Linear, Module, VarBuilder};

#[derive(Debug, Clone)]
pub struct Config {
    pub vocab_size: usize,
    pub n_positions: usize,
    pub n_embd: usize,
    pub n_layer: usize,
    pub n_head: usize,
    pub layer_norm_epsilon: f64,
}

impl Config {
    pub fn t3_turbo() -> Self {
        Self {
            vocab_size: 32000,
            n_positions: 2048,
            n_embd: 1024,
            n_layer: 24,
            n_head: 16,
            layer_norm_epsilon: 1e-5,
        }
    }
}

fn linear(in_dim: usize, out_dim: usize, vb: VarBuilder) -> Result<Linear> {
    let w = vb.get((out_dim, in_dim), "weight")?;
    let b = vb.get(out_dim, "bias")?;
    Ok(Linear::new(w, Some(b)))
}

#[derive(Debug)]
struct Attention {
    c_attn: Linear,
    c_proj: Linear,
    n_head: usize,
    head_dim: usize,
    kv_cache: Option<(Tensor, Tensor)>,
}

impl Attention {
    fn new(cfg: &Config, vb: VarBuilder) -> Result<Self> {
        let n_embd = cfg.n_embd;
        let n_head = cfg.n_head;
        let head_dim = n_embd / n_head;
        let c_attn = linear(n_embd, 3 * n_embd, vb.pp("attn.c_attn"))?;
        let c_proj = linear(n_embd, n_embd, vb.pp("attn.c_proj"))?;
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
        let qkv = self.c_attn.forward(x)?;
        let qkv = qkv
            .reshape((b_sz, seq_len, 3, self.n_head, self.head_dim))?
            .transpose(1, 3)?;
        let q = qkv.narrow(2, 0, 1)?.squeeze(2)?;
        let mut k = qkv.narrow(2, 1, 1)?.squeeze(2)?;
        let mut v = qkv.narrow(2, 2, 1)?.squeeze(2)?;

        if let Some((prev_k, prev_v)) = &self.kv_cache {
            k = Tensor::cat(&[prev_k, &k], 2)?;
            v = Tensor::cat(&[prev_v, &v], 2)?;
        }
        self.kv_cache = Some((k.clone(), v.clone()));

        let _k_len = k.dim(D::Minus2)?;
        let mut att = (q.matmul(&k.transpose(D::Minus2, D::Minus1)?)? / (self.head_dim as f64).sqrt())?;
        if let Some(mask) = mask {
            let mask = mask.broadcast_as(att.shape())?;
            let neg_inf = Tensor::new(f32::NEG_INFINITY, att.device())?.broadcast_as(att.shape())?;
            att = mask.where_cond(&att, &neg_inf)?;
        }
        att = candle_nn::ops::softmax(&att, D::Minus1)?;
        let y = att.matmul(&v)?;
        let y = y.transpose(1, 2)?.reshape((b_sz, seq_len, n_embd))?;
        self.c_proj.forward(&y)
    }

    fn clear_cache(&mut self) {
        self.kv_cache = None;
    }
}

#[derive(Debug)]
struct MLP {
    c_fc: Linear,
    c_proj: Linear,
}

impl MLP {
    fn new(cfg: &Config, vb: VarBuilder) -> Result<Self> {
        let n_embd = cfg.n_embd;
        let n_inner = n_embd * 4;
        let c_fc = linear(n_embd, n_inner, vb.pp("mlp.c_fc"))?;
        let c_proj = linear(n_inner, n_embd, vb.pp("mlp.c_proj"))?;
        Ok(Self { c_fc, c_proj })
    }

    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let x = self.c_fc.forward(x)?;
        let x = x.gelu()?;
        self.c_proj.forward(&x)
    }
}

#[derive(Debug)]
struct Block {
    ln_1: LayerNorm,
    attn: Attention,
    ln_2: LayerNorm,
    mlp: MLP,
}

impl Block {
    fn new(cfg: &Config, vb: VarBuilder) -> Result<Self> {
        let ln_1 = candle_nn::layer_norm(cfg.n_embd, cfg.layer_norm_epsilon, vb.pp("ln_1"))?;
        let attn = Attention::new(cfg, vb.clone())?;
        let ln_2 = candle_nn::layer_norm(cfg.n_embd, cfg.layer_norm_epsilon, vb.pp("ln_2"))?;
        let mlp = MLP::new(cfg, vb.clone())?;
        Ok(Self { ln_1, attn, ln_2, mlp })
    }

    fn forward(&mut self, x: &Tensor, mask: Option<&Tensor>) -> Result<Tensor> {
        let residual = x;
        let x = self.ln_1.forward(x)?;
        let x = self.attn.forward(&x, mask)?;
        let x = (x + residual)?;
        let residual = &x;
        let x = self.ln_2.forward(&x)?;
        let x = self.mlp.forward(&x)?;
        x + residual
    }

    fn clear_cache(&mut self) {
        self.attn.clear_cache();
    }
}

#[derive(Debug)]
pub struct T3Model {
    wte: Embedding,
    wpe: Embedding,
    h: Vec<Block>,
    ln_f: LayerNorm,
    cfg: Config,
}

impl T3Model {
    pub fn new(cfg: &Config, vb: VarBuilder) -> Result<Self> {
        let wte = embedding(cfg.vocab_size, cfg.n_embd, vb.pp("wte"))?;
        let wpe = embedding(cfg.n_positions, cfg.n_embd, vb.pp("wpe"))?;
        let mut h = Vec::new();
        let vb_h = vb.pp("h");
        for i in 0..cfg.n_layer {
            h.push(Block::new(cfg, vb_h.pp(i))?);
        }
        let ln_f = candle_nn::layer_norm(cfg.n_embd, cfg.layer_norm_epsilon, vb.pp("ln_f"))?;
        Ok(Self { wte, wpe, h, ln_f, cfg: cfg.clone() })
    }

    pub fn forward(&mut self, input_ids: &Tensor, position_ids: &Tensor) -> Result<Tensor> {
        let (_b_sz, seq_len) = input_ids.dims2()?;
        let input_embeds = self.wte.forward(input_ids)?;
        let position_embeds = self.wpe.forward(position_ids)?;
        let mut x = (input_embeds + position_embeds)?;
        
        let mask = if seq_len > 1 {
            let mask: Vec<_> = (0..seq_len)
                .flat_map(|i| (0..seq_len).map(move |j| u8::from(j <= i)))
                .collect();
            Some(Tensor::from_slice(&mask, (seq_len, seq_len), x.device())?)
        } else {
            None
        };

        for block in self.h.iter_mut() {
            x = block.forward(&x, mask.as_ref())?;
        }
        self.ln_f.forward(&x)
    }

    pub fn forward_embeds(&mut self, embeds: &Tensor, position_ids: &Tensor) -> Result<Tensor> {
        let (_b_sz, seq_len, _n_embd) = embeds.dims3()?;
        let position_embeds = self.wpe.forward(position_ids)?;
        let mut x = (embeds + position_embeds)?;
        
        // Causal mask for the input sequence
        let mask = if seq_len > 1 {
            let mask: Vec<_> = (0..seq_len)
                .flat_map(|i| (0..seq_len).map(move |j| u8::from(j <= i)))
                .collect();
            Some(Tensor::from_slice(&mask, (seq_len, seq_len), x.device())?)
        } else {
            None
        };

        for block in self.h.iter_mut() {
            x = block.forward(&x, mask.as_ref())?;
        }
        self.ln_f.forward(&x)
    }

    pub fn clear_cache(&mut self) {
        for block in self.h.iter_mut() {
            block.clear_cache();
        }
    }
}