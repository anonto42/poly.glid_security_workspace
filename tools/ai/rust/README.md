# PolyGlid AI Engine (Rust Edition)

High-performance Rust-based AI assistant system integrated with the PolyGlid workspace.

## Features
- **Code Analysis:** Quality reviews and structural scoring.
- **Dependency Advisor:** Outdated library recommendations.
- **Build Optimizer:** Recommends compilation speed improvements.
- **Local & Remote Inference:** Supports OpenAI API and local quantized GGUF models.
- **Caching Layer:** Utilizes memory (Moka) and disk caching.

## Usage
Run the binary:
```bash
cargo run --release --bin polyglid-ai -- --help
```
