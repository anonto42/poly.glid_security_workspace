# PolyGlid Workspace & Monorepo Guide

This document describes the codebase management structure of the **PolyGlid** polyglot monorepo and summarizes the conversation history, milestones, and development journey.

---

## 🏗️ 1. Codebase Management Architecture (Nx-First Polyglot Monorepo)

To coordinate multiple languages and frameworks (Rust, Node.js, TypeScript, Tauri) without creating package manager conflicts, the codebase is structured as a **Raw Nx-First Polyglot Monorepo**:

```text
polyglid-workspace/                  # 1. WORKSPACE ROOT (Strictly managed by Nx)
├── nx.json                          # Global task pipeline & caching definitions
├── package.json                     # Root configuration for installing the Nx CLI tool
├── Cargo.toml                       # 2. RUST WORKSPACE ENGINE (Cargo)
│
├── apps/                            # 3. APPLICATIONS LAYER
│   ├── web/
│   │   ├── package.json             # React SPA (Vite + TypeScript) managed by pnpm
│   │   ├── pnpm-lock.yaml           # Local pnpm dependency locks
│   │   └── project.json             # Nx targets mapping tasks to local apps/web/
│   │
│   └── desktop/
│       ├── package.json             # Tauri UI (Vite + React) managed by npm
│       ├── package-lock.json        # Local npm dependency locks
│       └── project.json             # Nx targets mapping tasks to local apps/desktop/
│
└── crates/                          # 4. ENGINE CRATES
    ├── polyglid-core/               # Core workspace logic, database store, API services
    ├── polyglid-server/             # Axum HTTP/WS API backend server
    ├── polyglid-cli/                # Command-line interface tool
    └── project.json                 # Nx targets mapping tasks to Cargo commands
```

### Key Principles of the Structure
1. **Isolated Subdirectories:** Subprojects under `apps/` maintain their own node modules, package managers (`pnpm` for web, `npm` for desktop), and lockfiles. There is no shared node package manager at the root workspace.
2. **Local Cwd Tasks:** Target commands inside `project.json` files are configured with `"cwd": "<subdirectory>"` parameters. Nx runs scripts natively inside each folder.
3. **Unified Orchestration:** The global Nx CLI coordinates and caches tasks (builds, checks, tests) across all projects.

---

## 📜 2. Project Conversation History & Milestones

The development of PolyGlid has progressed through several major structural and functional phases, transitioning from basic feature implementations into a highly secure, platform-grade workspace:

* **✅ Milestone 1: Architectural Definition**
  Created core systems documentation (`ARCHITECTURE.md`) defining the boundaries between the host engine, Webview shell, and untrusted WASM plugins.
* **✅ Milestone 2: Plugin Lifecycle & Persistence**
  Designed runtime contracts (WIT definitions) using Wasmtime. Built the plugin sandbox registry and database storage layers using SQLite (isolating database queries inside the Core Store layer).
* **✅ Milestone 3: Command-line Execution (CLI)**
  Created the `polyglid` CLI tool to run checks, manage target lists, and invoke sandboxed WASM plugins directly from the terminal.
* **✅ Milestone 4: Tauri Desktop & TUI**
  Developed the native desktop app shell using Tauri v2 and built a terminal TUI layout for console monitoring.
* **✅ Milestone 5: Headless API Backend Server (v0.9.0)**
  Decoupled the web interface by spawning a dedicated Axum HTTP and WebSocket API server crate (`polyglid-server`) to handle executions, targets, reports, and settings.
* **✅ Milestone 6: Multi-User Collaboration & RBAC (Phase 13)**
  * Implemented database schema migration v4 for users, teams, and session tokens.
  * Added secure authentication session verification middleware and Role-Based Access Control guards (`Owner`, `Editor`, `Viewer`) to protect REST endpoints.
  * Integrated registration/login interfaces, a Team Collaboration dashboard, and UI checks into the React client.
* **✅ Milestone 7: Nx Polyglot Monorepo Migration (Current)**
  Organized the repository into a raw Nx template to enable local subdirectory package isolation, task pipeline dependencies, and blazing-fast computation caching.
