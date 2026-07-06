To transform your offensive testing framework into a reality for **PolyGlid**, we must plan the implementation of each plugin as an independent, sandboxed **WebAssembly Component (`wasm32-wasip1`)**.

Because WASM is inherently sandboxed, your plugins cannot natively execute low-level socket manipulations or read files without explicit host permission. Therefore, our design introduces **Host Extensions (Capabilities)** that the PolyGlid Core Engine exposes to the WASM modules safely.

Here is the master step-by-step implementation plan for the 5 core offensive security plugins of PolyGlid.

---

## 🛠️ PolyGlid Plugin Architecture Foundation

Every plugin you write will export a unified interface to the UI workspace, but request specific underlying system permissions from the PolyGlid core engine.

```
+--------------------------------------------------------------------------+
|                        PolyGlid Host Runtime (Tauri)                     |
|  Exposes: [Network Sockets Capability]  |  [File System I/O Capability]  |
+--------------------------------------------------------------------------+
                                    ||  (Strict Permission Boundary)
+--------------------------------------------------------------------------+
|                        Isolated WASM Sandbox Layers                     |
|  [Plugin 1: Recon]   [Plugin 2: Audit]   [Plugin 3: Auth]   [Plugin 4]   |
+--------------------------------------------------------------------------+

```

---

## 📋 The 5-Phase Plugin Implementation Plan

### Phase 1: Reconnaissance (Subdomain & Port Sweep Engine)

* **Objective:** Replace Nmap and Subfinder with a unified, non-blocking UI dashboard page.
* **What it does:** Scans target IPs for active TCP ports and brute-forces subdomains using a chosen wordlist.
* **WASM Sandbox Requirement:** Requests `HostNetworkConnect` capability.
* **Implementation Steps:**
1. **UI Layout:** Create a dual-input form layout (Target Domain/IP + Wordlist selection) with a real-time grid that populates open ports asynchronously.
2. **Core Logic:** The plugin reads line-by-line inputs from the wordlist via a stream, dispatches DNS query tasks, and fires parallel asynchronous TCP socket creation loops.
3. **Rust Crate Ecosystem:** `tokio` (for handling async task loops), `trust-dns-resolver` (for high-speed DNS querying).



### Phase 2: Vulnerability Assessment (Banner Parser & CVE Matcher)

* **Objective:** Replace Nuclei/Searchsploit with an offline, lightning-fast matching machine.
* **What it does:** Takes the open port banners grabbed in Phase 1 (e.g., `SSH-2.0-OpenSSH_8.2p1`), parses the exact version numbers, and cross-references them against an offline local JSON database of known CVE vulnerabilities.
* **WASM Sandbox Requirement:** Requests `HostFileSystemRead` capability (to parse the local CVE JSON database).
* **Implementation Steps:**
1. **UI Layout:** A searchable datatable workspace split screen: Left side shows active target banners; Right side renders interactive markdown descriptions of matching CVE risks.
2. **Core Logic:** Write custom Regex parsers within the plugin to handle raw banner data cleanly and isolate version strings. Use high-speed text indices to query the vulnerability payload file.
3. **Rust Crate Ecosystem:** `regex`, `serde_json` (for memory-efficient, fast JSON schema serialization).



### Phase 3: Exploitation Testing (Multi-Threaded Service Auditor)

* **Objective:** Replace Hydra/Metasploit brute-forcers with a memory-safe protocol tester.
* **What it does:** Tests the strength of administrative gateways (like exposed SSH or FTP ports found in Phase 1) by attempting credential validation loops against a provided list of test profiles.
* **WASM Sandbox Requirement:** Requests `HostNetworkConnect` capability + Rate-limiting controls.
* **Implementation Steps:**
1. **UI Layout:** An offensive terminal visualizer showing real-time password attempts, response times, and an immediate red/green credential validation indicator.
2. **Core Logic:** Establish asynchronous protocol handshake channels. The plugin opens channels, handles authentication payloads, catches error blocks (like authentication failures), and drops the connection cleanly without triggering memory leaks.
3. **Rust Crate Ecosystem:** `ssh2` (for native SSH interop channels), `reqwest` (for basic web authentication endpoints).



### Phase 4: Post-Exploitation (Linux Local System Security Auditor)

* **Objective:** Replace LinPEAS with a fully sandboxed, fast auditing UI module.
* **What it does:** When dropped onto an administrative testing machine, it performs rapid discovery of misconfigured files, loose environment parameters, and dangerous file permissions.
* **WASM Sandbox Requirement:** Requests `HostFileSystemRead` (Root system scope access context).
* **Implementation Steps:**
1. **UI Layout:** An executive summary page layout classifying server health metrics into High, Medium, and Low risk vulnerabilities with clear patch suggestions.
2. **Core Logic:** The plugin recursively loops through system configuration clusters (`/etc/passwd`, SUID binaries, active systemd timers). It reads permissions flags and reports anomalies instantly.
3. **Rust Crate Ecosystem:** `nix` (for reading UNIX system files and checking user/group bit identities safely).



### Phase 5: Command & Control (Encrypted Communication Node)

* **Objective:** Replace simple Netcat shells with a secure, modern, cryptographic communications matrix.
* **What it does:** Manages an outbound secure stream node that acts as a secure target communication relay using end-to-end encryption.
* **WASM Sandbox Requirement:** Requests `HostNetworkListen` + `Cryptography` extensions.
* **Implementation Steps:**
1. **UI Layout:** An interactive command-line workspace shell that mimics terminal execution tabs directly inside the PolyGlid interface.
2. **Core Logic:** The plugin instantiates an asynchronous listener loop. All inbound traffic passing into the node is wrapped in symmetric encryption, processed via specialized commands, and output cleanly to the console viewport.
3. **Rust Crate Ecosystem:** `aes-gcm` (for hardware-accelerated Authenticated Encryption), `bytes` (for ultra-fast, clean network byte stream buffer management).



---

## 🧪 Unified Testing Framework Plan for Plugins

To make sure these plugins can be written and verified easily without compiling the full graphical front-end shell every single time, you will implement an internal testing harness:

```text
polyglid/
└── development_harness/
    ├── Cargo.toml
    └── src/
        └── main.rs      <-- Simulates PolyGlid UI to run and debug plugins via the terminal

```

### The Plugin Testing Lifecycle:

1. **Compile to WASM:** You write your plugin code in Rust and build it using:
```bash
cargo build --target wasm32-wasip1 --release

```


2. **Run inside the Harness:** Execute the test suite command line to simulate host permissions:
```bash
cargo run -p development_harness -- --plugin ./target/wasm32-wasip1/release/port_scanner.wasm --target "127.0.0.1"

```


3. **Validate Structure:** The harness validates that the WebAssembly payload complies with PolyGlid’s component contracts before passing it forward to production.

This end-to-end plan integrates all your targets, architectural constraints, and learning goals into a concrete development path for **PolyGlid Security Workspace**.