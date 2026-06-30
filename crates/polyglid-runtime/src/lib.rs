//! Runtime adapter for WebAssembly Component Model plugins.

use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use polyglid_core::{CoreError, PluginRef, PluginRunRequest, PluginRuntime};
use polyglid_plugin_api::{
    Capability, Issue as ApiIssue, PluginId, PluginManifest, PluginReport as ApiPluginReport,
    Severity as ApiSeverity,
};
use serde::Deserialize;
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

wasmtime::component::bindgen!({
    world: "security-tool",
    path: "../../wit",
});

#[derive(Debug, Default)]
pub struct WasmRuntime;

impl WasmRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl PluginRuntime for WasmRuntime {
    fn inspect(&self, plugin: &PluginRef) -> Result<PluginManifest, CoreError> {
        ensure_file_exists(plugin.path())?;
        if let Some(manifest_path) = manifest_path_for(plugin.path()) {
            return read_manifest(&manifest_path);
        }

        let id = plugin
            .path()
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("plugin");

        Ok(PluginManifest {
            id: PluginId::new(id).map_err(|err| CoreError::Runtime(err.to_string()))?,
            name: id.replace('_', " "),
            version: "0.1.0".to_string(),
            requested_capabilities: Vec::new(),
        })
    }

    fn execute(&self, request: &PluginRunRequest) -> Result<ApiPluginReport, CoreError> {
        ensure_file_exists(request.plugin.path())?;
        run_component(request.plugin.path(), request.target.as_str())
    }
}

fn ensure_file_exists(path: &Path) -> Result<(), CoreError> {
    if path.is_file() {
        Ok(())
    } else {
        Err(CoreError::PluginNotFound(path.to_path_buf()))
    }
}

fn manifest_path_for(plugin_path: &Path) -> Option<PathBuf> {
    let same_name = plugin_path.with_extension("polyglid.toml");
    if same_name.is_file() {
        return Some(same_name);
    }

    let same_dir = plugin_path.parent()?.join("polyglid.toml");
    if same_dir.is_file() {
        return Some(same_dir);
    }

    for stem in manifest_stems(plugin_path) {
        let source_manifest = Path::new("plugins").join(stem).join("polyglid.toml");
        if source_manifest.is_file() {
            return Some(source_manifest);
        }
    }

    None
}

fn read_manifest(path: &Path) -> Result<PluginManifest, CoreError> {
    let raw = fs::read_to_string(path).map_runtime_error()?;
    let manifest: RawPluginManifest = toml::from_str(&raw).map_runtime_error()?;
    let id = PluginId::new(manifest.id).map_err(|err| CoreError::Runtime(err.to_string()))?;
    let requested_capabilities = manifest
        .capabilities
        .into_iter()
        .map(|capability| Capability::from_str(&capability))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| CoreError::Runtime(err.to_string()))?;

    Ok(PluginManifest {
        id,
        name: manifest.name,
        version: manifest.version,
        requested_capabilities,
    })
}

#[derive(Debug, Deserialize)]
struct RawPluginManifest {
    id: String,
    name: String,
    version: String,
    #[allow(dead_code)]
    entry_world: Option<String>,
    #[serde(default)]
    capabilities: Vec<String>,
}

fn manifest_stems(plugin_path: &Path) -> Vec<String> {
    let Some(stem) = plugin_path.file_stem().and_then(|value| value.to_str()) else {
        return Vec::new();
    };

    let mut stems = vec![stem.to_string()];
    if let Some(stripped) = stem.strip_suffix(".component") {
        stems.push(stripped.to_string());
    }
    stems
}

fn run_component(path: &Path, target: &str) -> Result<ApiPluginReport, CoreError> {
    let mut config = Config::new();
    config.wasm_component_model(true);

    let engine = Engine::new(&config).map_runtime_error()?;
    let component = Component::from_file(&engine, path).map_runtime_error()?;
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker).map_runtime_error()?;
    let mut store = Store::new(&engine, RuntimeState::default());

    let bindings =
        SecurityTool::instantiate(&mut store, &component, &linker).map_runtime_error()?;
    let report = bindings
        .call_execute(&mut store, target)
        .map_runtime_error()?
        .map_err(CoreError::Runtime)?;

    Ok(report.into())
}

struct RuntimeState {
    table: ResourceTable,
    wasi: WasiCtx,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            table: ResourceTable::new(),
            wasi: WasiCtxBuilder::new().build(),
        }
    }
}

impl WasiView for RuntimeState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

trait RuntimeResultExt<T> {
    fn map_runtime_error(self) -> Result<T, CoreError>;
}

impl<T, E> RuntimeResultExt<T> for Result<T, E>
where
    E: std::fmt::Display,
{
    fn map_runtime_error(self) -> Result<T, CoreError> {
        self.map_err(|err| CoreError::Runtime(err.to_string()))
    }
}

impl From<PluginReport> for ApiPluginReport {
    fn from(report: PluginReport) -> Self {
        Self {
            plugin_name: report.plugin_name,
            target_tested: report.target_tested,
            issues: report.issues.into_iter().map(Into::into).collect(),
            summary: report.summary,
        }
    }
}

impl From<polyglid::engine::types::Issue> for ApiIssue {
    fn from(issue: polyglid::engine::types::Issue) -> Self {
        Self {
            title: issue.title,
            severity: issue.severity.into(),
            description: issue.description,
            recommendation: issue.recommendation,
        }
    }
}

impl From<polyglid::engine::types::Severity> for ApiSeverity {
    fn from(severity: polyglid::engine::types::Severity) -> Self {
        match severity {
            polyglid::engine::types::Severity::Info => Self::Info,
            polyglid::engine::types::Severity::Low => Self::Low,
            polyglid::engine::types::Severity::Medium => Self::Medium,
            polyglid::engine::types::Severity::High => Self::High,
            polyglid::engine::types::Severity::Critical => Self::Critical,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_stems_strip_component_suffix() {
        let stems = manifest_stems(Path::new("target/recon_probe.component.wasm"));

        assert_eq!(
            stems,
            vec![
                "recon_probe.component".to_string(),
                "recon_probe".to_string()
            ]
        );
    }

    #[test]
    fn reads_plugin_manifest_capabilities() {
        let dir =
            std::env::temp_dir().join(format!("polyglid-manifest-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let manifest_path = dir.join("polyglid.toml");
        std::fs::write(
            &manifest_path,
            r#"
id = "polyglid.test"
name = "Test Plugin"
version = "0.1.0"
entry_world = "security-tool"
capabilities = ["dns-resolve", "network-connect"]
"#,
        )
        .expect("write manifest");

        let manifest = read_manifest(&manifest_path).expect("manifest");

        assert_eq!(manifest.id.as_str(), "polyglid.test");
        assert_eq!(
            manifest.requested_capabilities,
            vec![Capability::DnsResolve, Capability::NetworkConnect]
        );

        let _ = std::fs::remove_file(manifest_path);
        let _ = std::fs::remove_dir(dir);
    }
}
