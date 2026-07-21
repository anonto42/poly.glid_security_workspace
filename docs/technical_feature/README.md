# PolyGlid Technical Feature Documentation

> **Historical Tauri concept.** This document is retained as product ideation;
> it does not describe the current Dioxus implementation. Use
> [Client Architecture](../architecture/CLIENT_ARCHITECTURE.md) and
> [Desktop UI](../architecture/DESKTOP_UI.md) for active decisions.

## 1. Multi-Window Workspace Engine

PolyGlid provides an IDE-like interface where multiple tabs can be broken out into independent, concurrent windows across both desktop (Windows, Linux, macOS) and mobile (Android tablets/foldables).

### Architectural Behavior:

* **The Main Anchor Shell:** The primary host app layout window. It acts as the central router and keeps state persistence alive.
* **Webview Fracturing:** Users can drag any diagnostic page tab (e.g., *Port Scanner*) to the edge of the screen to split the viewport vertically or horizontally, or drag it out completely to spawn an entirely separate native operating system window.
* **Inter-Window Communication (`IPC`):** Built natively using Tauri v2 event dispatch channels. If an offensive scan catches a vulnerability in a background window, it emits a global event (`emit_to`) which updates the real-time activity charts in the main workspace dashboard instantly.

---

## 2. Sandboxed WebAssembly (WASM) Plugin Pipeline

To achieve absolute cross-platform parity where the exact same plugin runs on a Windows Server and an Android device, plugins are stripped of native `.so`/`.dll` runtime binaries and compiled into isolated WebAssembly modules.

```
       +--------------------------------------------+
       |             PolyGlid Host Engine           |
       |  [ Tauri Core ]  <== IPC ==>  [ Frontend ] |
       +--------------------------------------------+
                             ||
                 (Secure Sandboxed Boundary)
                             ||
       +--------------------------------------------+
       |             Wasmtime Runtime               |
       |  [ Plugin A: Scanner ]   [ Plugin B: C2 ]  |
       +--------------------------------------------+

```

### Feature Mechanics:

* **Zero-Trust Execution:** Plugins are loaded via the `Wasmtime` engine crate inside Rust. They have zero direct access to the host's filesystem, network cards, or environment variables unless the user explicitly grants them permission in the workspace settings panel.
* **Streaming Metrics:** As a plugin runs, it streams chunks of diagnostic data through a standard WebAssembly interface layer back to the core UI without locking up the user interface.

---

# PolyGlid Settings & Appearance Engine

To manage app appearance, engine states, and plugin permissions seamlessly, PolyGlid utilizes a structured system configuration file called `config.json`. This configuration is updated via the UI and handled natively on the backend using the `tauri-plugin-store` crate for persistent disk storage.

## 🛠️ The Global Configuration Schema

Below is the complete architectural layout of how PolyGlid stores app state, custom fonts, color modes, and security configurations:

```json
{
  "appearance": {
    "theme_mode": "dark",
    "font_interface": "Inter",
    "font_monospace": "JetBrains Mono",
    "font_size_ui": 13,
    "font_size_logs": 11,
    "panel_opacity": 0.95,
    "acrylic_blur": true
  },
  "workspace": {
    "default_layout": "split-vertical",
    "save_window_positions": true,
    "confirm_on_window_close": true,
    "max_concurrent_tasks": 5
  },
  "security_sandbox": {
    "isolate_plugins": true,
    "allow_network_scan_default": false,
    "allow_filesystem_read_default": false,
    "blocked_ip_ranges": ["127.0.0.1", "192.168.1.1"]
  }
}

```

---

## 🎨 Layout and Configuration Breakdown

### 1. The Appearance Control Block

Allows the engineer to fully control the visual fidelity of their workspace environment.

* **Theme Switcher:** Toggles application state properties between `dark` (Default Cyber Blue-Grey) and `light` (Clean Slate).
* **Typography Customization:** Configures the rendering stack to fall back cleanly on high-legibility developer choices like `JetBrains Mono` for log terminals and data matrices.
* **Window Vibrancy (`acrylic_blur`):** On Windows and macOS, this instructs the Tauri compiler to allow native window blurring behind surface panels, creating a modern, sleek aesthetic.

### 2. The Workspace Management Block

Controls how multi-task operations behave inside the core window stack.

* **Task Throttling (`max_concurrent_tasks`):** Prevents your system from crashing or freezing during heavy security loops. If set to `5`, the core engine will queue any additional automated plugin scans until an active slot becomes free.
* **State Recovery (`save_window_positions`):** When launching PolyGlid, the app reads this store parameter to automatically snap split windows back to their exact previous layouts and monitor positions.

### 3. The Security Sandbox Block (The Guardrail)

Because PolyGlid is an offensive testing framework, this system block ensures that malicious or buggy third-party plugins cannot run rampant on your machine.

* **Boundary Toggles:** Controls if a plugin has explicit permission to send raw network packets or scan local directories.
* **IP Blacklisting (`blocked_ip_ranges`):** Protects the engineer from accidentally launching automated scanning tools against forbidden targets (like localhost loopbacks or their company's core firewall gateways).

---

## 🚀 How Changes Apply in Real Time (The Reactive Cycle)

When a user modifies a setting (e.g., changes the UI theme from dark to light or increases log font size):

1. **Frontend Dispatches State Change:** Trigger.
The user clicks a setting inside the UI panel. The frontend environment fires a reactive action and updates the internal state manager.


2. **Write to Tauri Storage Engine:** Persist.
The frontend emits an IPC command invoking `tauri-plugin-store`. The new configuration payload is instantly written down to the local `config.json` system file.


3. **Broadcast State Events Globally:** Sync.
The Rust Core Engine catches the file write update, reflects changes on native backend properties, and broadcasts a global configuration synchronization event to all active open workspace windows.


4. **Dynamic DOM / CSS Re-Rendering:** Render.
Each independent webview context catches the sync event and dynamically updates CSS custom properties inline (e.g., `--text-size`, `--bg-color`) without needing to reload or restart the application shell.

