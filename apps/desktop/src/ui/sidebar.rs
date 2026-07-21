use dioxus::prelude::*;

use crate::backend::DesktopBackend;

use super::commands::{execute, ShellCommand};
use super::components::SidebarOption;
use super::models::{EditorTab, PendingPluginInfo, TrackFilter, WorkspaceView};
use super::state::AppState;

#[component]
pub(crate) fn WorkspaceSidebar() -> Element {
    let state = use_context::<AppState>();
    let backend = use_context::<DesktopBackend>();
    let active_view = *state.active_view.read();
    rsx! {
        aside { class: "sidebar", style: "width: {state.sidebar_width}px; flex-basis: {state.sidebar_width}px",
            div { class: "sidebar-heading", span { "{active_view.title()}" }
                button { title: "Hide side bar (Ctrl+B)", aria_label: "Hide side bar", onclick: move |_| execute(state, ShellCommand::ToggleSidebar, backend.clone()), "‹" }
            }
            match active_view {
                WorkspaceView::Projects => rsx! { ProjectsSidebar {} },
                WorkspaceView::Explorer => rsx! { ExplorerSidebar {} },
                WorkspaceView::Plugins => rsx! { PluginsSidebar {} },
                WorkspaceView::Tracks => rsx! { TracksSidebar {} },
                WorkspaceView::Automation => rsx! { AutomationSidebar {} },
                WorkspaceView::Agents => rsx! { AgentsSidebar {} },
            }
        }
    }
}

#[component]
fn ProjectsSidebar() -> Element {
    let mut state = use_context::<AppState>();
    rsx! {
        div { class: "sidebar-section",
            p { class: "section-label", "Workspace" }
            div { class: "workspace-summary",
                span { class: "live-dot" }
                div { strong { "{state.active_workspace}" } small { "local catalog" } }
            }
        }
        div { class: "sidebar-section grow",
            p { class: "section-label", "Projects · {state.projects.read().len()}" }
            for project in state.projects.read().iter() {
                div { class: "project-nav", span { "◇" } div { strong { "{project.name}" } small { "{project.kind}" } } }
            }
            button { class: "sidebar-option", onclick: move |_| {
                let next = *state.workspace_refresh.read() + 1;
                state.workspace_refresh.set(next);
            }, span { "Refresh discovery" } small { "↻" } }
        }
    }
}

#[component]
fn ExplorerSidebar() -> Element {
    let mut state = use_context::<AppState>();
    rsx! {
        div { class: "sidebar-section",
            p { class: "section-label", "Targets" }
            div { class: "add-row",
                input { value: "{state.new_target}", placeholder: "Add domain or IP", oninput: move |event| state.new_target.set(event.value()) }
                button { onclick: move |_| {
                    let candidate = state.new_target.read().trim().to_string();
                    if !candidate.is_empty() && !state.targets.read().contains(&candidate) {
                        state.targets.write().push(candidate.clone());
                        state.selected_target.set(candidate);
                        state.new_target.set(String::new());
                    }
                }, "+" }
            }
            div { class: "target-list",
                for target in state.targets.read().iter() {
                    button { class: if *state.selected_target.read() == *target { "target active" } else { "target" }, onclick: { let target = target.clone(); move |_| { state.selected_target.set(target.clone()); state.editor_tab.set(EditorTab::Scanner); } }, span { "◎" } span { "{target}" } }
                }
            }
        }
        div { class: "sidebar-section grow",
            p { class: "section-label", "Active plugins" }
            for plugin in state.plugins.read().iter() {
                div { class: if plugin.enabled { "mini-plugin" } else { "mini-plugin disabled" }, span { class: "live-dot" } div { strong { "{plugin.name}" } small { "{plugin.id}" } } }
            }
        }
    }
}

