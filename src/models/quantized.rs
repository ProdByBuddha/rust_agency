use candle_core::{Tensor, Result, Device, Module, DType, bail, Shape};
use candle_core::quantized::{QTensor, QStorage, QMatMul, GgmlDType};

#[derive(Debug, Clone)]
pub enum UnifiedWeight {
    F32(Tensor),
    Q8 {
        weight: Tensor,
        scales: Tensor,
        zp: Tensor,
        n: usize,
        k: usize,
        dequantized: Option<Tensor>,
    },
    Q8_0(QMatMul),
    // Simple 8-bit or packed format (like GGML Q8_0)
    Q8Simple {
        data: Tensor,
        n: usize,
        k: usize,
        dequantized: Option<Tensor>,
    },
}

impl UnifiedWeight {
    pub fn dequantize(&self, device: &Device) -> Result<Tensor> {
        match self {
            UnifiedWeight::F32(t) => t.to_device(device),
            UnifiedWeight::Q8 { dequantized, .. } => {
                dequantized.as_ref().cloned().ok_or_else(|| candle_core::Error::Msg("Q8 dequant not cached".to_string()))
            },
            UnifiedWeight::Q8Simple { dequantized, .. } => {
                dequantized.as_ref().cloned().ok_or_else(|| candle_core::Error::Msg("Q8Simple dequant not cached".to_string()))
            },
            UnifiedWeight::Q8_0(_) => bail!("Cannot manually dequantize Q8_0 MatMul object"),
        }
    }
}

#[derive(Debug)]
pub struct UnifiedLinear {
    pub weight: UnifiedWeight,
    pub bias: Option<Tensor>,
}

impl UnifiedLinear {
    pub fn new(weight: UnifiedWeight, bias: Option<Tensor>) -> Self {
        Self { weight, bias }
    }

    pub fn load_quantized(weight: Tensor, scales: Tensor, zp: Tensor, bias: Option<Tensor>, n: usize, k: usize, device: &Device) -> Result<Self> {
        let w_f32 = weight.to_dtype(DType::F32)?;
        let zp_f32 = zp.to_dtype(DType::F32)?;
        let scales_f32 = scales.to_dtype(DType::F32)?;
        
        let block_count = scales.dim(1)?;
        let block_size = (n * k) / (n * block_count);
        
        let scales_exp = scales_f32.unsqueeze(2)?.expand((n, block_count, block_size))?.reshape((n, k))?;
        let zp_exp = zp_f32.unsqueeze(2)?.expand((n, block_count, block_size))?.reshape((n, k))?;
        
        let w_f32_flat = w_f32.reshape((n, k))?;
        let w_deq = w_f32_flat.broadcast_sub(&zp_exp)?.broadcast_mul(&scales_exp)?;
        let uw = UnifiedWeight::Q8 { 
            weight, scales, zp, n, k, 
            dequantized: Some(w_deq.to_device(device)?) 
        };
        
        Ok(Self::new(uw, bias))
    }

    pub fn load_q8_simple(data: Tensor, bias: Option<Tensor>, n: usize, k: usize, device: &Device) -> Result<Self> {
        // Check if it matches GGML Q8_0 size (34 bytes per 32 elements)
        let expected_size = (n * k / 32) * 34;
        let uw = if data.elem_count() == expected_size {
            let data_vec = data.flatten_all()?.to_vec1::<u8>()?;
            let storage = QStorage::from_data(std::borrow::Cow::Owned(data_vec), &Device::Cpu, GgmlDType::Q8_0)?;
            let qtensor = QTensor::new(storage, Shape::from((n, k)))?;
            let w_deq = qtensor.dequantize(&Device::Cpu)?;
            UnifiedWeight::Q8Simple { 
                data, n, k, 
                dequantized: Some(w_deq.to_device(device)?) 
            }
        } else {
            // Fallback for other simple formats
            let w_f32 = data.to_dtype(DType::F32)?;
            let w_f32_flat = w_f32.reshape((n, k))?;
            let w_deq = (w_f32_flat.broadcast_sub(&Tensor::new(128.0f32, &Device::Cpu)?.broadcast_as((n, k))?)? / 128.0)?;
            UnifiedWeight::Q8Simple { 
                data, n, k, 
                dequantized: Some(w_deq.to_device(device)?) 
            }
        };
        
        Ok(Self::new(uw, bias))
    }

    pub fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let x_dtype = x.dtype();
        let out = match &self.weight {
            UnifiedWeight::F32(w) => x.matmul(&w.t()?.to_dtype(x_dtype)?)?,
            UnifiedWeight::Q8_0(q) => q.forward(x)?,
            UnifiedWeight::Q8 { dequantized, .. } => {
                if let Some(w_deq) = dequantized {
                    let w_t = w_deq.t()?;
                    let w_dtype = w_deq.dtype();
                    let x_cast = if x_dtype != w_dtype { x.to_dtype(w_dtype)? } else { x.clone() };
                    
                    match x_cast.rank() {
                        3 => {
                            let (b, s, _) = x_cast.dims3()?;
                            let res = x_cast.reshape((b * s, w_deq.dim(1)?))?.matmul(&w_t)?;
                            res.reshape((b, s, w_deq.dim(0)?))?
                        },
                        _ => x_cast.matmul(&w_t)?
                    }.to_dtype(DType::F32)?
                } else {
                    bail!("Uncached dequantization not supported for performance");
                }
            },
            UnifiedWeight::Q8Simple { dequantized, .. } => {
                if let Some(w_deq) = dequantized {
                    let w_t = w_deq.t()?;
                    let w_dtype = w_deq.dtype();
                    let x_cast = if x_dtype != w_dtype { x.to_dtype(w_dtype)? } else { x.clone() };
                    
                    match x_cast.rank() {
                        3 => {
                            let (b, s, _) = x_cast.dims3()?;
                            let res = x_cast.reshape((b * s, w_deq.dim(1)?))?.matmul(&w_t)?;
                            res.reshape((b, s, w_deq.dim(0)?))?
                        },
                        _ => x_cast.matmul(&w_t)?
                    }.to_dtype(DType::F32)?
                } else {
                    bail!("Uncached simple dequantization not supported for performance");
                }
            }
        };
        match &self.bias {
            Some(b) => out.broadcast_add(&b.to_dtype(out.dtype())?),
            None => Ok(out),
        }
    }
}