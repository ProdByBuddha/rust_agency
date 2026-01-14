use anyhow::Result;
use std::fs::File;
use candle_core::quantized::gguf_file;

fn main() -> Result<()> {
    let path = "/Users/javoerokour/.cache/huggingface/hub/models--Qwen--Qwen2.5-Coder-0.5B-Instruct-GGUF/snapshots/ebb2015119c907b064c512bf053e945850b5875f/qwen2.5-coder-0.5b-instruct-q4_k_m.gguf";
    println!("Inspecting: {}", path);
    
    let mut file = File::open(path)?;
    let content = gguf_file::Content::read(&mut file)?;
    
    println!("Metadata Keys:");
    let mut keys: Vec<_> = content.metadata.keys().collect();
    keys.sort();
    for key in keys {
        let value = content.metadata.get(key).unwrap();
        println!(" - {}: {:?}", key, value);
    }
    
    Ok(())
}
