use anyhow::Result;
use ort::session::{Session, builder::GraphOptimizationLevel};

fn main() -> Result<()> {
    println!("🚀 Testing simple ORT load (no external data)...");
    let session = Session::builder()?
        .with_optimization_level(GraphOptimizationLevel::Level3)?
        .commit_from_file("/Users/javoerokour/Desktop/BUDDHA/CODE/agency/rust_agency/artifacts/chatterbox/s3_flow_encoder.onnx")?;
    
    let first_input_name = session.inputs[0].name.to_string();
    println!("✅ Model loaded. Running dummy inference on input: {}...", first_input_name);
    
    let tokens: Vec<i64> = vec![1, 2, 3, 4, 5];
    let tokens_val = ort::value::Value::from_array((vec![1, 5], tokens))?;
    
    let outputs = session.run(ort::inputs![first_input_name.as_str() => tokens_val]?)?;
    println!("✅ Inference success! Output count: {}", outputs.len());
    
    Ok(())
}