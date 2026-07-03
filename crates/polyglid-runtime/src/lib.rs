//! Runtime adapter for WebAssembly Component Model plugins.

use std::fs;
use std::net::ToSocketAddrs;
use std::path::{Component as PathComponent, Path, PathBuf};
use std::str::FromStr;

use polyglid_config::AppConfig;
use polyglid_core::{CoreError, PluginRef, PluginRunRequest, PluginRuntime};
use polyglid_plugin_api::{
    ApiPluginMetadata, Capability, CapabilityRequest, CapabilityScope, Issue as ApiIssue,
    PanelLayout as ApiPanelLayout, PanelWidget as ApiPanelWidget, PluginId, PluginManifest,
    PluginReport as ApiPluginReport, Severity as ApiSeverity, WidgetKind as ApiWidgetKind,
};
use serde::Deserialize;
use wasmtime::component::{Component, HasSelf, Linker};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

wasmtime::component::bindgen!({
    world: "security-tool",
    path: "../../wit",
});

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use uuid::Uuid;

pub static ACTIVE_ENGINES: OnceLock<Mutex<HashMap<Uuid, wasmtime::Engine>>> = OnceLock::new();

use polyglid_core::execution::CURRENT_JOB_ID;

pub fn register_engine(id: Uuid, engine: wasmtime::Engine) {
    let map = ACTIVE_ENGINES.get_or_init(|| Mutex::new(HashMap::new()));
    map.lock().unwrap().insert(id, engine);
}

pub fn unregister_engine(id: Uuid) {
    if let Some(map) = ACTIVE_ENGINES.get() {
        map.lock().unwrap().remove(&id);
    }
}

pub fn interrupt_engine(id: Uuid) -> bool {
    if let Some(map) = ACTIVE_ENGINES.get() {
        if let Some(engine) = map.lock().unwrap().get(&id) {
            engine.increment_epoch();
            return true;
        }
    }
    false
}

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

        // Try to extract metadata from the WASM component itself first.
        if let Ok((metadata, capabilities)) = inspect_from_wasm(plugin.path()) {
            let id =
                PluginId::new(&metadata.name).map_err(|err| CoreError::Runtime(err.to_string()))?;
            let requested_capabilities = capabilities
                .into_iter()
                .map(|cap_str| {
                    Capability::from_str(&cap_str)
                        .map(CapabilityRequest::unscoped)
                        .map_err(|err| CoreError::Runtime(err.to_string()))
                })
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(PluginManifest {
                id,
                name: metadata.display_name,
                version: metadata.version,
                requested_capabilities,
            });
        }

        // Fallback: try side-car manifest file (legacy path).
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

    fn execute(
        &self,
        request: &PluginRunRequest,
        config: &AppConfig,
    ) -> Result<ApiPluginReport, CoreError> {
        ensure_file_exists(request.plugin.path())?;
        run_component(
            request.plugin.path(),
            request.target.as_str(),
            config.reports_dir.clone(),
            config.max_wasm_fuel,
        )
    }

    fn cancel(&self, job_id: uuid::Uuid) -> Result<(), CoreError> {
        interrupt_engine(job_id);
        Ok(())
    }
}

impl WasmRuntime {
    /// Call the plugin's `metadata()` export to get embedded self-description.
    pub fn call_metadata(&self, plugin: &PluginRef) -> Result<ApiPluginMetadata, CoreError> {
        let (metadata, _) = inspect_from_wasm(plugin.path())?;
        Ok(metadata)
    }

    /// Call the plugin's `cli-panel(report)` export to get a TUI layout.
    pub fn call_cli_panel(
        &self,
        plugin: &PluginRef,
        report: &ApiPluginReport,
    ) -> Result<ApiPanelLayout, CoreError> {
        call_panel_export(plugin.path(), report, PanelKind::Cli)
    }

