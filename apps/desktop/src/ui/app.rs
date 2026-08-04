use dioxus::prelude::*;
use polyglid_desktop::client::{LocalClient, SetupReport, SetupStatus};
use polyglid_desktop::controllers::DesktopControllers;

use super::commands::{handle_shortcut, persist_shell};
use super::editor::EditorWorkspace;
use super::models::{LoadState, ResizeAxis};
use super::shell::{ActivityRail, StatusBar};
use super::sidebar::WorkspaceSidebar;
use super::state::{use_app_state, AppState};
use super::top_bar::TitleBar;

const APP_CSS: &str = concat!(
    include_str!("../../assets/theme.css"),
    include_str!("../../assets/main.css"),
    include_str!("../../assets/shell.css"),
    include_str!("../../assets/projects.css"),
);

#[component]
pub(crate) fn App() -> Element {
    let mut state = use_app_state();
    use_context_provider(|| state);
    let opened = use_hook(LocalClient::open_default_with_setup);
    let (client, setup) = match opened {
        Ok(result) => result,
        Err(error) => {
            return rsx! {
                style { dangerous_inner_html: APP_CSS }
                main { class: "startup-failure",
                    div { class: "state-panel error-state",
                        span { class: "state-icon", "!" }
                        h1 { "PolyGlid could not start" }
                        p { "{error}" }
                        small { "Check POLYGLID_DATA_DIR and POLYGLID_WORKSPACE_ROOT, then restart the desktop client." }
                    }
                }
            };
        }
    };
    let controllers = DesktopControllers::new(client);
    use_context_provider(|| controllers.clone());

    let setup_message = setup_message(&setup);
    use_effect(move || {
        if state.catalog.notice.read().is_none() {
            if let Some(message) = setup_message.clone() {
                state.catalog.notice.set(Some(message));
            }
        }
    });

    load_projects(state, controllers.clone());

    let resize_controllers = controllers.clone();
    let shortcut_controllers = controllers.clone();
    let mut shell_state = state;
    rsx! {
        style { dangerous_inner_html: APP_CSS }
        div {
            class: if state.shell.resizing.read().is_some() { "developer-space resizing" } else { "developer-space" },
            tabindex: 0,
            autofocus: true,
            onkeydown: move |event| handle_shortcut(event, state, shortcut_controllers.clone()),
            onmousemove: move |event| resize_sidebar(shell_state, event),
            onmouseup: move |_| finish_resize(state, resize_controllers.clone()),
            TitleBar {}
            div { class: "workspace-body",
                ActivityRail {}
                if *state.shell.sidebar_visible.read() {
                    WorkspaceSidebar {}
                    div {
                        class: "resize-handle vertical",
                        onmousedown: move |_| shell_state.shell.resizing.set(Some(ResizeAxis::Sidebar))
                    }
                }
                div { class: "main-column", EditorWorkspace {} }
            }
            StatusBar {}
        }
    }
}

fn setup_message(report: &SetupReport) -> Option<String> {
    match report.status {
        SetupStatus::FirstRun => Some(format!(
            "PolyGlid initialized this workspace and applied {} database migrations.",
            report.applied_migrations.len()
        )),
        SetupStatus::Migrated => Some(format!(
            "PolyGlid upgraded this workspace with {} database migration(s).",
            report.applied_migrations.len()
        )),
        SetupStatus::Ready => None,
    }
}

fn load_projects(mut state: AppState, controllers: DesktopControllers) {
    use_effect(move || {
        let refresh = *state.catalog.refresh.read();
        let _refresh = refresh;
        state.catalog.load.set(LoadState::Loading);
        let controller = controllers.application.clone();
        spawn(async move {
            let result = tokio::task::spawn_blocking(move || controller.bootstrap()).await;
            match result {
                Ok(Ok(snapshot)) => {
                    let load = if snapshot.projects.is_empty() {
                        LoadState::Empty
                    } else {
                        LoadState::Ready
                    };
                    let selected_project_is_valid = state
                        .catalog
                        .selected_project_id
                        .read()
                        .as_ref()
                        .is_some_and(|id| {
                            snapshot.projects.iter().any(|project| &project.id == id)
                        });
                    if !selected_project_is_valid {
                        state
                            .catalog
                            .selected_project_id
                            .set(snapshot.projects.first().map(|project| project.id.clone()));
                    }

                    state
                        .catalog
                        .active_workspace_id
                        .set(Some(snapshot.active_workspace.id.clone()));
                    state
                        .catalog
                        .active_workspace_name
                        .set(snapshot.active_workspace.name);
                    state.catalog.workspaces.set(snapshot.workspaces);
                    state.catalog.projects.set(snapshot.projects);
                    state.catalog.error.set(None);
                    state.catalog.load.set(load);
                    state
                        .shell
                        .sidebar_visible
                        .set(snapshot.shell.sidebar_visible);
                    state.shell.sidebar_width.set(snapshot.shell.sidebar_width);
                }
                Ok(Err(error)) => state.catalog.load.set(LoadState::Error(error.to_string())),
                Err(error) => state
                    .catalog
                    .load
                    .set(LoadState::Error(format!("bootstrap task failed: {error}"))),
            }
        });
    });
}

fn resize_sidebar(mut state: AppState, event: MouseEvent) {
    if matches!(*state.shell.resizing.read(), Some(ResizeAxis::Sidebar)) {
        let width = (event.client_coordinates().x - 48.0).clamp(180.0, 480.0);
        state.shell.sidebar_width.set(width);
    }
}

fn finish_resize(mut state: AppState, controllers: DesktopControllers) {
    if state.shell.resizing.read().is_some() {
        state.shell.resizing.set(None);
        persist_shell(state, controllers);
    }
}
