# Models

Native Rust implementation of neural architectures using the **Candle** framework.

## 🎙️ Audio Models

- **T3 (Text-to-Token Transformer) (`t3_candle.rs`)**: A custom-trained transformer for high-fidelity speech synthesis. Features global positional alignment and paralinguistic tag support.
- **HiFT-GAN (`hiftgan.rs`)**: High-fidelity vocoder implementation for 24kHz audio reconstruction.
- **Unified Quantization (`quantized.rs`)**: Custom support for 8-bit quantized weights across diverse layer types.

## 🧠 Reasoning Models

- **Qwen-2.5 Reasoner (`reasoner.rs`)**: A deconstructed, pure-Rust implementation of the Qwen architecture. Designed for direct logit access and gradient calculations required by reinforcement learning (GRPO).

## 🛠️ Performance Features

- **MPS/Metal Support**: Optimized for Apple Silicon acceleration.
- **Harmonic Guarding**: Integrated `sanitize()` functions and activation clamping to prevent numerical explosion during long sequences.
