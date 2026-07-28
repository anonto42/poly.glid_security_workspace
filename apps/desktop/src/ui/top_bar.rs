use dioxus::prelude::*;
use polyglid_desktop::controllers::DesktopControllers;

use super::models::LoadState;
use super::state::AppState;

#[component]
pub(crate) fn TitleBar() -> Element {
    rsx! {
        header { class: "titlebar",
            BrandArea {}
            div { class: "phase-title", "Projects" }
            SystemStatus {}
        }
    }
}

#[component]
fn BrandArea() -> Element {
    let mut state = use_context::<AppState>();
    let controllers = use_context::<DesktopControllers>();
    let mut menu_open = use_signal(|| false);
    let workspaces = state.catalog.workspaces.read().clone();
    rsx! {
        div { class: "brand-area",
            div { class: "wordmark",
                span { class: "wordmark-icon", "P" }
                div { strong { "polyglid" } small { "security workspace" } }
            }
            div { class: "workspace-picker",
                button {
                    class: "workspace-trigger",
                    aria_label: "Choose workspace",
                    onclick: move |_| menu_open.toggle(),
                    span { class: "workspace-pulse" }
                    span { "{state.catalog.active_workspace_name}" }
                    span { class: "chevron", "⌄" }
                }
                if *menu_open.read() {
                    div { class: "topbar-menu workspace-menu",
                        p { "Workspace" }
                        for workspace in workspaces {
                            button {
                                class: if workspace.is_active { "selected" } else { "" },
                                onclick: {
                                    let controllers = controllers.clone();
                                    let workspace_id = workspace.id.clone();
                                    move |_| {
                                        menu_open.set(false);
                                        let controllers = controllers.clone();
                                        let workspace_id = workspace_id.clone();
                                        spawn(async move {
                                            let result = tokio::task::spawn_blocking(move || {
                                                controllers.application.activate_workspace(&workspace_id)
                                            })
                                            .await;
                                            match result {
                                                Ok(Ok(())) => {
                                                    let next = *state.catalog.refresh.read() + 1;
                                                    state.catalog.refresh.set(next);
                                                }
                                                Ok(Err(error)) => {
                                                    state.catalog.error.set(Some(error.to_string()))
                                                }
                                                Err(error) => state.catalog.error.set(Some(
                                                    format!("workspace task failed: {error}"),
                                                )),
                                            }
                                        });
                                    }
                                },
                                span { if workspace.is_active { "◈" } else { "◇" } }
                                div { strong { "{workspace.name}" } small { "{workspace.root_path}" } }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SystemStatus() -> Element {
    let state = use_context::<AppState>();
    let status = match &*state.catalog.load.read() {
        LoadState::Loading => ("INDEXING", "workspace loading", false),
        LoadState::Error(_) => ("ERROR", "workspace unavailable", false),
        LoadState::Empty => ("LOCAL", "workspace empty", true),
        LoadState::Ready => ("LOCAL", "catalog ready", true),
    };
    rsx! {
        div { class: "system-status",
            span { class: "status-orbit",
                span { class: if status.2 { "live-dot" } else { "live-dot off" } }
            }
            span { class: "status-copy",
                strong { "{status.0}" }
                small { "{status.1}" }
            }
        }
    }
}
