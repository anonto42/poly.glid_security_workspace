use dioxus::prelude::*;
use polyglid_desktop::client::{ClientGateway, ExecutionState, JobId, LocalClient, PluginStatus};

use super::features::{
    ExecutionsDashboard, PluginDashboard, ProjectsDashboard, ReportsDashboard, ScannerDashboard,
};
use super::models::{DialogError, Overlay, PermissionReview, WorkspaceView};
use super::state::{activate_view, close_view, refresh_operational_data, AppState};

#[component]
pub(crate) fn EditorWorkspace() -> Element {
    let state = use_context::<AppState>();
    let active_view = *state.shell.active_view.read();
    rsx! {
        main { class: "editor",
            WorkspaceEditorTabs {}
            div { class: "editor-surface",
                match active_view {
                    WorkspaceView::Projects => rsx! { ProjectsDashboard {} },
                    WorkspaceView::Scanner => rsx! { ScannerEditor {} },
                    WorkspaceView::Executions => rsx! { ExecutionsEditor {} },
                    WorkspaceView::Reports => rsx! { ReportsDashboard {} },
                    WorkspaceView::Plugins => rsx! { PluginsEditor {} },
                }
            }
        }
    }
}

#[component]
fn WorkspaceEditorTabs() -> Element {
    let state = use_context::<AppState>();
    let active = *state.shell.active_view.read();
    let views = state.shell.open_views.read().clone();
    rsx! {
        div { class: "workbench-tabs", role: "tablist", aria_label: "Open views",
            for view in views {
                button {
                    class: if view == active { "workbench-tab active" } else { "workbench-tab" },
                    role: "tab",
                    aria_selected: view == active,
                    onclick: move |_| activate_view(state, view),
                    span { class: "tab-icon", "{view.icon()}" }
                    span { "{view.title()}" }
                    span {
                        class: "tab-close",
                        title: "Close view",
                        onclick: move |event| { event.stop_propagation(); close_view(state, view); },
                        "×"
                    }
                }
            }
        }
    }
}

#[component]
fn ScannerEditor() -> Element {
    let mut state = use_context::<AppState>();
    let plugins = state.plugins.items.read().clone();
    let selected_id = state.plugins.selected_id.read().clone();
    let target = state.runs.selected_target.read().clone();
    let running = state
        .runs
        .active_job_id
        .read()
        .and_then(|id| {
            state
                .runs
                .executions
                .read()
                .iter()
                .find(|run| run.id == id)
                .map(|run| !run.state.is_terminal())
        })
        .unwrap_or(false);

    rsx! {
        ScannerDashboard {
            target,
            selected_plugin: selected_id,
            plugins,
            running,
            on_target: move |value| state.runs.selected_target.set(value),
            on_plugin: move |value| state.plugins.selected_id.set(Some(value)),
            on_review: move |_| {
                let Some(plugin_id) = state.plugins.selected_id.read().clone() else { return; };
                let Some(plugin) = state.plugins.items.read().iter().find(|item| item.id == plugin_id).cloned() else { return; };
                state.shell.overlay.set(Some(Overlay::PermissionReview(PermissionReview {
                    plugin_id: plugin.id,
                    plugin_name: plugin.name,
                    target: state.runs.selected_target.read().trim().to_string(),
                    requested: plugin.requested_capabilities,
                    approved: Vec::new(),
                })));
            }
        }
    }
}

#[component]
fn ExecutionsEditor() -> Element {
    let mut state = use_context::<AppState>();
    let client = use_context::<LocalClient>();
    let cancel_client = client.clone();
    rsx! {
        ExecutionsDashboard {
            executions: state.runs.executions.read().clone(),
            active_job_id: *state.runs.active_job_id.read(),
            on_cancel: move |job_id: JobId| {
                let client = cancel_client.clone();
                spawn(async move {
                    let result = tokio::task::spawn_blocking(move || client.cancel_execution(job_id)).await;
                    match result {
                        Ok(Ok(())) => {
                            state.runs.activity.write().push(format!("Cancelled execution {job_id}"));
                            refresh_operational_data(state);
                        }
                        Ok(Err(error)) => show_error(state, "Cancellation failed", error.to_string()),
                        Err(error) => show_error(state, "Cancellation failed", format!("execution task failed: {error}")),
                    }
                });
            },
            on_open_report: move |job_id: JobId| {
                if let Some(report) = state.runs.reports.read().iter().find(|report| report.job_id == job_id) {
                    state.runs.selected_report_id.set(Some(report.id.clone()));
                    activate_view(state, WorkspaceView::Reports);
                } else {
                    state.runs.error.set(Some("The execution completed, but its persisted report is not available yet. Refresh and try again.".to_string()));
                }
            },
            on_refresh: move |_| refresh_operational_data(state),
        }
    }
}

#[component]
fn PluginsEditor() -> Element {
    let mut state = use_context::<AppState>();
    let client = use_context::<LocalClient>();
    let toggle_client = client.clone();
    let uninstall_client = client.clone();
    rsx! {
        PluginDashboard {
            plugins: state.plugins.items.read().clone(),
            selected: state.plugins.selected_id.read().clone(),
            on_select: move |id: String| state.plugins.selected_id.set(Some(id)),
            on_toggle: move |id: String| {
                let enabled = state
                    .plugins
                    .items
                    .read()
                    .iter()
                    .find(|plugin| plugin.id == id)
                    .is_some_and(|plugin| plugin.status != PluginStatus::Enabled);
                let client = toggle_client.clone();
                spawn(async move {
                    let toggle_id = id.clone();
                    let result = tokio::task::spawn_blocking(move || client.set_plugin_enabled(&toggle_id, enabled)).await;
                    match result {
                        Ok(Ok(())) => {
                            if let Some(plugin) = state.plugins.items.write().iter_mut().find(|plugin| plugin.id == id) {
                                plugin.status = if enabled { PluginStatus::Enabled } else { PluginStatus::Disabled };
                            }
                        }
                        Ok(Err(error)) => show_error(state, "Plugin update failed", error.to_string()),
                        Err(error) => show_error(state, "Plugin update failed", format!("plugin task failed: {error}")),
                    }
                });
            },
            on_uninstall: move |id: String| {
                let client = uninstall_client.clone();
                let removed_id = id.clone();
                spawn(async move {
                    let operation_id = removed_id.clone();
                    let result = tokio::task::spawn_blocking(move || client.uninstall_plugin(&operation_id)).await;
                    match result {
                        Ok(Ok(())) => {
                            state.plugins.items.write().retain(|plugin| plugin.id != removed_id);
                            if state.plugins.selected_id.read().as_ref() == Some(&removed_id) {
                                let next = state.plugins.items.read().first().map(|plugin| plugin.id.clone());
                                state.plugins.selected_id.set(next);
                            }
                        }
                        Ok(Err(error)) => show_error(state, "Uninstall failed", error.to_string()),
                        Err(error) => show_error(state, "Uninstall failed", format!("plugin task failed: {error}")),
                    }
                });
            }
        }
    }
}

fn show_error(mut state: AppState, title: impl Into<String>, message: impl Into<String>) {
    state.shell.overlay.set(Some(Overlay::Error(DialogError {
        title: title.into(),
        message: message.into(),
    })));
}

#[allow(dead_code)]
fn _is_running(state: ExecutionState) -> bool {
    !state.is_terminal()
}
