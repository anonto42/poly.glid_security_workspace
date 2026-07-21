use dioxus::prelude::*;

use crate::backend::DesktopBackend;

use super::bottom_panel::BottomPanel;
use super::commands::{handle_shortcut, persist_shell};
use super::editor::EditorWorkspace;
use super::models::PluginCard;
use super::models::{ResizeAxis, WorkspaceLoadState};
use super::overlays::WorkspaceOverlays;
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
    let backend = use_hook(DesktopBackend::open_default);
    use_context_provider(|| backend.clone());
    load_workspace_catalog(state, backend);
    let mouse_backend = use_context::<DesktopBackend>();
    let shortcut_backend = mouse_backend.clone();
    rsx! {
        style { dangerous_inner_html: APP_CSS }
        div {
            class: if state.resizing.read().is_some() { "developer-space resizing" } else { "developer-space" },
            tabindex: 0,
            autofocus: true,
            onkeydown: move |event| handle_shortcut(event, state, shortcut_backend.clone()),
            onmousemove: move |event| resize_shell(state, event),
            onmouseup: move |_| finish_resize(state, mouse_backend.clone()),
            TitleBar {}
            div { class: "workspace-body",
                ActivityRail {}
                if *state.sidebar_visible.read() {
                    WorkspaceSidebar {}
                    div { class: "resize-handle vertical", onmousedown: move |_| state.resizing.set(Some(ResizeAxis::Sidebar)) }
                }
                div { class: "main-column",
                    EditorWorkspace {}
                    if *state.bottom_panel_visible.read() {
                        div { class: "resize-handle horizontal", onmousedown: move |_| state.resizing.set(Some(ResizeAxis::BottomPanel)) }
                        BottomPanel {}
                    }
                }
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
                    let plugins = snapshot
                        .plugins
                        .into_iter()
                        .map(|entry| PluginCard {
                            id: entry.id.as_str().to_string(),
                            name: entry.name,
                            version: entry.version.to_string(),
                            description: entry.description,
                            capabilities: entry
                                .capabilities
                                .into_iter()
                                .map(|capability| capability.as_str().to_string())
                                .collect(),
                            enabled: matches!(
                                entry.status,
                                polyglid_config::plugin_registry::PluginStatus::Enabled
                            ),
                        })
                        .collect::<Vec<_>>();
                    if let Some(first) = plugins.first() {
                        if !plugins
                            .iter()
                            .any(|plugin| plugin.id == *state.selected_plugin.read())
                        {
                            state.selected_plugin.set(first.id.clone());
                        }
                    }
                    state.plugins.set(plugins);
                    state.sidebar_visible.set(snapshot.shell.sidebar_visible);
                    state
                        .bottom_panel_visible
                        .set(snapshot.shell.bottom_panel_visible);
                    state.sidebar_width.set(snapshot.shell.sidebar_width);
                    state
                        .bottom_panel_height
                        .set(snapshot.shell.bottom_panel_height);
                    state.workspace_load.set(load_state);
                    state.workspace_mutation_error.set(None);
                }
                Err(error) => state.workspace_load.set(WorkspaceLoadState::Error(error)),
            }
        });
    });
}

fn resize_shell(mut state: AppState, event: MouseEvent) {
    match *state.resizing.read() {
        Some(ResizeAxis::Sidebar) => {
            let width = (event.client_coordinates().x - 48.0).clamp(180.0, 480.0);
            state.sidebar_width.set(width);
        }
        Some(ResizeAxis::BottomPanel) => {
            let window = dioxus::desktop::window();
            let viewport_height = window.inner_size().height as f64 / window.scale_factor();
            let height =
                (viewport_height - event.client_coordinates().y - 26.0).clamp(120.0, 520.0);
            state.bottom_panel_height.set(height);
        }
        None => {}
    }
}

fn finish_resize(mut state: AppState, backend: DesktopBackend) {
    if state.resizing.read().is_some() {
        state.resizing.set(None);
        persist_shell(state, backend);
    }
}
