use std::path::PathBuf;
use std::time::{Duration, Instant};

use polyglid_core::execution::{
    ExecutionConfig, ExecutionEvent, ExecutionManager, JobState,
};
use polyglid_core::{PluginRef, PluginRuntime};
use polyglid_runtime::WasmRuntime;
use serde::Serialize;

#[derive(Serialize)]
struct SerializablePluginMetadata {
    name: String,
    display_name: String,
    version: String,
    description: String,
    author: String,
    required_capabilities: Vec<String>,
}

#[derive(Serialize)]
struct SerializableReport {
    plugin_name: String,
    target_tested: String,
    issues: Vec<SerializableIssue>,
    summary: String,
    panel: Option<SerializablePanelLayout>,
}

#[derive(Serialize)]
struct SerializableIssue {
    title: String,
    severity: String,
    description: String,
    recommendation: String,
}

#[derive(Serialize)]
struct SerializablePanelLayout {
    title: String,
    widgets: Vec<SerializablePanelWidget>,
}

#[derive(Serialize)]
struct SerializablePanelWidget {
    widget_kind: String,
    title: String,
    data: Vec<Vec<String>>,
}

fn map_panel_layout(layout: polyglid_plugin_api::PanelLayout) -> SerializablePanelLayout {
    SerializablePanelLayout {
        title: layout.title,
        widgets: layout
            .widgets
            .into_iter()
            .map(|w| SerializablePanelWidget {
                widget_kind: match w.widget_kind {
                    polyglid_plugin_api::WidgetKind::Table => "Table",
                    polyglid_plugin_api::WidgetKind::KeyValue => "KeyValue",
                    polyglid_plugin_api::WidgetKind::Tree => "Tree",
                    polyglid_plugin_api::WidgetKind::Log => "Log",
                    polyglid_plugin_api::WidgetKind::ChartBar => "ChartBar",
                    polyglid_plugin_api::WidgetKind::TextBlock => "TextBlock",
                }
                .to_string(),
                title: w.title,
                data: w.data,
            })
            .collect(),
    }
}

#[tauri::command]
fn inspect_plugin_wasm(plugin_path: String) -> Result<SerializablePluginMetadata, String> {
    let runtime = WasmRuntime::new();
    let plugin_ref = PluginRef::from_path(PathBuf::from(&plugin_path));

    let manifest = runtime.inspect(&plugin_ref).map_err(|e| e.to_string())?;
    let metadata = runtime
        .call_metadata(&plugin_ref)
        .map_err(|e| e.to_string())?;

    Ok(SerializablePluginMetadata {
        name: metadata.name,
        display_name: metadata.display_name,
        version: metadata.version,
        description: metadata.description,
        author: metadata.author,
        required_capabilities: manifest
            .requested_capabilities
            .iter()
            .map(|c| c.to_string())
            .collect(),
    })
}

#[tauri::command]
fn cancel_scan_job(
    state: tauri::State<'_, ExecutionManager<WasmRuntime>>,
    job_id: String,
) -> Result<(), String> {
    let uuid = uuid::Uuid::parse_str(&job_id).map_err(|e| e.to_string())?;
    state.cancel_job(uuid).map_err(|e| e.to_string())
}

#[tauri::command]
fn run_plugin(
    state: tauri::State<'_, ExecutionManager<WasmRuntime>>,
    plugin_path: String,
    target: String,
) -> Result<SerializableReport, String> {
    let runtime = WasmRuntime::new();
    let plugin_ref = PluginRef::from_path(PathBuf::from(&plugin_path));

    let manifest = runtime.inspect(&plugin_ref).map_err(|e| e.to_string())?;
    let allowed_caps = manifest
        .requested_capabilities
        .iter()
        .map(|c| c.capability)
        .collect();

    let config = ExecutionConfig {
        fuel_limit: 25_000_000,
        timeout: Duration::from_secs(30),
        memory_limit: None,
        allowed_capabilities: allowed_caps,
    };

    let mut rx = state.subscribe();
    let job_id = state.submit_job(plugin_path, target, config);

    // Wait for the job to complete in the background execution pipeline
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(35) {
        if let Ok(event) = rx.blocking_recv() {
            match event {
                ExecutionEvent::JobFinished {
                    job_id: id,
                    report,
                    ..
                } if id == job_id => {
                    let jobs = state.get_jobs();
                    let job = jobs.iter().find(|j| j.id == job_id).unwrap();

                    return Ok(SerializableReport {
                        plugin_name: report.plugin_name,
                        target_tested: report.target_tested,
                        issues: report
                            .issues
                            .into_iter()
                            .map(|i| SerializableIssue {
                                title: i.title,
                                severity: i.severity.to_string(),
                                description: i.description,
                                recommendation: i.recommendation,
                            })
                            .collect(),
                        summary: report.summary,
                        panel: job.report.as_ref().and_then(|r| {
                            runtime
                                .call_desktop_panel(&plugin_ref, r)
                                .ok()
                                .map(map_panel_layout)
                        }),
                    });
                }
                ExecutionEvent::JobFailed {
                    job_id: id,
                    error,
                    ..
                } if id == job_id => {
                    return Err(format!("Scan execution failed: {error}"));
                }
                ExecutionEvent::JobStateChanged {
                    job_id: id,
                    state: JobState::TimedOut,
                } if id == job_id => {
                    return Err("Scan execution timed out".to_string());
                }
                ExecutionEvent::JobStateChanged {
                    job_id: id,
                    state: JobState::Cancelled,
                } if id == job_id => {
                    return Err("Scan execution cancelled by user".to_string());
                }
                _ => {}
            }
        }
    }

    Err("Scan execution timed out".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ExecutionManager::new(WasmRuntime::new()))
        .invoke_handler(tauri::generate_handler![
            run_plugin,
            inspect_plugin_wasm,
            cancel_scan_job
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