    /// Call the plugin's `desktop-panel(report)` export to get a desktop layout.
    pub fn call_desktop_panel(
        &self,
        plugin: &PluginRef,
        report: &ApiPluginReport,
    ) -> Result<ApiPanelLayout, CoreError> {
        call_panel_export(plugin.path(), report, PanelKind::Desktop)
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
    let mut requested_capabilities = manifest
        .capabilities
        .into_iter()
        .map(|capability| Capability::from_str(&capability))
        .map(|result| result.map(CapabilityRequest::unscoped))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| CoreError::Runtime(err.to_string()))?;
    for request in manifest.capability_requests {
        requested_capabilities.push(request.into_capability_request()?);
    }

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
    #[serde(default)]
    capability_requests: Vec<RawCapabilityRequest>,
}

#[derive(Debug, Deserialize)]
struct RawCapabilityRequest {
    capability: String,
    #[serde(default)]
    target: Option<String>,
    #[serde(default)]
    path_prefix: Option<String>,
    #[serde(default)]
    host: Option<String>,
    #[serde(default)]
    port: Option<u16>,
}

impl RawCapabilityRequest {
    fn into_capability_request(self) -> Result<CapabilityRequest, CoreError> {
        let capability = Capability::from_str(&self.capability)
            .map_err(|err| CoreError::Runtime(err.to_string()))?;
        let scope = match (self.target, self.path_prefix, self.host, self.port) {
            (Some(target), None, None, None) => CapabilityScope::Target(target),
            (None, Some(path_prefix), None, None) => CapabilityScope::PathPrefix(path_prefix),
            (None, None, Some(host), Some(port)) => CapabilityScope::HostPort { host, port },
            (None, None, None, None) => CapabilityScope::Any,
            _ => {
                return Err(CoreError::Runtime(
                    "capability request must use one scope shape".to_string(),
                ));
            }
        };

        Ok(CapabilityRequest::new(capability, scope))
    }
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

/// Instantiate the WASM component and call `metadata()` + `required-capabilities()`.
fn inspect_from_wasm(path: &Path) -> Result<(ApiPluginMetadata, Vec<String>), CoreError> {
    let (mut store, bindings) =
        instantiate_plugin(path, "_inspect_", PathBuf::from("reports"), 1_000_000)?;
    let metadata = bindings.call_metadata(&mut store).map_runtime_error()?;
    let capabilities = bindings
        .call_required_capabilities(&mut store)
        .map_runtime_error()?;
    Ok((metadata.into(), capabilities))
}

#[derive(Debug, Clone, Copy)]
enum PanelKind {
    Cli,
    Desktop,
}

/// Instantiate the WASM component and call `cli-panel(report)` or `desktop-panel(report)`.
fn call_panel_export(
    path: &Path,
    report: &ApiPluginReport,
    kind: PanelKind,
) -> Result<ApiPanelLayout, CoreError> {
    let (mut store, bindings) =
        instantiate_plugin(path, "_panel_", PathBuf::from("reports"), 1_000_000)?;
    let wit_report = api_report_to_wit(report);
    let panel = match kind {
        PanelKind::Cli => bindings
            .call_cli_panel(&mut store, &wit_report)
            .map_runtime_error()?,
        PanelKind::Desktop => bindings
            .call_desktop_panel(&mut store, &wit_report)
            .map_runtime_error()?,
    };
    Ok(panel.into())
}

/// Shared helper: build engine + linker + store + instantiate the plugin.
fn instantiate_plugin(
    path: &Path,
    allowed_dns_host: &str,
    reports_dir: PathBuf,
    max_fuel: u64,
) -> Result<(Store<RuntimeState>, SecurityTool), CoreError> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.consume_fuel(true);
    config.epoch_interruption(true);

    let engine = Engine::new(&config).map_runtime_error()?;
    let component = Component::from_file(&engine, path).map_runtime_error()?;
    
    // Register the engine clone under the current job ID if configured
    if let Some(job_id) = CURRENT_JOB_ID.with(|id| id.get()) {
        register_engine(job_id, engine.clone());
    }

