The current repository ownership, runtime sequence, development order, and
GitHub automation routing are documented in [Project Flow](PROJECT_FLOW.md).

> **Historical implementation blueprint.** The Tauri host and illustrative
> file paths/code below predate the current Rust/Dioxus desktop. Use
> [Client Architecture](CLIENT_ARCHITECTURE.md) for client boundaries and
> [Desktop UI](DESKTOP_UI.md) for the implemented component map. The WIT
> isolation principles remain background context, not a copy-ready plan.

To make **PolyGlid** a true plug-and-play workspace, we will use the **WebAssembly Component Model**. Instead of writing standard traits that risk crashing your host application, we define a formal language-agnostic interface called a **WIT file (WebAssembly Interface Type)**.

Think of the WIT file as the contract rulebook. The Core Engine reads it to know what functions it can call inside the plugin, and the plugin reads it to know what data to return.

Here is the architectural layout, rules, and code documentation blueprint for **PolyGlid Core-to-Plugin Interoperability**.

---

## 📐 1. Architectural Interaction Model

The host engine manages permissions and passes instructions to the sandboxed runtime. The plugin computes within its container and streams results back across the boundary line.

```
+--------------------------------------------------------------------------+
|                     POLYGLID CORE ENGINE (Tauri Host)                     |
|                                                                          |
|  1. Reads target configuration string                                    |
|  2. Allocates isolated runtime memory via Wasmtime                       |
|  3. Invokes plugin execution channel ==> [ Execution Bridge ]            |
+--------------------------------------------------------------------------+
                                     ||
                       (WIT Interface Abstraction Layer)
                                     ||
+--------------------------------------------------------------------------+
|                     POLYGLID PLUGIN SANDBOX (WASM)                       |
|                                                                          |
|  [ Internal Logic Loop ] <=========== Receives Target Configurations      |
|  1. Executes security testing commands                                   |
|  2. Collects issue payloads and structures vectors                       |
|  3. Yields results cleanly back to Host App                              |
+--------------------------------------------------------------------------+

```

---

## 📜 2. Architectural Rules & Lifecycle

1. **Strict Isolation (Sandbox Rule):** Plugins *cannot* access system files, spawn processes, or listen to raw network ports directly. If a plugin needs to connect to a target, it must request access via the host runtime layer.
2. **No Shared Memory Errors:** WebAssembly modules communicate using static values (integers, strings, arrays). Rust's compiler guarantees memory safety inside the plugin, while `Wasmtime` ensures safety at the boundary layer.
3. **Deterministic Return Format:** Every security plugin must output data matching the structured `plugin-report` schema so the frontend workspace panels can render the UI components instantly without parsing strings.

---

## 📄 3. Code Implementation Document

Here is the complete code blueprint for your workspace files.

### Step A: Defining the Contract (`contracts/polyglid.wit`)

Create this file at the root. This is the shared API rule definition used by both your core engine and your dynamic plugins.

```wit
package polyglid:engine@0.1.0;

interface types {
    // Shared structural layout returned by all diagnostics modules
    record plugin-report {
        plugin-name: string,
        target-tested: string,
        is-vulnerable: bool,
        issues: list<string>,
    }
}

world security-tool {
    use types.{plugin-report};
    
    // The Core Engine expects every plugin to expose this callable function
    export execute: func(target: string) -> result<plugin-report, string>;
}

```

---

### Step B: Writing a Plugin (`plugins/recon-probe/src/lib.rs`)

This is the code for an independent plugin module. It uses `wit-bindgen` to automatically read the contract file and generate matching types.

