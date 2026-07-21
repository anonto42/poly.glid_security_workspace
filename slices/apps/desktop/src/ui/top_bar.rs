use dioxus::prelude::*;

use crate::backend::DesktopBackend;

use super::models::WorkspaceLoadState;
use super::state::{activate_view, AppState};

#[component]
pub(crate) fn TitleBar() -> Element {
    rsx! {
        header { class: "titlebar",
            BrandArea {}
            CommandCenter {}
            div { class: "topbar-actions", PluginActionSlot {} SystemStatus {} UserActions {} }
        }
    }
}

#[component]
fn BrandArea() -> Element {
    let mut state = use_context::<AppState>();
    let backend = use_context::<DesktopBackend>();
    let mut menu_open = use_signal(|| false);
    let workspaces = state.workspaces.read().clone();
    rsx! {
        div { class: "brand-area",
            div { class: "wordmark", span { class: "wordmark-icon", "P" } div { strong { "polyglid" } small { "developer space" } } }
            div { class: "workspace-picker",
                button { class: "workspace-trigger", aria_label: "Choose workspace", onclick: move |_| menu_open.toggle(), span { class: "workspace-pulse" } span { "{state.active_workspace}" } span { class: "chevron", "⌄" } }
                if *menu_open.read() {
                    div { class: "topbar-menu workspace-menu",
                        p { "Workspace" }
                        for workspace in workspaces {
                            button {
                                class: if workspace.is_active { "selected" } else { "" },
                                onclick: {
                                    let backend = backend.clone();
                                    let workspace_id = workspace.id.clone();
                                    move |_| {
                                        menu_open.set(false);
                                        let backend = backend.clone();
                                        let workspace_id = workspace_id.clone();
                                        spawn(async move {
                                            let result = tokio::task::spawn_blocking(move || backend.activate(&workspace_id)).await;
                                            match result {
                                                Ok(Ok(())) => {
                                                    let next = *state.workspace_refresh.read() + 1;
                                                    state.workspace_refresh.set(next);
                                                }
                                                Ok(Err(error)) => state.workspace_mutation_error.set(Some(error)),
                                                Err(error) => state.workspace_mutation_error.set(Some(format!("workspace task failed: {error}"))),
                                            }
                                        });
                                    }
                                },
                                span { if workspace.is_active { "◈" } else { "◇" } }
                                div { strong { "{workspace.name}" } small { if workspace.is_active { "Local · current" } else { "Local workspace" } } }
                            }
                        }
                        button { class: "menu-command", onclick: move |_| { state.command_open.set(true); menu_open.set(false); }, "+ Workspace commands" }
                    }
                }
            }
        }
    }
}

#[component]
fn CommandCenter() -> Element {
    let mut state = use_context::<AppState>();
    rsx! {
        button { class: "command-trigger", onclick: move |_| state.command_open.set(true),
            span { class: "search-icon", "⌕" }
            span { class: "command-copy", "Search workspace, actions, or plugins" }
            span { class: "command-mode", "COMMAND" }
            kbd { "Ctrl Shift P" }
        }
    }
}

#[component]
fn PluginActionSlot() -> Element {
    let mut state = use_context::<AppState>();
    rsx! {
        div { class: "plugin-action-slot", aria_label: "Plugin actions",
            span { class: "slot-label", "extensions" }
            for action in state.top_bar_actions.read().iter() {
                button { id: "{action.id}", class: "topbar-icon plugin-action", title: "{action.source}: {action.label}", aria_label: "{action.label}", onclick: { let destination = action.destination; move |_| activate_view(state, destination) }, span { "{action.icon}" } }
            }
            button { class: "topbar-icon", title: "More extension actions", aria_label: "More extension actions", onclick: move |_| state.command_open.set(true), "••" }
        }
    }
}

#[component]
fn SystemStatus() -> Element {
    let mut state = use_context::<AppState>();
    let status = match &*state.workspace_load.read() {
        WorkspaceLoadState::Loading => ("INDEXING", "workspace loading", false),
        WorkspaceLoadState::Error(_) => ("ERROR", "workspace unavailable", false),
        WorkspaceLoadState::Empty => ("LOCAL", "workspace empty", true),
        WorkspaceLoadState::Ready => ("LOCAL", "workspace ready", true),
    };
    rsx! {
        button { class: "system-status", title: "Open runtime settings", onclick: move |_| state.settings_open.set(true),
            span { class: "status-orbit", span { class: if status.2 { "live-dot" } else { "live-dot off" } } }
            span { class: "status-copy", strong { "{status.0}" } small { "{status.1}" } }
        }
    }
}

#[component]
fn UserActions() -> Element {
    let mut state = use_context::<AppState>();
    let mut notifications_open = use_signal(|| false);
    let workspace_error = match &*state.workspace_load.read() {
        WorkspaceLoadState::Error(error) => Some(error.clone()),
        _ => state.workspace_mutation_error.read().clone(),
    };
    rsx! {
        div { class: "user-actions",
            div { class: "notification-wrap",
                button { class: "topbar-icon notification-button", title: "Notifications", aria_label: "Notifications", onclick: move |_| notifications_open.toggle(), "○"
                    if workspace_error.is_some() { span { class: "notification-badge", "1" } }
                }
                if *notifications_open.read() {
                    div { class: "topbar-menu notification-menu",
                        div { class: "menu-heading", strong { "Activity" } span { if workspace_error.is_some() { "1 issue" } else { "current" } } }
                        if let Some(error) = workspace_error {
                            div { class: "notification-item", span { class: "event-dot" } div { strong { "Workspace action failed" } small { "{error}" } } }
                        } else {
                            div { class: "notification-item", span { class: "event-dot good" } div { strong { "Workspace ready" } small { "{state.projects.read().len()} projects indexed locally." } } }
                        }
                    }
                }
            }
            button { class: "avatar", title: "Open settings", aria_label: "Open settings", onclick: move |_| state.settings_open.set(true), span { "S" } }
        }
    }
}
