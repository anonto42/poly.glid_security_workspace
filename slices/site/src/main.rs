use std::path::Path;
use tera::{Context, Tera};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let src = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tera = Tera::new(&format!("{}/templates/**/*", src.display()))?;

    let mut ctx = Context::new();
    ctx.insert("version", env!("CARGO_PKG_VERSION"));
    ctx.insert("name", "PolyGlid");
    ctx.insert(
        "tagline",
        "Sandboxed security testing workspace. Plugins run in Wasmtime behind explicit permission gates.",
    );
    ctx.insert(
        "description",
        "PolyGlid is a modular security diagnostics platform where plugins are untrusted by default. Each plugin must request capabilities (DNS resolve, file read, network connect, etc.) and the user explicitly approves or denies each one. The host engine runs locally — no cloud, no telemetry.",
    );
    let features = serde_json::json!([
        {"name": "Wasmtime sandbox", "desc": "Every plugin executes in an isolated WebAssembly runtime. No VMs, no containers, no host access by default."},
        {"name": "Capability-based permissions", "desc": "Plugins declare what they need. You approve or deny every capability — before and during execution."},
        {"name": "Multi-surface UI", "desc": "Desktop app (Dioxus), terminal client (Ratatui), and API server — all from the same core engine."},
        {"name": "SQLite registry", "desc": "Plugins, permissions, reports, and settings persist locally. No external database required."},
        {"name": "Structured reports", "desc": "Results export as JSON, Markdown, HTML, or SARIF 2.1.0. Integrate with your existing pipeline."},
        {"name": "First-party WASM plugin", "desc": "Recon Probe ships as a real reconnaissance plugin — DNS resolution, target diagnostics, and native panel rendering."},
    ]);
    ctx.insert("features", &features);

    let html = tera.render("index.html", &ctx)?;
    let out = src.join("public").join("index.html");
    std::fs::create_dir_all(out.parent().unwrap())?;
    std::fs::write(&out, html)?;
    println!("Site generated: {}", out.display());
    Ok(())
}
