use dioxus::prelude::*;
use polyglid_core::store::DbProject;

use crate::backend::DesktopBackend;

use super::super::models::WorkspaceLoadState;
use super::super::state::AppState;

#[component]
pub(crate) fn ProjectsDashboard() -> Element {
    let state = use_context::<AppState>();
    let backend = use_context::<DesktopBackend>();
    let create_backend = backend.clone();
    let mut new_name = use_signal(String::new);
    let load_state = state.workspace_load.read().clone();
    let projects = state.projects.read().clone();

    rsx! {
        div { class: "dashboard-scroll projects-page",
            div { class: "projects-hero",
                div { class: "page-heading",
                    span { class: "eyebrow", "Local workspace catalog" }
                    h1 { "My Projects" }
                    p { "Discover and manage real project folders from {state.active_workspace}." }
                }
                div { class: "project-create",
                    input {
                        value: "{new_name}",
                        placeholder: "New project name",
                        aria_label: "New project name",
                        oninput: move |event| new_name.set(event.value())
                    }
                    button {
                        class: "primary small",
                        disabled: new_name.read().trim().is_empty(),
                        onclick: move |_| {
                            let Some(workspace_id) = state.active_workspace_id.read().clone() else { return; };
                            let name = new_name.read().trim().to_string();
                            new_name.set(String::new());
                            let backend = create_backend.clone();
                            run_mutation(state, move || backend.create_project(&workspace_id, &name));
                        },
                        "+ Create project"
                    }
                }
            }
            if let Some(error) = state.workspace_mutation_error.read().as_ref() {
                div { class: "project-alert", strong { "Action failed" } span { "{error}" } }
            }
            match load_state {
                WorkspaceLoadState::Loading => rsx! { ProjectSkeleton {} },
                WorkspaceLoadState::Error(error) => rsx! {
                    div { class: "project-state error-state", h2 { "Workspace unavailable" } p { "{error}" }
                        button { class: "secondary", onclick: move |_| refresh(state), "Try again" }
                    }
                },
                WorkspaceLoadState::Empty => rsx! {
                    div { class: "project-state", h2 { "No projects yet" }
                        p { "Create a project here or add a folder inside the active workspace, then refresh discovery." }
                    }
                },
                WorkspaceLoadState::Ready => rsx! {
                    div { class: "project-grid",
                        for project in projects {
                            ProjectCard {
                                key: "{project.id}",
                                project,
                            }
                        }
                    }
                },
            }
        }
    }
}

#[component]
fn ProjectCard(project: DbProject) -> Element {
    let backend = use_context::<DesktopBackend>();
    let state = use_context::<AppState>();
    let mut editing = use_signal(|| false);
    let mut confirming = use_signal(|| false);
    let mut name = use_signal(|| project.name.clone());
    let project_id = project.id.clone();
    rsx! {
        article { class: "project-card",
            div { class: "project-card-head", span { class: "project-symbol", "◇" } span { class: "badge good", "{project.kind}" } }
            if *editing.read() {
                input { value: "{name}", aria_label: "Rename project", oninput: move |event| name.set(event.value()) }
            } else {
                h2 { "{project.name}" }
            }
            p { class: "project-path", title: "{project.path}", "{project.path}" }
            div { class: "project-actions",
                if *editing.read() {
                    button { class: "secondary", onclick: {
                        let project_id = project_id.clone();
                        let backend = backend.clone();
                        move |_| {
                            let id = project_id.clone();
                            let name = name.read().trim().to_string();
                            let backend = backend.clone();
                            run_mutation(state, move || backend.rename_project(&id, &name));
                            editing.set(false);
                        }
                    }, "Save" }
                    button { class: "ghost-button", onclick: move |_| editing.set(false), "Cancel" }
                } else if *confirming.read() {
                    button { class: "secondary", onclick: {
                        let project_id = project_id.clone();
                        let backend = backend.clone();
                        move |_| {
                            let id = project_id.clone();
                            let backend = backend.clone();
                            run_mutation(state, move || backend.remove_project(&id, false));
                        }
                    }, "Remove only" }
                    button { class: "danger-button", onclick: {
                        let project_id = project_id.clone();
                        let backend = backend.clone();
                        move |_| {
                            let id = project_id.clone();
                            let backend = backend.clone();
                            run_mutation(state, move || backend.remove_project(&id, true));
                        }
                    }, "Delete files" }
                    button { class: "ghost-button", onclick: move |_| confirming.set(false), "Cancel" }
                } else {
                    button { class: "secondary", onclick: move |_| editing.set(true), "Rename" }
                    button { class: "ghost-button", onclick: move |_| confirming.set(true), "Remove" }
                }
            }
        }
    }
}

#[component]
fn ProjectSkeleton() -> Element {
    rsx! { div { class: "project-grid", for _ in 0..3 { div { class: "project-card project-skeleton", div {} div {} div {} } } } }
}

fn run_mutation(
    mut state: AppState,
    operation: impl FnOnce() -> Result<(), String> + Send + 'static,
) {
    state.workspace_mutation_error.set(None);
    spawn(async move {
        let result = tokio::task::spawn_blocking(operation)
            .await
            .map_err(|error| format!("project task failed: {error}"))
            .and_then(|result| result);
        match result {
            Ok(()) => refresh(state),
            Err(error) => state.workspace_mutation_error.set(Some(error)),
        }
    });
}

fn refresh(mut state: AppState) {
    let next = *state.workspace_refresh.read() + 1;
    state.workspace_refresh.set(next);
}
