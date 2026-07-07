# Model Configs — Per-Domain Overrides

Each `.toml` file here overrides the default model for a specific domain.
Loaded by the AI engine on startup if the file exists.

## Domain → file mapping

| Domain      | File                | Default model         |
|-------------|---------------------|-----------------------|
| code        | `code-model.toml`   | `phi3:3.8b`          |
| security    | `security-model.toml`| `phi3:3.8b`          |
| build       | `build-model.toml`  | `phi3:3.8b`          |
| suggest     | `suggest-model.toml`| `phi3:3.8b`          |

If a file is missing, the engine falls back to `[models]` in `ai-config.toml`,
then to the top-level `model` field in `ai-config.toml`.