    let mut linker = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker).map_runtime_error()?;
    polyglid::engine::dns::add_to_linker::<RuntimeState, HasSelf<RuntimeState>>(
        &mut linker,
        |state: &mut RuntimeState| state,
    )
    .map_runtime_error()?;
    polyglid::engine::reports::add_to_linker::<RuntimeState, HasSelf<RuntimeState>>(
        &mut linker,
        |state: &mut RuntimeState| state,
    )
    .map_runtime_error()?;
    
    let mut store = Store::new(&engine, RuntimeState::new(allowed_dns_host, reports_dir));
    store.set_fuel(max_fuel).map_runtime_error()?;
    store.set_epoch_deadline(1);

    let bindings =
        SecurityTool::instantiate(&mut store, &component, &linker).map_runtime_error()?;
    Ok((store, bindings))
}

fn run_component(
    path: &Path,
    target: &str,
    reports_dir: PathBuf,
    max_wasm_fuel: u64,
) -> Result<ApiPluginReport, CoreError> {
    let (mut store, bindings) = instantiate_plugin(path, target, reports_dir, max_wasm_fuel)?;
    let result = bindings
        .call_execute(&mut store, target)
        .map_runtime_error()?
        .map_err(CoreError::Runtime);

    if let Some(job_id) = CURRENT_JOB_ID.with(|id| id.get()) {
        unregister_engine(job_id);
    }

    result.map(Into::into)
}

/// Convert host API report back to the WIT representation (needed for panel calls).
fn api_report_to_wit(report: &ApiPluginReport) -> PluginReport {
    PluginReport {
        plugin_name: report.plugin_name.clone(),
        target_tested: report.target_tested.clone(),
        issues: report
            .issues
            .iter()
            .map(|issue| polyglid::engine::types::Issue {
                title: issue.title.clone(),
                severity: match issue.severity {
                    ApiSeverity::Info => polyglid::engine::types::Severity::Info,
                    ApiSeverity::Low => polyglid::engine::types::Severity::Low,
                    ApiSeverity::Medium => polyglid::engine::types::Severity::Medium,
                    ApiSeverity::High => polyglid::engine::types::Severity::High,
                    ApiSeverity::Critical => polyglid::engine::types::Severity::Critical,
                },
                description: issue.description.clone(),
                recommendation: issue.recommendation.clone(),
            })
            .collect(),
        summary: report.summary.clone(),
    }
}

struct RuntimeState {
    table: ResourceTable,
    wasi: WasiCtx,
    allowed_dns_host: String,
    reports_dir: PathBuf,
}

