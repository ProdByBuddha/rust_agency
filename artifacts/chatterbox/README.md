---
license: mit
language:
- en
base_model:
- ResembleAI/chatterbox-turbo
pipeline_tag: text-to-speech
tags:
- text-to-speech
- speech
- speech-generation
- voice-cloning
---
![resemble+cbturbo-1600x900](https://storage.googleapis.com/chatterbox-demo-samples/turbo/turbo-banner.jpg)


# Chatterbox TTS

<div style="display: flex; align-items: center; gap: 12px">
  <a href="https://resemble-ai.github.io/chatterbox_turbo_demopage/">
    <img src="https://img.shields.io/badge/listen-demo_samples-blue" alt="Listen to Demo Samples" />
  </a>
  <a href="https://huggingface.co/spaces/ResembleAI/chatterbox-turbo-demo">
    <img src="https://huggingface.co/datasets/huggingface/badges/resolve/main/open-in-hf-spaces-sm.svg" alt="Open in HF Spaces" />
  </a>
  <a href="https://podonos.com/resembleai/chatterbox">
    <img src="https://static-public.podonos.com/badges/insight-on-pdns-sm-dark.svg" alt="Insight on Podos" />
  </a>
</div>


<div style="display: flex; align-items: center; gap: 8px;">
  <span style="font-style: italic;white-space: pre-wrap">Made with  ❤️  by</span>
  <img width="100" alt="resemble-logo-horizontal" src="https://github.com/user-attachments/assets/35cf756b-3506-4943-9c72-c05ddfa4e525" />
</div>

**Chatterbox** is a family of three state-of-the-art, open-source text-to-speech models by Resemble AI.

We are excited to introduce **Chatterbox-Turbo**, our most efficient model yet. Built on a streamlined 350M parameter architecture, **Turbo** delivers high-quality speech with less compute and VRAM than our previous models. We have also distilled the speech-token-to-mel decoder, previously a bottleneck, reducing generation from 10 steps to just **one**, while retaining high-fidelity audio output.

**Paralinguistic tags** are now native to the Turbo model, allowing you to use `[cough]`, `[laugh]`, `[chuckle]`, and more to add distinct realism. While Turbo was built primarily for low-latency voice agents, it excels at narration and creative workflows.

If you like the model but need to scale or tune it for higher accuracy, check out our competitively priced TTS service (<a href="https://resemble.ai">link</a>). It delivers reliable performance with ultra-low latency of sub 200ms—ideal for production use in agents, applications, or interactive media.

<img width="1200" height="600" alt="Podonos Turbo Eval" src="https://storage.googleapis.com/chatterbox-demo-samples/turbo/podonos_turbo.png" />

### ⚡ Model Zoo

Choose the right model for your application.

| Model                                                                                                           | Size | Languages | Key Features                                            | Best For                                     | 🤗                                                                  | Examples |
|:----------------------------------------------------------------------------------------------------------------| :--- | :--- |:--------------------------------------------------------|:---------------------------------------------|:--------------------------------------------------------------------------| :--- |
| **Chatterbox-Turbo**                                                                                            | **350M** | **English** | Paralinguistic Tags (`[laugh]`), Lower Compute and VRAM | Zero-shot voice agents,  Production          | [Demo](https://huggingface.co/spaces/ResembleAI/chatterbox-turbo-demo)        | [Listen](https://resemble-ai.github.io/chatterbox_turbo_demopage/) |
| Chatterbox-Multilingual [(Language list)](#supported-languages)                                                 | 500M | 23+ | Zero-shot cloning, Multiple Languages                   | Global applications, Localization            | [Demo](https://huggingface.co/spaces/ResembleAI/Chatterbox-Multilingual-TTS) | [Listen](https://resemble-ai.github.io/chatterbox_demopage/) |
| Chatterbox [(Tips and Tricks)](#original-chatterbox-tips)                                                       | 500M | English | CFG & Exaggeration tuning                               | General zero-shot TTS with creative controls | [Demo](https://huggingface.co/spaces/ResembleAI/Chatterbox)              | [Listen](https://resemble-ai.github.io/chatterbox_demopage/) |

## Usage

##### Chatterbox-Turbo

```python
import onnxruntime
from transformers import AutoTokenizer
from huggingface_hub import hf_hub_download
import numpy as np
from tqdm import trange
import librosa
import soundfile as sf

MODEL_ID = "ResembleAI/chatterbox-turbo-ONNX"
SAMPLE_RATE = 24000
START_SPEECH_TOKEN = 6561
STOP_SPEECH_TOKEN = 6562
SILENCE_TOKEN = 4299
NUM_KV_HEADS = 16
HEAD_DIM = 64

class RepetitionPenaltyLogitsProcessor:
    def __init__(self, penalty: float):
        if not isinstance(penalty, float) or not (penalty > 0):
            raise ValueError(f"`penalty` must be a strictly positive float, but is {penalty}")
        self.penalty = penalty

    def __call__(self, input_ids: np.ndarray, scores: np.ndarray) -> np.ndarray:
        score = np.take_along_axis(scores, input_ids, axis=1)
        score = np.where(score < 0, score * self.penalty, score / self.penalty)
        scores_processed = scores.copy()
        np.put_along_axis(scores_processed, input_ids, score, axis=1)
        return scores_processed

def download_model(name: str, dtype: str = "fp32") -> str:
    filename = f"{name}{'' if dtype == 'fp32' else '_quantized' if dtype == 'q8' else f'_{dtype}'}.onnx"
    graph = hf_hub_download(MODEL_ID, subfolder="onnx", filename=filename)      # Download graph
    hf_hub_download(MODEL_ID, subfolder="onnx", filename=f"{filename}_data")    # Download weights
    return graph

# Download models
## dtype options: fp32, fp16, q8, q4, q4f16
conditional_decoder_path = download_model("conditional_decoder", dtype="fp32")
speech_encoder_path = download_model("speech_encoder", dtype="fp32")
embed_tokens_path = download_model("embed_tokens", dtype="fp32")
language_model_path = download_model("language_model", dtype="fp32")

# Create ONNX sessions
speech_encoder_session = onnxruntime.InferenceSession(speech_encoder_path)
embed_tokens_session = onnxruntime.InferenceSession(embed_tokens_path)
language_model_session = onnxruntime.InferenceSession(language_model_path)
cond_decoder_session = onnxruntime.InferenceSession(conditional_decoder_path)

# Generation parameters
text = "Oh, that's hilarious! [chuckle] Um anyway, how are you doing today?"
target_voice_path = "path/to/voice.wav"
output_file_name = "output.wav"
max_new_tokens = 1024
repetition_penalty = 1.2
apply_watermark = False

# Prepare audio input
audio_values, _ = librosa.load(target_voice_path, sr=SAMPLE_RATE)
audio_values = audio_values[np.newaxis, :].astype(np.float32)

# Prepare text input
tokenizer = AutoTokenizer.from_pretrained(MODEL_ID)
input_ids = tokenizer(text, return_tensors="np")["input_ids"].astype(np.int64)

# Generation loop
repetition_penalty_processor = RepetitionPenaltyLogitsProcessor(penalty=repetition_penalty)
generate_tokens = np.array([[START_SPEECH_TOKEN]], dtype=np.int64)
for i in trange(max_new_tokens, desc="Sampling", dynamic_ncols=True):
    inputs_embeds = embed_tokens_session.run(None, {"input_ids": input_ids})[0]

    if i == 0:
        ort_speech_encoder_input = {"audio_values": audio_values}
        cond_emb, prompt_token, speaker_embeddings, speaker_features = speech_encoder_session.run(None, ort_speech_encoder_input)
        inputs_embeds = np.concatenate((cond_emb, inputs_embeds), axis=1)

        # Initialize cache and LLM inputs
        batch_size, seq_len, _ = inputs_embeds.shape
        past_key_values = {
            i.name: np.zeros([batch_size, NUM_KV_HEADS, 0, HEAD_DIM], dtype=np.float16 if i.type == 'tensor(float16)' else np.float32)
            for i in language_model_session.get_inputs()
            if "past_key_values" in i.name
        }
        attention_mask = np.ones((batch_size, seq_len), dtype=np.int64)
        position_ids = np.arange(seq_len, dtype=np.int64).reshape(1, -1).repeat(batch_size, axis=0)

    logits, *present_key_values = language_model_session.run(None, dict(
        inputs_embeds=inputs_embeds,
        attention_mask=attention_mask,
        position_ids=position_ids,
        **past_key_values,
    ))

    logits = logits[:, -1, :]
    next_token_logits = repetition_penalty_processor(generate_tokens, logits)

    input_ids = np.argmax(next_token_logits, axis=-1, keepdims=True).astype(np.int64)
    generate_tokens = np.concatenate((generate_tokens, input_ids), axis=-1)
    if (input_ids.flatten() == STOP_SPEECH_TOKEN).all():
        break

    # Update values for next generation loop
    attention_mask = np.concatenate([attention_mask, np.ones((batch_size, 1), dtype=np.int64)], axis=1)
    position_ids = position_ids[:, -1:] + 1
    for j, key in enumerate(past_key_values):
        past_key_values[key] = present_key_values[j]

# Decode audio
speech_tokens = generate_tokens[:, 1:-1]
silence_tokens = np.full((speech_tokens.shape[0], 3), SILENCE_TOKEN, dtype=np.int64) # Add silence at the end
speech_tokens = np.concatenate([prompt_token, speech_tokens, silence_tokens], axis=1)

wav = cond_decoder_session.run(None, dict(
    speech_tokens=speech_tokens,
    speaker_embeddings=speaker_embeddings,
    speaker_features=speaker_features,
))[0].squeeze(axis=0)

# Optional: Apply watermark
if apply_watermark:
    import perth
    watermarker = perth.PerthImplicitWatermarker()
    wav = watermarker.apply_watermark(wav, sample_rate=SAMPLE_RATE)

sf.write(output_file_name, wav, SAMPLE_RATE)
```


## Built-in PerTh Watermarking for Responsible AI

Every audio file generated by Chatterbox includes [Resemble AI's Perth (Perceptual Threshold) Watermarker](https://github.com/resemble-ai/perth) - imperceptible neural watermarks that survive MP3 compression, audio editing, and common manipulations while maintaining nearly 100% detection accuracy.


## Watermark extraction

You can look for the watermark using the following script.

```python
import perth
import librosa

AUDIO_PATH = "YOUR_FILE.wav"

# Load the watermarked audio
watermarked_audio, sr = librosa.load(AUDIO_PATH, sr=None)

# Initialize watermarker (same as used for embedding)
watermarker = perth.PerthImplicitWatermarker()

# Extract watermark
watermark = watermarker.get_watermark(watermarked_audio, sample_rate=sr)
print(f"Extracted watermark: {watermark}")
# Output: 0.0 (no watermark) or 1.0 (watermarked)
```


## Official Discord

👋 Join us on [Discord](https://discord.gg/rJq9cRJBJ6) and let's build something awesome together!

## Acknowledgements
- [Cosyvoice](https://github.com/FunAudioLLM/CosyVoice)
- [Real-Time-Voice-Cloning](https://github.com/CorentinJ/Real-Time-Voice-Cloning)
- [HiFT-GAN](https://github.com/yl4579/HiFTNet)
- [Llama 3](https://github.com/meta-llama/llama3)
- [S3Tokenizer](https://github.com/xingchensong/S3Tokenizer)

## Citation
If you find this model useful, please consider citing.
```
@misc{chatterboxtts2025,
  author       = {{Resemble AI}},
  title        = {{Chatterbox-TTS}},
  year         = {2025},
  howpublished = {\url{https://github.com/resemble-ai/chatterbox}},
  note         = {GitHub repository}
}
```
## Disclaimer
Don't use this model to do bad things. Prompts are sourced from freely available data on the internet.