```rust
// plugins/recon-probe/Cargo.toml needs: wit-bindgen = "0.58.0"

// Generate the Rust structures automatically from our WIT file definitions
wit_bindgen::generate!({
    world: "security-tool",
    path: "../polyglid-contracts",
});

// Create our structural implementation component
struct ReconPlugin;

// Export our implementation matching the generated Guest trait rules
impl Guest for ReconPlugin {
    fn execute(target: String) -> Result<exports::polyglid::engine::types::PluginReport, String> {
        // Perform safe sandboxed logic calculations here...
        let mut discovered_issues = Vec::new();
        
        if target == "127.0.0.1" || target == "localhost" {
            discovered_issues.push("Localhost loopback target flag flagged.".to_string());
        }
        
        discovered_issues.push("Port 80 active: Plaintext HTTP protocol surface identified.".to_string());

        // Construct the strict structure response
        Ok(exports::polyglid::engine::types::PluginReport {
            plugin_name: "PolyGlid Native Port Recon".to_string(),
            target_tested: target,
            is_vulnerable: true,
            issues: discovered_issues,
        })
    }
}

// Export the component hooks globally inside the WASM header space
export!(ReconPlugin);

```

---

### Step C: The Core App Runner (`core_engine/src/main.rs`)

This file lives inside the main application shell. It reads the compiled `.wasm` file from the file system, initializes the execution space, and runs it safely.

```rust
// core_engine/Cargo.toml needs: wasmtime = "29.0.0"
use wasmtime::{Config, Engine, Module, Store, Linker};
use std::path::Path;

// Macros generate the structures for the Host side
wasmtime::component::bindgen!({
    world: "security-tool",
    path: "../polyglid-contracts",
});

fn run_sandboxed_plugin(wasm_path: &Path, target_host: &str) -> Result<(), anyhow::Error> {
    // 1. Initialize the Wasmtime Execution Engine with Component Model flags active
    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;
    
    // 2. Load the pre-compiled plugin binary from the file system
    let component = wasmtime::component::Component::from_file(&engine, wasm_path)?;
    
    // 3. Provision the localized state store and interface linker links
    let mut store = Store::new(&engine, ());
    let linker = Linker::new(&engine);
    
    // 4. Instantiate the plugin world within the environment bounds
    let (security_tool, _) = SecurityTool::instantiate(&mut store, &component, &linker)?;
    
    println!("[Core] Handshaking with WASM module sandbox layer...");
    
    // 5. Invoke the exposed function safely across the boundary line
    match security_tool.call_execute(&mut store, target_host) {
        Ok(Ok(report)) => {
            println!("\n==============================================");
            println!("Plugin Module: {}", report.plugin_name);
            println!("Target Tested: {}", report.target_tested);
            println!("Vulnerable:    {}", if report.is_vulnerable { "YES ⚠️" } else { "NO ✅" });
            println!("Discovered System Anomalies:");
            for issue in report.issues {
                println!(" -> {}", issue);
            }
            println!("==============================================");
        }
        Ok(Err(plugin_err)) => println!("[Plugin Error] Task aborted internally: {}", plugin_err),
        Err(runtime_err) => println!("[Runtime Panic] Sandbox crashed or violated memory bounds: {}", runtime_err),
    }

    Ok(())
}

fn main() {
    println!("=== PolyGlid Core Sandbox Host Booting ===");
    // In production, this path will point directly to your localized /plugins directory
    let plugin_target = Path::new("./target/wasm32-wasip1/release/recon_tool.wasm");
    
    if plugin_target.exists() {
        if let Err(e) = run_sandboxed_plugin(plugin_target, "127.0.0.1") {
            println!("Execution Failure: {:?}", e);
        }
    } else {
        println!("[System Configuration] Plugin binary not found. Run compilation pipeline first.");
    }
}

```

---

## 🎯 Why This Architecture is Bulletproof

By implementing this structure, you solve the cross-platform problem completely.

* If your **UI frontend** wants to trigger a scan, it sends an IPC command down to your **Tauri Rust app core**.
* The **Tauri Core** simply triggers `run_sandboxed_plugin` against the chosen `.wasm` file.
* The execution happens entirely in a safe sandbox—protecting the master software dashboard layer across Linux, Windows, Mac, and Android simultaneously.