impl RuntimeState {
    fn new(allowed_dns_host: &str, reports_dir: PathBuf) -> Self {
        Self {
            table: ResourceTable::new(),
            wasi: WasiCtxBuilder::new().build(),
            allowed_dns_host: allowed_dns_host.to_string(),
            reports_dir,
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

impl polyglid::engine::dns::Host for RuntimeState {
    fn resolve(&mut self, host: String) -> Result<Vec<String>, String> {
        if host != self.allowed_dns_host {
            return Err(format!(
                "dns-resolve is scoped to {}",
                self.allowed_dns_host
            ));
        }

        let addresses = (host.as_str(), 0)
            .to_socket_addrs()
            .map_err(|err| err.to_string())?
            .map(|address| address.ip().to_string())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();

        Ok(addresses)
    }
}

impl polyglid::engine::reports::Host for RuntimeState {
    fn write(&mut self, filename: String, contents: String) -> Result<String, String> {
        let output_path = safe_report_path(&self.reports_dir, &filename)?;
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
        fs::write(&output_path, contents).map_err(|err| err.to_string())?;
        Ok(output_path.display().to_string())
    }
}

fn safe_report_path(reports_dir: &Path, filename: &str) -> Result<PathBuf, String> {
    let path = Path::new(filename);
    if path.as_os_str().is_empty() || path.is_absolute() {
        return Err("report filename must be relative".to_string());
    }
    if path
        .components()
        .any(|component| !matches!(component, PathComponent::Normal(_)))
    {
        return Err("report filename cannot contain path separators or traversal".to_string());
    }

    Ok(reports_dir.join(path))
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

impl From<polyglid::engine::types::PluginMetadata> for ApiPluginMetadata {
    fn from(m: polyglid::engine::types::PluginMetadata) -> Self {
        Self {
            name: m.name,
            display_name: m.display_name,
            version: m.version,
            description: m.description,
            author: m.author,
        }
    }
}

impl From<polyglid::engine::types::PanelLayout> for ApiPanelLayout {
    fn from(layout: polyglid::engine::types::PanelLayout) -> Self {
        Self {
            title: layout.title,
            widgets: layout.widgets.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<polyglid::engine::types::PanelWidget> for ApiPanelWidget {
    fn from(widget: polyglid::engine::types::PanelWidget) -> Self {
        Self {
            widget_kind: widget.widget_kind.into(),
            title: widget.title,
            data: widget.data,
        }
    }
}

impl From<polyglid::engine::types::WidgetType> for ApiWidgetKind {
    fn from(wt: polyglid::engine::types::WidgetType) -> Self {
        match wt {
            polyglid::engine::types::WidgetType::Table => Self::Table,
            polyglid::engine::types::WidgetType::KeyValue => Self::KeyValue,
            polyglid::engine::types::WidgetType::Tree => Self::Tree,
            polyglid::engine::types::WidgetType::Log => Self::Log,
            polyglid::engine::types::WidgetType::ChartBar => Self::ChartBar,
            polyglid::engine::types::WidgetType::TextBlock => Self::TextBlock,
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
            vec![
                CapabilityRequest::unscoped(Capability::DnsResolve),
                CapabilityRequest::unscoped(Capability::NetworkConnect)
            ]
        );

        let _ = std::fs::remove_file(manifest_path);
        let _ = std::fs::remove_dir(dir);
    }

    #[test]
    fn reads_scoped_plugin_manifest_requests() {
        let dir = std::env::temp_dir().join(format!(
            "polyglid-scoped-manifest-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let manifest_path = dir.join("polyglid.toml");
        std::fs::write(
            &manifest_path,
            r#"
id = "polyglid.test"
name = "Test Plugin"
version = "0.1.0"

[[capability_requests]]
capability = "network-connect"
host = "example.com"
port = 443

[[capability_requests]]
capability = "filesystem-read"
path_prefix = "/tmp/polyglid"
"#,
        )
        .expect("write manifest");

        let manifest = read_manifest(&manifest_path).expect("manifest");

        assert_eq!(
            manifest.requested_capabilities,
            vec![
                CapabilityRequest::new(
                    Capability::NetworkConnect,
                    CapabilityScope::HostPort {
                        host: "example.com".to_string(),
                        port: 443,
                    },
                ),
                CapabilityRequest::new(
                    Capability::FilesystemRead,
                    CapabilityScope::PathPrefix("/tmp/polyglid".to_string()),
                ),
            ]
        );

        let _ = std::fs::remove_file(manifest_path);
        let _ = std::fs::remove_dir(dir);
    }

    #[test]
    fn dns_host_import_is_scoped_to_run_target() {
        let mut state = RuntimeState::new("example.com", PathBuf::from("reports"));

        let err = polyglid::engine::dns::Host::resolve(&mut state, "not-example.com".to_string())
            .expect_err("host is denied");

        assert_eq!(err, "dns-resolve is scoped to example.com");
    }

    #[test]
    fn report_host_import_rejects_path_traversal() {
        let err = safe_report_path(Path::new("reports"), "../escape.txt")
            .expect_err("path traversal rejected");

        assert_eq!(
            err,
            "report filename cannot contain path separators or traversal"
        );
    }

    #[test]
    fn report_host_import_writes_under_reports_dir() {
        let dir = std::env::temp_dir().join(format!("polyglid-report-test-{}", std::process::id()));
        let mut state = RuntimeState::new("example.com", dir.clone());

        let path = polyglid::engine::reports::Host::write(
            &mut state,
            "demo.txt".to_string(),
            "report body".to_string(),
        )
        .expect("report write");

        assert_eq!(
            std::fs::read_to_string(&path).expect("report body"),
            "report body"
        );

        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_dir(dir);
    }
}
