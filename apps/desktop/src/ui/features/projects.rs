use dioxus::prelude::*;
use polyglid_desktop::client::Project;
use polyglid_desktop::controllers::DesktopControllers;

use super::super::models::LoadState;
use super::super::state::{push_activity, AppState};

#[component]
pub(crate) fn ProjectsDashboard() -> Element {
    let mut state = use_context::<AppState>();
    let client = use_context::<DesktopControllers>();
    let create_client = client.clone();
    let mut new_name = use_signal(String::new);
    let load_state = state.catalog.load.read().clone();
    let projects = state.catalog.projects.read().clone();
    let active_operation = state.catalog.operation.read().clone();
    let operation_in_progress = active_operation.is_some();
    let creating_project = active_operation.as_deref() == Some("Creating project");

    rsx! {
        div {
            class: "dashboard-scroll projects-page",
            aria_busy: operation_in_progress,
            div { class: "projects-hero",
                div { class: "page-heading",
                    span { class: "eyebrow", "Local workspace catalog" }
                    h1 { "My Projects" }
                    p { "Discover and manage real project folders from {state.catalog.active_workspace_name}." }
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
                        disabled: operation_in_progress || new_name.read().trim().is_empty(),
                        aria_busy: creating_project,
                        onclick: move |_| {
                            let Some(workspace_id) = state.catalog.active_workspace_id.read().clone() else { return; };
                            let name = new_name.read().trim().to_string();
                            new_name.set(String::new());
                            let client = create_client.clone();
                            run_mutation(
                                state,
                                "Creating project",
                                format!("Created project {name}"),
                                move || client.projects.create(&workspace_id, &name).map(|_| ()).map_err(|error| error.to_string()),
                            );
                        },
                        if creating_project { "Creating…" } else { "+ Create project" }
                    }
                }
            }
            if let Some(error) = state.catalog.error.read().as_ref() {
                div {
                    class: "project-alert",
                    role: "alert",
                    strong { "Action failed" }
                    span { "{error}" }
                    button {
                        class: "ghost-button",
                        aria_label: "Dismiss project error",
                        onclick: move |_| state.catalog.error.set(None),
                        "Dismiss"
                    }
                }
            }
            if let Some(operation) = active_operation {
                div { class: "project-operation", role: "status", "{operation}…" }
            }
            match load_state {
                LoadState::Loading => rsx! { ProjectSkeleton {} },
                LoadState::Error(error) => rsx! {
                    div { class: "project-state error-state", h2 { "Workspace unavailable" } p { "{error}" }
                        button { class: "secondary", onclick: move |_| refresh(state), "Try again" }
                    }
                },
                LoadState::Empty => rsx! {
                    div { class: "project-state", h2 { "No projects yet" }
                        p { "Create a project here or add a folder inside the active workspace, then refresh discovery." }
                    }
                },
                LoadState::Ready => rsx! {
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
fn ProjectCard(project: Project) -> Element {
    let client = use_context::<DesktopControllers>();
    let mut state = use_context::<AppState>();
    let mut editing = use_signal(|| false);
    let mut confirming = use_signal(|| false);
    let mut name = use_signal(|| project.name.clone());
    let project_id = project.id.clone();
    let active_operation = state.catalog.operation.read().clone();
    let operation_in_progress = active_operation.is_some();
    let renaming_project = active_operation.as_deref() == Some("Renaming project");
    let removing_project = active_operation.as_deref() == Some("Removing project");
    let deleting_project = active_operation.as_deref() == Some("Deleting project files");
    let rename_is_invalid = name.read().trim().is_empty() || name.read().trim() == project.name;
    let selected = state
        .catalog
        .selected_project_id
        .read()
        .as_ref()
        .is_some_and(|id| id == &project.id);
    rsx! {
        article { class: if selected { "project-card selected" } else { "project-card" },
            div { class: "project-card-head", span { class: "project-symbol", "◇" } span { class: "badge good", "{project.kind}" } }
            if *editing.read() {
                input {
                    value: "{name}",
                    aria_label: "Rename project",
                    disabled: operation_in_progress,
                    oninput: move |event| name.set(event.value())
                }
            } else {
                h2 { "{project.name}" }
            }
            p { class: "project-path", title: "{project.path}", "{project.path}" }
            div { class: "project-actions",
                if *editing.read() {
                    button {
                        class: "secondary",
                        disabled: operation_in_progress || rename_is_invalid,
                        aria_busy: renaming_project,
                        onclick: {
                        let project_id = project_id.clone();
                        let client = client.clone();
                        move |_| {
                            let id = project_id.clone();
                            let name = name.read().trim().to_string();
                            let client = client.clone();
                            run_mutation(
                                state,
                                "Renaming project",
                                format!("Renamed project to {name}"),
                                move || client.projects.rename(&id, &name).map(|_| ()).map_err(|error| error.to_string()),
                            );
                            editing.set(false);
                        }
                    },
                        if renaming_project { "Saving…" } else { "Save" }
                    }
                    button { class: "ghost-button", disabled: operation_in_progress, onclick: move |_| editing.set(false), "Cancel" }
                } else if *confirming.read() {
                    div { class: "project-delete-warning", role: "alert",
                        strong { "Remove {project.name}?" }
                        span { "Removing keeps the folder. Deleting files permanently removes {project.path}." }
                    }
                    button {
                        class: "secondary",
                        disabled: operation_in_progress,
                        aria_busy: removing_project,
                        onclick: {
                        let project_id = project_id.clone();
                        let client = client.clone();
                        move |_| {
                            let id = project_id.clone();
                            let client = client.clone();
                            run_mutation(
                                state,
                                "Removing project",
                                "Removed project from the catalog",
                                move || client.projects.remove(&id, false).map_err(|error| error.to_string()),
                            );
                        }
                    },
                        if removing_project { "Removing…" } else { "Keep folder" }
                    }
                    button {
                        class: "danger-button",
                        disabled: operation_in_progress,
                        aria_busy: deleting_project,
                        onclick: {
                        let project_id = project_id.clone();
                        let client = client.clone();
                        move |_| {
                            let id = project_id.clone();
                            let client = client.clone();
                            run_mutation(
                                state,
                                "Deleting project files",
                                "Deleted project files",
                                move || client.projects.remove(&id, true).map_err(|error| error.to_string()),
                            );
                        }
                    },
                        if deleting_project { "Deleting…" } else { "Delete folder" }
                    }
                    button { class: "ghost-button", disabled: operation_in_progress, onclick: move |_| confirming.set(false), "Cancel" }
                } else {
                    button {
                        class: if selected { "secondary" } else { "primary small" },
                        disabled: selected || operation_in_progress,
                        onclick: {
                            let project_id = project_id.clone();
                            move |_| state.catalog.selected_project_id.set(Some(project_id.clone()))
                        },
                        if selected { "Selected" } else { "Use project" }
                    }
                    button { class: "secondary", disabled: operation_in_progress, onclick: move |_| editing.set(true), "Rename" }
                    button { class: "ghost-button", disabled: operation_in_progress, onclick: move |_| confirming.set(true), "Remove" }
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
    operation_name: &'static str,
    success_message: impl Into<String>,
    operation: impl FnOnce() -> Result<(), String> + Send + 'static,
) {
    if state.catalog.operation.read().is_some() {
        return;
    }
    let success_message = success_message.into();
    state.catalog.error.set(None);
    state
        .catalog
        .operation
        .set(Some(operation_name.to_string()));
    spawn(async move {
        let result = tokio::task::spawn_blocking(operation)
            .await
            .map_err(|error| format!("project task failed: {error}"))
            .and_then(|result| result);
        match result {
            Ok(()) => {
                push_activity(state, success_message);
                refresh(state);
            }
            Err(error) => state.catalog.error.set(Some(error)),
        }
        state.catalog.operation.set(None);
    });
}

fn refresh(mut state: AppState) {
    let next = *state.catalog.refresh.read() + 1;
    state.catalog.refresh.set(next);
}
