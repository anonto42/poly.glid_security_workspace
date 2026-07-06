# Packaging And Distribution

PolyGlid should be designed so the core engine can be packaged into runnable executables for different devices and operating systems.

## Short Answer

Yes, the CLI and core engine can be packaged into one runnable executable per platform.

Example release artifacts:

```text
polyglid-linux-x86_64
polyglid-linux-aarch64
polyglid-macos-aarch64
polyglid-macos-x86_64
polyglid-windows-x86_64.exe
```

The desktop app will be packaged differently because Tauri creates native app bundles/installers.

## What Goes Into One CLI Binary

The CLI executable can include:

- command parser
- terminal UI
- core engine
- config validation
- permission checks
- plugin runtime adapter
- embedded default config
- optional embedded first-party demo plugin metadata

The CLI executable should not include:

- user secrets
- local machine config
- downloaded third-party plugins
- generated reports
- large vulnerability databases

Those should live in platform-specific app data directories.

## CLI Packaging Model

```text
polyglid executable
      |
      v
loads config from user data directory
      |
      v
loads plugins from plugin directory
      |
      v
runs plugins through embedded runtime
```

The binary is portable, while config and plugins are external data.

## Desktop Packaging Model

The Tauri app packages:

- frontend assets
- Rust host engine
- Tauri command handlers
- plugin runtime
- default config

It still loads user-approved plugins and user config from local app data.

## Plugin Packaging Model

Plugins should be distributed as `.wasm` components plus a manifest.

Example:

```text
recon_probe/
├── plugin.yaml
└── recon_probe.wasm
```

Later this can become a compressed package:

```text
recon_probe.polyglid-plugin
```

## Cross-Platform Data Locations

Use a Rust crate such as `directories` to find correct OS-specific locations.

Expected examples:

```text
Linux:   ~/.local/share/polyglid
macOS:   ~/Library/Application Support/PolyGlid
Windows: C:\Users\<user>\AppData\Roaming\PolyGlid
```

Inside the app data directory:

```text
polyglid/
├── config/
│   └── config.toml
├── plugins/
│   └── recon_probe/
├── reports/
├── logs/
└── cache/
```

## Why Not Put Everything Inside The Binary?

Putting everything in one binary is good for the engine, but not for all data.

Keep external:

- plugins, so they can be added or removed
- config, so users can change settings
- reports, so generated output is preserved
- cache, so it can be deleted safely
- secrets, so they are never committed or embedded

## Build Targets

The first target is the local development platform.

Then add common release targets:

```text
x86_64-unknown-linux-gnu
aarch64-unknown-linux-gnu
x86_64-apple-darwin
aarch64-apple-darwin
x86_64-pc-windows-msvc
```

## Release Stages

### Stage 1: Local Binary

```bash
cargo build -p polyglid-cli
```

Goal: run locally during development.

### Stage 2: Optimized CLI Release

```bash
cargo build -p polyglid-cli --release
```

Goal: produce one fast executable for the current device.

### Stage 3: Multi-Platform CLI Releases

Use CI release builds to create Linux, macOS, and Windows binaries.

Goal: download the right binary on another device and run:

```bash
polyglid doctor
polyglid plugin run ./plugins/recon_probe/recon_probe.wasm --target example.com
```

### Stage 4: Tauri App Bundles

Build native installers through Tauri.

Goal: desktop users install a normal app while the same Rust core still powers the CLI.

## Design Rule

Code should never assume a hardcoded platform path or shell.

Use:

- `PathBuf` for paths
- `directories` for OS data directories
- Rust APIs before shell commands
- adapters for OS-specific behavior
- feature flags only when a dependency is truly platform-specific

