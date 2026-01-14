use anyhow::Result;
use ort::session::{Session, builder::GraphOptimizationLevel};

fn main() -> Result<()> {
    let session = Session::builder()?
        .with_optimization_level(GraphOptimizationLevel::Level3)?
        .commit_from_file("/Users/javoerokour/Desktop/BUDDHA/CODE/agency/rust_agency/artifacts/chatterbox/speech_encoder.onnx")?;
    
    println!("Inputs:");
    for input in &session.inputs {
        println!("  - Name: {}", input.name);
    }
    
    println!("Outputs:");
    for output in &session.outputs {
        println!("  - Name: {}", output.name);
    }
    
    Ok(())
}