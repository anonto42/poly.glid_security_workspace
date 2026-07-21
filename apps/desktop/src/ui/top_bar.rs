use dioxus::prelude::*;
use polyglid_desktop::client::{ClientGateway, LocalClient};

use super::models::{LoadState, Overlay, WorkspaceView};
use super::state::{activate_view, AppState};

#[component]
pub(crate) fn TitleBar() -> Element {
    rsx! {
        header { class: "titlebar",
            BrandArea {}
            CommandCenter {}
            div { class: "topbar-actions", ProductActions {} SystemStatus {} UserActions {} }
        }
    }
}

#[component]
fn BrandArea() -> Element {
    let mut state = use_context::<AppState>();
    let client = use_context::<LocalClient>();
    let mut menu_open = use_signal(|| false);
    let workspaces = state.catalog.workspaces.read().clone();
    rsx! {
        div { class: "brand-area",
            div { class: "wordmark", span { class: "wordmark-icon", "P" } div { strong { "polyglid" } small { "security workspace" } } }
            div { class: "workspace-picker",
                button { class: "workspace-trigger", aria_label: "Choose workspace", onclick: move |_| menu_open.toggle(), span { class: "workspace-pulse" } span { "{state.catalog.active_workspace_name}" } span { class: "chevron", "⌄" } }
                if *menu_open.read() {
                    div { class: "topbar-menu workspace-menu",
                        p { "Workspace" }
                        for workspace in workspaces {
                            button {
                                class: if workspace.is_active { "selected" } else { "" },
                                onclick: {
                                    let client = client.clone();
                                    let workspace_id = workspace.id.clone();
                                    move |_| {
                                        menu_open.set(false);
                                        let client = client.clone();
                                        let workspace_id = workspace_id.clone();
                                        spawn(async move {
                                            let result = tokio::task::spawn_blocking(move || client.activate_workspace(&workspace_id)).await;
                                            match result {
                                                Ok(Ok(())) => {
                                                    let next = *state.catalog.refresh.read() + 1;
                                                    state.catalog.refresh.set(next);
                                                }
                                                Ok(Err(error)) => state.catalog.error.set(Some(error.to_string())),
                                                Err(error) => state.catalog.error.set(Some(format!("workspace task failed: {error}"))),
                                            }
                                        });
                                    }
                                },
                                span { if workspace.is_active { "◈" } else { "◇" } }
                                div { strong { "{workspace.name}" } small { "{workspace.root_path}" } }
                            }
                        }
                        button { class: "menu-command", onclick: move |_| { state.shell.overlay.set(Some(Overlay::Commands)); menu_open.set(false); }, "Workspace commands" }
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
        button { class: "command-trigger", onclick: move |_| state.shell.overlay.set(Some(Overlay::Commands)),
            span { class: "search-icon", "⌕" }
            span { class: "command-copy", "Search views and actions" }
            span { class: "command-mode", "COMMAND" }
            kbd { "Ctrl P" }
        }
    }
}

#[component]
fn ProductActions() -> Element {
    let state = use_context::<AppState>();
    rsx! {
        div { class: "plugin-action-slot", aria_label: "Primary actions",
            button { class: "secondary small", onclick: move |_| activate_view(state, WorkspaceView::Scanner), "New scan" }
            button { class: "topbar-icon", title: "Open reports", aria_label: "Open reports", onclick: move |_| activate_view(state, WorkspaceView::Reports), "▥" }
        }
    }
}

#[component]
fn SystemStatus() -> Element {
    let mut state = use_context::<AppState>();
    let status = match &*state.catalog.load.read() {
        LoadState::Loading => ("INDEXING", "workspace loading", false),
        LoadState::Error(_) => ("ERROR", "workspace unavailable", false),
        LoadState::Empty => ("LOCAL", "workspace empty", true),
        LoadState::Ready => ("LOCAL", "services ready", true),
    };
    rsx! {
        button { class: "system-status", title: "Open execution settings", onclick: move |_| state.shell.overlay.set(Some(Overlay::Settings)),
            span { class: "status-orbit", span { class: if status.2 { "live-dot" } else { "live-dot off" } } }
            span { class: "status-copy", strong { "{status.0}" } small { "{status.1}" } }
        }
    }
}

#[component]
fn UserActions() -> Element {
    let mut state = use_context::<AppState>();
    let mut notifications_open = use_signal(|| false);
    let workspace_error = match &*state.catalog.load.read() {
        LoadState::Error(error) => Some(error.clone()),
        _ => state
            .catalog
            .error
            .read()
            .clone()
            .or_else(|| state.runs.error.read().clone()),
    };
    rsx! {
        div { class: "user-actions",
            div { class: "notification-wrap",
                button { class: "topbar-icon notification-button", title: "Activity", aria_label: "Activity", onclick: move |_| notifications_open.toggle(), "○"
                    if workspace_error.is_some() { span { class: "notification-badge", "1" } }
                }
                if *notifications_open.read() {
                    div { class: "topbar-menu notification-menu",
                        div { class: "menu-heading", strong { "Local activity" } span { if workspace_error.is_some() { "attention" } else { "current" } } }
                        if let Some(error) = workspace_error {
                            div { class: "notification-item", span { class: "event-dot" } div { strong { "Action needs attention" } small { "{error}" } } }
                        } else {
                            div { class: "notification-item", span { class: "event-dot good" } div { strong { "Client ready" } small { "{state.catalog.projects.read().len()} projects · {state.runs.reports.read().len()} reports" } } }
                        }
                    }
                }
            }
            button { class: "avatar", title: "Open settings", aria_label: "Open settings", onclick: move |_| state.shell.overlay.set(Some(Overlay::Settings)), span { "S" } }
        }
    }
}
