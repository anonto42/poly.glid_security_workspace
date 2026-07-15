use dioxus::prelude::*;

use crate::backend::DesktopBackend;

use super::state::AppState;

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
            kbd { "⌘ K" }
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
                button { id: "{action.id}", class: "topbar-icon plugin-action", title: "{action.source}: {action.label}", aria_label: "{action.label}", onclick: { let destination = action.destination; move |_| state.active_view.set(destination) }, span { "{action.icon}" } }
            }
            button { class: "topbar-icon", title: "More extension actions", aria_label: "More extension actions", onclick: move |_| state.command_open.set(true), "••" }
        }
    }
}

#[component]
fn SystemStatus() -> Element {
    let mut state = use_context::<AppState>();
    rsx! {
        button { class: "system-status", title: "Open runtime settings", onclick: move |_| state.settings_open.set(true),
            span { class: "status-orbit", span { class: "live-dot" } }
            span { class: "status-copy", strong { "LOCAL" } small { "core ready" } }
        }
    }
}

#[component]
fn UserActions() -> Element {
    let mut state = use_context::<AppState>();
    let mut notifications_open = use_signal(|| false);
    rsx! {
        div { class: "user-actions",
            div { class: "notification-wrap",
                button { class: "topbar-icon notification-button", title: "Notifications", aria_label: "Notifications", onclick: move |_| notifications_open.toggle(), "○" span { class: "notification-badge", "2" } }
                if *notifications_open.read() {
                    div { class: "topbar-menu notification-menu",
                        div { class: "menu-heading", strong { "Activity" } span { "2 new" } }
                        div { class: "notification-item", span { class: "event-dot good" } div { strong { "Workspace ready" } small { "All local services are available." } } }
                        div { class: "notification-item", span { class: "event-dot" } div { strong { "Preview actions loaded" } small { "2 extension actions registered." } } }
                    }
                }
            }
            button { class: "avatar", title: "Open settings", aria_label: "Open settings", onclick: move |_| state.settings_open.set(true), span { "S" } }
        }
    }
}
