use polyglid_config::AppConfig;
use polyglid_core::{CoreEngine, InMemoryPermissionStore, PluginRef, PluginRunRequest, Target};
use polyglid_events::VecEventSink;
use polyglid_plugin_api::Capability;
use polyglid_runtime::WasmRuntime;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
struct SerializableReport {
    plugin_name: String,
    target_tested: String,
    issues: Vec<SerializableIssue>,
    summary: String,
}

#[derive(Serialize)]
struct SerializableIssue {
    title: String,
    severity: String,
    description: String,
    recommendation: String,
}

#[tauri::command]
fn run_plugin(plugin_path: String, target: String) -> Result<SerializableReport, String> {
    let mut permissions = InMemoryPermissionStore::default();
    permissions.grant_for_all(Capability::DnsResolve);
    permissions.grant_for_all(Capability::ReportWrite);
    
    let config = AppConfig::development();

    let mut engine = CoreEngine::new(
        WasmRuntime::new(),
        permissions,
        VecEventSink::default(),
        config,
    ).map_err(|e| e.to_string())?;

    let path = PathBuf::from(plugin_path);

    let report = engine
        .run_plugin(PluginRunRequest {
            plugin: PluginRef::from_path(path),
            target: Target::parse(target).map_err(|e| e.to_string())?,
        })
        .map_err(|e| e.to_string())?;

    let serializable_report = SerializableReport {
        plugin_name: report.plugin_name,
        target_tested: report.target_tested,
        issues: report.issues.into_iter().map(|i| SerializableIssue {
            title: i.title,
            severity: i.severity.to_string(),
            description: i.description,
            recommendation: i.recommendation,
        }).collect(),
        summary: report.summary,
    };

    Ok(serializable_report)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![run_plugin])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
