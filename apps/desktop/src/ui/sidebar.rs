use dioxus::prelude::*;

use super::state::AppState;

#[component]
pub(crate) fn WorkspaceSidebar() -> Element {
    let state = use_context::<AppState>();
    rsx! {
        aside {
            class: "sidebar",
            style: "width: {state.shell.sidebar_width}px; flex-basis: {state.shell.sidebar_width}px",
            div { class: "sidebar-heading", span { "Projects" } }
            ProjectsSidebar {}
        }
    }
}

#[component]
fn ProjectsSidebar() -> Element {
    let mut state = use_context::<AppState>();
    let selected_project_id = state.catalog.selected_project_id.read().clone();
    let operation_in_progress = state.catalog.operation.read().is_some();
    rsx! {
        div { class: "sidebar-section",
            p { class: "section-label", "Workspace" }
            div { class: "workspace-summary",
                span { class: "live-dot" }
                div {
                    strong { "{state.catalog.active_workspace_name}" }
                    small { "local catalog" }
                }
            }
        }
        div { class: "sidebar-section grow",
            p { class: "section-label", "Projects · {state.catalog.projects.read().len()}" }
            for project in state.catalog.projects.read().iter() {
                button {
                    class: if selected_project_id.as_ref() == Some(&project.id) { "project-nav active" } else { "project-nav" },
                    aria_label: "Select project {project.name}",
                    onclick: {
                        let project_id = project.id.clone();
                        move |_| state.catalog.selected_project_id.set(Some(project_id.clone()))
                    },
                    span { "◇" }
                    div {
                        strong { "{project.name}" }
                        small { "{project.kind}" }
                    }
                }
            }
            button {
                class: "sidebar-option",
                disabled: operation_in_progress,
                onclick: move |_| {
                    let next = *state.catalog.refresh.read() + 1;
                    state.catalog.refresh.set(next);
                },
                span { "Refresh discovery" }
                small { "↻" }
            }
        }
    }
}
