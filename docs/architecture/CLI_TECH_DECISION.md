# CLI Technology Decision

> **Historical decision — superseded for product direction.** The CLI now
> remains a frozen development and regression harness. Dioxus Desktop is the
> primary product client; see [Client Architecture](CLIENT_ARCHITECTURE.md).
> The `clap` implementation notes below remain useful when maintaining the
> harness.

PolyGlid starts with a CLI-first development flow. The CLI is the first real client for the engine, so the command parser and terminal UX should be stable enough to grow with the project.

## Decision

Use `clap` as the main Rust CLI framework.

Start version `0.1` with `clap` only, then add richer terminal UX crates when the command shape is stable.

Recommended stack:

```text
clap + dialoguer + console + indicatif
```

## Rust CLI Options

| Option | Best For | Notes |
| --- | --- | --- |
| `clap` | Serious production CLI | Most popular, powerful subcommands, flags, derive macros |
| `argh` | Small simple CLIs | Lightweight, less feature-rich |
| `bpaf` | Type-safe parser-heavy CLIs | Powerful, more advanced style |
| `pico-args` | Tiny tools | Minimal, manual parsing |
| `gumdrop` | Simple derive-based CLI | Nice but less common now |
| `structopt` | Older derive CLI | Mostly replaced by `clap` derive |

## Why `clap`

`clap` fits PolyGlid because the CLI will not stay tiny. It needs nested commands, clear help output, typed arguments, flags, and future shell completions.

PolyGlid CLI commands will likely grow into groups:

```bash
polyglid --help
polyglid doctor
polyglid init
polyglid config show
polyglid config validate
polyglid plugin list
polyglid plugin inspect ./plugin.wasm
polyglid plugin run ./plugin.wasm --target example.com
polyglid engine status
```

That command shape is exactly where `clap` is strong.

## Supporting Crates

Use these after the basic CLI skeleton works:

```text
dialoguer  -> interactive prompts and select menus
console    -> styled terminal output and color handling
indicatif  -> progress bars and spinners
```

Optional later:

```text
comfy-table -> nice tables
owo-colors  -> terminal colors
tracing     -> logs/debug mode
```

## CLI Architecture

The CLI is a client, not the engine.

```text
polyglid-cli
  |
  | uses clap to parse command
  v
polyglid-core
  |
  | calls runtime/config/events later
  v
polyglid-runtime
```

`polyglid-cli` should own:

- command parsing
- terminal output
- interactive prompts
- formatting tables
- calling core use cases

`polyglid-cli` should not own:

- plugin execution logic
- engine policy
- permission rules
- Wasmtime internals
- config persistence internals

## Version 0.1 Crates

Recommended dependencies:

```toml
clap = { version = "4", features = ["derive"] }
dialoguer = "0.11"
console = "0.15"
indicatif = "0.17"
comfy-table = "7"
anyhow = "1"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = "0.3"
```

For the first skeleton, start smaller:

```toml
clap = { version = "4", features = ["derive"] }
anyhow = "1"
```

Then add `console`, `dialoguer`, and `comfy-table` when the output style is being designed.

## First Output Goal

The first CLI milestone should produce useful output without needing the full engine.

Example:

```text
PolyGlid Security Workspace

Engine:    not initialized
Runtime:   not connected
Plugins:   0 discovered
Config:    missing

Next:
  polyglid init
  polyglid doctor
```

This gives the project a visible face early while the engine and plugin runtime are still being built.
