use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use schemars::JsonSchema;
use std::path::PathBuf;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::io::Cursor;

use crate::tools::{Tool, ToolOutput};
use screenshots::Screen;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::Camera;
use image::{ImageFormat, DynamicImage};
use candle_core::{Device, Tensor, DType};
use candle_transformers::models::quantized_moondream;
use candle_transformers::generation::LogitsProcessor;
use tokenizers::Tokenizer;

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct VisionParams {
    /// The action to perform: 'capture_screen', 'capture_camera', or 'describe'.
    pub action: String,
    /// For 'describe', the path to the image or 'last_capture'.
    pub image_source: Option<String>,
    /// For 'describe', the question about the image.
    pub prompt: Option<String>,
    /// Optional display index for screen capture.
    pub display_id: Option<usize>,
}

pub struct VisionTool {
    last_image: Arc<Mutex<Option<PathBuf>>>,
}

impl Default for VisionTool {
    fn default() -> Self {
        Self {
            last_image: Arc::new(Mutex::new(None)),
        }
    }
}

impl VisionTool {
    pub fn new() -> Self {
        Self::default()
    }

    async fn capture_screen(&self, display_id: Option<usize>) -> Result<PathBuf> {
        let screens = Screen::all()?;
        let screen = if let Some(id) = display_id {
            screens.get(id).context("Display not found")? 
        } else {
            screens.first().context("No screens found")?
        };

        let image = screen.capture()?;
        let mut buffer = Vec::new();
        
        // Convert screenshots::Image to RgbaImage buffer then to DynamicImage
        let rgba_image = image::ImageBuffer::from_raw(image.width(), image.height(), image.into_raw())
            .context("Failed to create image buffer")?;
        let dynamic_image = DynamicImage::ImageRgba8(rgba_image);
        
        dynamic_image.write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)?;
        
        let path = PathBuf::from("artifacts/last_screen.png");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, buffer)?;
        
        let mut last = self.last_image.lock().await;
        *last = Some(path.clone());
        
        Ok(path)
    }

    async fn capture_camera(&self) -> Result<PathBuf> {
        let path = {
            // Simple camera capture using nokhwa
            let index = CameraIndex::Index(0);
            let format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
            let mut camera = Camera::new(index, format)?;
            camera.open_stream()?;
            let frame = camera.frame()?;
            let decoded = frame.decode_image::<RgbFormat>()?;
            
            let path = PathBuf::from("artifacts/last_camera.png");
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            decoded.save(&path)?;
            path
        };
        
        let mut last = self.last_image.lock().await;
        *last = Some(path.clone());
        
        Ok(path)
    }

    async fn describe_image(&self, image_path: PathBuf, prompt: String) -> Result<String> {
        // This is a heavy operation, we'll use Moondream2 via Candle
        let device = if candle_core::utils::metal_is_available() {
            Device::new_metal(0).unwrap_or(Device::Cpu)
        } else {
            Device::Cpu
        };

        // Load model configuration
        let model_file = tokio::task::spawn_blocking(move || {
            use hf_hub::{api::sync::ApiBuilder, Repo};
            let api = ApiBuilder::new().build()?;
            let repo = api.repo(Repo::new("santiagomed/candle-moondream".to_string(), hf_hub::RepoType::Model));
            repo.get("model-q4_0.gguf")
        }).await??;

        let tokenizer_file = tokio::task::spawn_blocking(move || {
            use hf_hub::{api::sync::ApiBuilder, Repo};
            let api = ApiBuilder::new().build()?;
            let repo = api.repo(Repo::new("vikhyatk/moondream2".to_string(), hf_hub::RepoType::Model));
            repo.get("tokenizer.json")
        }).await??;

        let tokenizer = Tokenizer::from_file(tokenizer_file).map_err(anyhow::Error::msg)?;
        let config = candle_transformers::models::moondream::Config::v2();
        
        let vb = candle_transformers::quantized_var_builder::VarBuilder::from_gguf(&model_file, &device)?;
        let mut model = quantized_moondream::Model::new(&config, vb)?;

        // Process image
        let img = image::open(image_path)?;
        let img = img.resize_to_fill(378, 378, image::imageops::FilterType::Triangle).to_rgb8();
        let data = img.into_raw();
        let data = Tensor::from_vec(data, (378, 378, 3), &Device::Cpu)?.permute((2, 0, 1))?;
        let mean = Tensor::new(&[0.5f32, 0.5, 0.5], &Device::Cpu)?.reshape((3, 1, 1))?;
        let std = Tensor::new(&[0.5f32, 0.5, 0.5], &Device::Cpu)?.reshape((3, 1, 1))?;
        let image_tensor = (data.to_dtype(DType::F32)? / 255.)?.
            broadcast_sub(&mean)?.
            broadcast_div(&std)?.
            to_device(&device)?.
            unsqueeze(0)?;

        let image_embeds = image_tensor.apply(model.vision_encoder())?;

        // Generate response
        let full_prompt = format!("\n\nQuestion: {}\n\nAnswer:", prompt);
        let tokens = tokenizer.encode(full_prompt, true).map_err(anyhow::Error::msg)?;
        let mut token_ids = tokens.get_ids().to_vec();
        
        let special_token = *tokenizer.get_vocab(true).get("<|endoftext|>").context("No special token")?;
        let mut logits_processor = LogitsProcessor::new(42, Some(0.7), Some(0.9));
        
        let mut result = String::new();
        for index in 0..512 {
            let context_size = if index > 0 { 1 } else { token_ids.len() };
            let ctxt = &token_ids[token_ids.len().saturating_sub(context_size)..];
            let input = Tensor::new(ctxt, &device)?.unsqueeze(0)?;
            
            let logits = if index > 0 {
                model.text_model.forward(&input)?
            } else {
                let bos_token = Tensor::new(&[special_token], &device)?.unsqueeze(0)?;
                model.text_model.forward_with_img(&bos_token, &input, &image_embeds)?
            };
            
            let logits = logits.squeeze(0)?.to_dtype(DType::F32)?;
            let next_token = logits_processor.sample(&logits)?;
            token_ids.push(next_token);
            
            if next_token == special_token || token_ids.ends_with(&[27, 10619, 29]) {
                break;
            }
            
            let token = tokenizer.decode(&[next_token], true).map_err(anyhow::Error::msg)?;
            result.push_str(&token);
        }

        Ok(result)
    }
}