#[component]
fn PluginsSidebar() -> Element {
    let mut state = use_context::<AppState>();
    let backend = use_context::<DesktopBackend>();
    let be_keydown = backend.clone();
    let be_install = backend.clone();
    rsx! {
        div { class: "sidebar-section", p { class: "section-label", "Install component" }
            div { class: "add-row",
                input { value: "{state.plugin_install_path}", placeholder: "/path/to/plugin.wasm",
                    oninput: move |event| state.plugin_install_path.set(event.value()),
                    onkeydown: move |event| {
                        if event.key().to_string() == "Enter" {
                            let path = state.plugin_install_path.read().clone();
                            if !path.is_empty() {
                                start_plugin_install(state, be_keydown.clone(), path);
                            }
                        }
                    }
                }
                button { onclick: move |_| {
                    let path = rfd::FileDialog::new()
                        .add_filter("WebAssembly Component", &["wasm"])
                        .set_title("Select a WASM plugin component")
                        .pick_file();
                    if let Some(picked) = path {
                        state.plugin_install_path.set(picked.display().to_string());
                    }
                }, "Browse" }
            }
            button { class: "primary small", onclick: move |_| {
                let path = state.plugin_install_path.read().clone();
                if !path.is_empty() {
                    start_plugin_install(state, be_install.clone(), path);
                }
            }, "+ Install plugin" }
        }
        div { class: "sidebar-section grow", p { class: "section-label", "Registry" }
            for plugin in state.plugins.read().iter() {
                button { class: if state.selected_plugin.read().as_str() == plugin.id { "plugin-nav active" } else { "plugin-nav" }, onclick: { let id = plugin.id.to_string(); move |_| state.selected_plugin.set(id.clone()) }, span { class: if plugin.enabled { "live-dot" } else { "live-dot off" } } div { strong { "{plugin.name}" } small { "v{plugin.version}" } } }
            }
        }
    }
}

fn start_plugin_install(mut state: AppState, backend: DesktopBackend, path: String) {
    state.install_error.set(None);
    let backend = backend.clone();
    let path_clone = path.clone();
    spawn(async move {
        let result = tokio::task::spawn_blocking(move || backend.validate_plugin(&path_clone))
            .await
            .map_err(|error| format!("validation task failed: {error}"))
            .and_then(|result| result);
        match result {
            Ok((manifest, metadata)) => {
                state.pending_install.set(Some(PendingPluginInfo {
                    path,
                    name: metadata.display_name,
                    id: manifest.id.as_str().to_string(),
                    version: metadata.version,
                    author: metadata.author,
                    description: metadata.description,
                    capabilities: manifest
                        .requested_capabilities
                        .into_iter()
                        .map(|req| req.capability.to_string())
                        .collect(),
                }));
            }
            Err(error) => state.install_error.set(Some(error)),
        }
    });
}

#[component]
fn TracksSidebar() -> Element {
    let mut state = use_context::<AppState>();
    rsx! { div { class: "sidebar-section grow", p { class: "section-label", "Delivery filters" }
        for filter in TrackFilter::ALL {
            button { class: if *state.track_filter.read() == filter { "sidebar-option active" } else { "sidebar-option" }, onclick: move |_| state.track_filter.set(filter), span { "{filter.label()}" } small { "{track_count(&state, filter)}" } }
        }
    } }
}

fn track_count(state: &AppState, filter: TrackFilter) -> usize {
    state
        .overview
        .read()
        .tracks
        .iter()
        .filter(|track| filter.matches(track.status))
        .count()
}

#[component]
fn AutomationSidebar() -> Element {
    rsx! { div { class: "sidebar-section grow", p { class: "section-label", "Pipelines" } SidebarOption { label: "Workspace verify", meta: "ready", active: true } SidebarOption { label: "Rust quality", meta: "4 steps", active: false } SidebarOption { label: "Security review", meta: "3 steps", active: false } SidebarOption { label: "Release gate", meta: "draft", active: false } } }
}

#[component]
fn AgentsSidebar() -> Element {
    rsx! { div { class: "sidebar-section grow", p { class: "section-label", "Agent roster" } SidebarOption { label: "Executive", meta: "online", active: true } SidebarOption { label: "Code analyst", meta: "idle", active: false } SidebarOption { label: "Security reviewer", meta: "idle", active: false } SidebarOption { label: "Test helper", meta: "idle", active: false } } }
}
