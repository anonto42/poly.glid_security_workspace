use dioxus::prelude::*;

use super::components::SidebarOption;
use super::models::{EditorTab, TrackFilter, WorkspaceView};
use super::state::AppState;

#[component]
pub(crate) fn WorkspaceSidebar() -> Element {
    let state = use_context::<AppState>();
    let active_view = *state.active_view.read();
    rsx! {
        aside { class: "sidebar",
            div { class: "sidebar-heading", span { "{active_view.title()}" } button { "•••" } }
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
    rsx! {
        div { class: "sidebar-section", p { class: "section-label", "Install component" } input { placeholder: "/path/to/plugin.wasm" } button { class: "primary small", "+ Install plugin" } }
        div { class: "sidebar-section grow", p { class: "section-label", "Registry" }
            for plugin in state.plugins.read().iter() {
                button { class: if state.selected_plugin.read().as_str() == plugin.id { "plugin-nav active" } else { "plugin-nav" }, onclick: { let id = plugin.id.to_string(); move |_| state.selected_plugin.set(id.clone()) }, span { class: if plugin.enabled { "live-dot" } else { "live-dot off" } } div { strong { "{plugin.name}" } small { "v{plugin.version}" } } }
            }
        }
    }
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
