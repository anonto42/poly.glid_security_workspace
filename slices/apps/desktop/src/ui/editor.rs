use dioxus::prelude::*;

use super::components::EditorTabButton;
use super::features::{
    AgentsDashboard, AutomationDashboard, PluginDashboard, ProjectsDashboard, ResultDashboard,
    ScannerDashboard, SourceDashboard, TracksDashboard,
};
use super::models::{BottomTab, EditorTab, WorkspaceView};
use super::state::{activate_view, close_view, AppState};
use crate::backend::DesktopBackend;

#[component]
pub(crate) fn EditorWorkspace() -> Element {
    let state = use_context::<AppState>();
    let active_view = *state.active_view.read();
    rsx! {
        main { class: "editor",
            WorkspaceEditorTabs {}
            div { class: "editor-surface",
                match active_view {
                    WorkspaceView::Projects => rsx! { ProjectsDashboard {} },
                    WorkspaceView::Explorer => rsx! { ExplorerEditor {} },
                    WorkspaceView::Plugins => rsx! { PluginsEditor {} },
                    WorkspaceView::Tracks => rsx! { TracksEditor {} },
                    WorkspaceView::Automation => rsx! { AutomationDashboard {} },
                    WorkspaceView::Agents => rsx! { AgentsDashboard {} },
                }
            }
        }
    }
}

#[component]
fn WorkspaceEditorTabs() -> Element {
    let state = use_context::<AppState>();
    let active = *state.active_view.read();
    let views = state.open_views.read().clone();
    rsx! {
        div { class: "workbench-tabs", role: "tablist", aria_label: "Open editors",
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
                        title: "Close editor",
                        onclick: move |event| { event.stop_propagation(); close_view(state, view); },
                        "×"
                    }
                }
            }
        }
    }
}

#[component]
fn ExplorerEditor() -> Element {
    let mut state = use_context::<AppState>();
    let backend = use_context::<DesktopBackend>();
    let report = state.report.read().clone();
    rsx! {
        div { class: "editor-tabs",
            EditorTabButton { label: "Scanner configuration", icon: "⚡", active: *state.editor_tab.read() == EditorTab::Scanner, onclick: move |_| state.editor_tab.set(EditorTab::Scanner) }
            if report.is_some() { EditorTabButton { label: "Result dashboard", icon: "▥", active: *state.editor_tab.read() == EditorTab::Result, onclick: move |_| state.editor_tab.set(EditorTab::Result) } }
            EditorTabButton { label: "recon_probe.rs", icon: "Rs", active: *state.editor_tab.read() == EditorTab::Source, onclick: move |_| state.editor_tab.set(EditorTab::Source) }
        }
        match *state.editor_tab.read() {
            EditorTab::Scanner => rsx! { ScannerDashboard {
                target: state.selected_target.read().clone(),
                selected_plugin: state.selected_plugin.read().clone(),
                plugins: state.plugins.read().clone(),
                on_target: move |value| state.selected_target.set(value),
                on_plugin: move |value| state.selected_plugin.set(value),
                on_run: move |_| {
                    let backend = backend.clone();
                    let plugin = state.selected_plugin.read().clone();
                    let target = state.selected_target.read().clone();
                    let fuel = *state.fuel_limit.read();
                    state.execution_running.set(true);
                    state.execution_error.set(None);
                    spawn(async move {
                        let result = tokio::task::spawn_blocking(move || backend.run_plugin(&plugin, &target, fuel)).await
                            .map_err(|error| format!("execution task failed: {error}"))
                            .and_then(|result| result);
                        state.execution_running.set(false);
                        match result {
                            Ok(report) => {
                                state.report.set(Some(super::models::ScanReport {
                                    target: report.target_tested,
                                    summary: report.summary,
                                    findings: report.issues.into_iter().map(|issue| super::models::Finding {
                                        severity: issue.severity.to_string().to_uppercase(),
                                        title: issue.title,
                                        description: issue.description,
                                        recommendation: issue.recommendation,
                                    }).collect(),
                                }));
                                state.editor_tab.set(EditorTab::Result);
                                state.bottom_tab.set(BottomTab::Problems);
                            }
                            Err(error) => state.execution_error.set(Some(error)),
                        }
                    });
                }
            } },
            EditorTab::Result => rsx! { ResultDashboard { report } },
            EditorTab::Source => rsx! { SourceDashboard {} },
        }
    }
}

#[component]
fn PluginsEditor() -> Element {
    let mut state = use_context::<AppState>();
    let backend = use_context::<DesktopBackend>();
    let backend_toggle = backend.clone();
    let backend_uninstall = backend.clone();
    rsx! { PluginDashboard {
        plugins: state.plugins.read().clone(),
        selected: state.selected_plugin.read().clone(),
        on_toggle: move |id: String| {
            let enabled = state.plugins.read().iter().find(|plugin| plugin.id == id).map(|plugin| !plugin.enabled).unwrap_or(false);
            let backend = backend_toggle.clone();
            spawn(async move {
                let toggle_id = id.clone();
                let result = tokio::task::spawn_blocking(move || backend.toggle_plugin(&toggle_id, enabled)).await
                    .map_err(|error| format!("plugin task failed: {error}"))
                    .and_then(|result| result);
                match result {
                    Ok(()) => if let Some(plugin) = state.plugins.write().iter_mut().find(|plugin| plugin.id == id) { plugin.enabled = enabled; },
                    Err(error) => state.execution_error.set(Some(error)),
                }
            });
        },
        on_uninstall: move |id: String| {
            let backend = backend_uninstall.clone();
            let uninstall_id = id.clone();
            let uninstall_id_spawn = uninstall_id.clone();
            spawn(async move {
                let result = tokio::task::spawn_blocking(move || {
                    backend.uninstall_plugin(&uninstall_id_spawn)
                }).await
                    .map_err(|error| format!("uninstall task failed: {error}"))
                    .and_then(|result| result);
                match result {
                    Ok(()) => {
                        state.plugins.write().retain(|plugin| plugin.id != uninstall_id);
                        if *state.selected_plugin.read() == uninstall_id {
                            if let Some(first) = state.plugins.read().first().cloned() {
                                state.selected_plugin.set(first.id);
                            }
                        }
                    }
                    Err(error) => state.execution_error.set(Some(error)),
                }
            });
        }
    } }
}

#[component]
fn TracksEditor() -> Element {
    let mut state = use_context::<AppState>();
    rsx! { TracksDashboard {
        overview: state.overview.read().clone(),
        filter: *state.track_filter.read(),
        selected: *state.selected_track.read(),
        on_select: move |index| state.selected_track.set(index),
    } }
}
