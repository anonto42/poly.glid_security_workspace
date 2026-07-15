use dioxus::prelude::*;

use crate::backend::DesktopBackend;

use super::bottom_panel::BottomPanel;
use super::editor::EditorWorkspace;
use super::models::WorkspaceLoadState;
use super::overlays::WorkspaceOverlays;
use super::shell::{ActivityRail, StatusBar};
use super::sidebar::WorkspaceSidebar;
use super::state::{use_app_state, AppState};
use super::top_bar::TitleBar;

const APP_CSS: &str = concat!(
    include_str!("../../assets/theme.css"),
    include_str!("../../assets/main.css"),
    include_str!("../../assets/projects.css"),
);

#[component]
pub(crate) fn App() -> Element {
    let state = use_app_state();
    use_context_provider(|| state);
    let backend = use_hook(DesktopBackend::open_default);
    use_context_provider(|| backend.clone());
    load_workspace_catalog(state, backend);
    rsx! {
        style { dangerous_inner_html: APP_CSS }
        div { class: "developer-space",
            TitleBar {}
            div { class: "workspace-body",
                ActivityRail {}
                WorkspaceSidebar {}
                div { class: "main-column", EditorWorkspace {} BottomPanel {} }
            }
            StatusBar {}
            WorkspaceOverlays {}
        }
    }
}

fn load_workspace_catalog(mut state: AppState, backend: DesktopBackend) {
    use_effect(move || {
        let refresh = *state.workspace_refresh.read();
        let backend = backend.clone();
        let _ = refresh;
        state.workspace_load.set(WorkspaceLoadState::Loading);
        spawn(async move {
            let result = tokio::task::spawn_blocking(move || backend.load())
                .await
                .map_err(|error| format!("workspace task failed: {error}"))
                .and_then(|result| result);
            match result {
                Ok(snapshot) => {
                    state.active_workspace.set(snapshot.active.name.clone());
                    state.active_workspace_id.set(Some(snapshot.active.id));
                    state.workspaces.set(snapshot.workspaces);
                    let load_state = if snapshot.projects.is_empty() {
                        WorkspaceLoadState::Empty
                    } else {
                        WorkspaceLoadState::Ready
                    };
                    state.projects.set(snapshot.projects);
                    state.workspace_load.set(load_state);
                    state.workspace_mutation_error.set(None);
                }
                Err(error) => state.workspace_load.set(WorkspaceLoadState::Error(error)),
            }
        });
    });
}
