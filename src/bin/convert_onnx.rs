use anyhow::Result;
use candle_core::{Device, Tensor};
use std::collections::HashMap;

fn main() -> Result<()> {
    let path = "/Users/javoerokour/Desktop/BUDDHA/CODE/agency/rust_agency/artifacts/chatterbox/conditional_decoder_q8_full.onnx";
    let model = candle_onnx::read_file(path)?;
    let graph = model.graph.as_ref().unwrap();
    let device = Device::Cpu;

    let mut inputs = HashMap::new();
    inputs.insert("speech_tokens".to_string(), Tensor::zeros((1, 20), candle_core::DType::I64, &device)?);
    inputs.insert("speaker_embeddings".to_string(), Tensor::zeros((1, 192), candle_core::DType::F32, &device)?);
    inputs.insert("speaker_features".to_string(), Tensor::zeros((1, 1, 80), candle_core::DType::F32, &device)?);

    // Track shapes specifically for the failing node /Expand_8
    let mut values = HashMap::new();
    for t in &graph.initializer {
        values.insert(t.name.clone(), candle_onnx::eval::get_tensor(t, &t.name, &device)?);
    }
    for (k, v) in inputs { values.insert(k, v); }

    let mut computed = values.keys().cloned().collect::<std::collections::HashSet<_>>();
    let mut nodes: Vec<_> = graph.node.iter().collect();

    while !nodes.is_empty() {
        let mut progress = false;
        let mut remaining = Vec::new();
        for node in nodes {
            if node.input.iter().all(|i| i.is_empty() || computed.contains(i)) {
                if node.name == "/Expand_8" {
                    let in0 = values.get(&node.input[0]).unwrap();
                    let in1 = values.get(&node.input[1]).unwrap();
                    println!("Expand_8 info:");
                    println!("  Input 0 ({}) shape: {:?}", node.input[0], in0.shape());
                    println!("  Input 1 ({}) shape: {:?}", node.input[1], in1.shape());
                    println!("  Input 1 values: {:?}", in1.to_vec1::<i64>()?);
                }
                
                // We don't need to actually execute all nodes, just enough to reach Expand_8
                // But we need to keep track of outputs
                let out_names = &node.output;
                // Dummy tensor for outputs we don't care about, but need for dependencies
                // This is risky if shape depends on values, but let's try to just get to Expand_8
                
                // Use actual execute_node for nodes leading to Expand_8
                if let Err(_) = candle_onnx::eval::execute_node(node, &mut values, &device) {
                    // ignore errors for now, we just want to see the shapes if possible
                }
                
                for o in out_names { computed.insert(o.clone()); }
                progress = true;
                
                if node.name == "/Expand_8" { return Ok(()); }
            } else {
                remaining.push(node);
            }
        }
        if !progress { break; }
        nodes = remaining;
    }
    
    Ok(())
}