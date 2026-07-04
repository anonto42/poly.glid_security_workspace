# PolyGlid Plugin SDK

Welcome to the PolyGlid Plugin SDK. This SDK provides templates, examples, and tooling to help developers build and verify custom WASM plugins.

## Compatibility Matrix

| PolyGlid Host | SDK Version | WIT Interface Contract | Plugin ABI State |
| ------------- | ----------- | ---------------------- | ---------------- |
| 0.9           | 0.9.0       | v0.2.0                 | Compatible       |
| 1.0 (Planned) | 1.0.0       | v0.3.0                 | Compatible       |

---

## SDK Project Layout

- `plugin-template/`: A ready-to-run template for bootstrap configurations.
- `examples/hello_world/`: Basic component returning text printouts.
- `examples/recon_probe/`: Advanced plugin executing DNS resolving and report listings.

---

## Getting Started

### 1. Generate Plugin from Template
Copy the `plugin-template/` directory to create your new plugin project.

### 2. Implement the WIT Interface
Edit `src/lib.rs` and implement the generated trait bindings:
```rust
struct MyPlugin;

impl Guest for MyPlugin {
    fn run(target: String) -> PluginReport {
        // Your logic here
    }
}
```

### 3. Build Guest Component
Compile the Rust project to target WASM:
```bash
cargo build --target wasm32-wasip1 --release
```

### 4. Detached Cryptographic Signing
To sign your plugin for distribution under workspaces configured with Balanced/Strict policies:
```bash
polyglid plugin sign target/wasm32-wasip1/release/my_plugin.wasm --key path/to/private_key.pem
```
This generates `my_plugin.sig` next to the binary containing signature tracing metadata.
