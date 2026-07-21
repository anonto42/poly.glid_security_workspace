# Models Directory

## embeddings/
Vector store for code embeddings. Populated by `polyglid-ai ingest`.
- `index.json` — JSON array of `{id, file, start_line, content, embedding: [f32; 768]}` chunks

## onnx/
Exported ONNX models for zero-dependency inference (planned).

## build-optimization/
Fine-tuned LoRA adapters for build optimization suggestions.

## code-completion/
Fine-tuned LoRA adapters for code generation.

## test-prediction/
Fine-tuned LoRA adapters for test generation.
