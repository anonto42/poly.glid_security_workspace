//! Runtime adapter for WebAssembly Component Model plugins.

use std::path::Path;

use polyglid_core::{CoreError, PluginRef, PluginRunRequest, PluginRuntime};
use polyglid_plugin_api::{
    Issue as ApiIssue, PluginId, PluginManifest, PluginReport as ApiPluginReport,
    Severity as ApiSeverity,
};
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