#[async_trait]
impl Tool for VisionTool {
    fn name(&self) -> String { "vision".to_string() } 
    
    fn description(&self) -> String {
        "Give the agency eyes. Capture the screen, access the camera, and describe what's being seen using Moondream VLM.".to_string()
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["capture_screen", "capture_camera", "describe"],
                    "description": "The vision action to perform."
                },
                "image_source": {
                    "type": "string",
                    "description": "For 'describe', path to image or 'last_capture' (default)."
                },
                "prompt": {
                    "type": "string",
                    "description": "For 'describe', what to look for or describe."
                },
                "display_id": {
                    "type": "integer",
                    "description": "Optional display index for screen capture."
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, params: Value) -> Result<ToolOutput> {
        let p: VisionParams = serde_json::from_value(params)?;
        
        match p.action.as_str() {
            "capture_screen" => {
                let path = self.capture_screen(p.display_id).await?;
                Ok(ToolOutput::success(
                    json!({"path": path.to_string_lossy()}),
                    format!("Captured screen to {:?}", path)
                ))
            },
            "capture_camera" => {
                let path = self.capture_camera().await?;
                Ok(ToolOutput::success(
                    json!({"path": path.to_string_lossy()}),
                    format!("Captured camera image to {:?}", path)
                ))
            },
            "describe" => {
                let source = p.image_source.unwrap_or_else(|| "last_capture".to_string());
                let path = if source == "last_capture" {
                    let last = self.last_image.lock().await;
                    last.clone().context("No image captured yet. Capture screen or camera first.")?
                } else {
                    PathBuf::from(source)
                };
                
                let prompt = p.prompt.unwrap_or_else(|| "Describe this image in detail.".to_string());
                let description = self.describe_image(path, prompt).await?;
                
                Ok(ToolOutput::success(
                    json!({"description": description}),
                    format!("Vision Analysis: {}", description)
                ))
            },
            _ => Ok(ToolOutput::failure("Unknown vision action")),
        }
    }
}
