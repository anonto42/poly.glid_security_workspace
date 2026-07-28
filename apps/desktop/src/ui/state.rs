use dioxus::prelude::*;
use polyglid_desktop::client::{Project, Workspace};

use super::models::{LoadState, ResizeAxis};

#[derive(Clone, Copy)]
pub(crate) struct ShellStore {
    pub(crate) sidebar_visible: Signal<bool>,
    pub(crate) sidebar_width: Signal<f64>,
    pub(crate) resizing: Signal<Option<ResizeAxis>>,
}

#[derive(Clone, Copy)]
pub(crate) struct CatalogStore {
    pub(crate) load: Signal<LoadState>,
    pub(crate) workspaces: Signal<Vec<Workspace>>,
    pub(crate) projects: Signal<Vec<Project>>,
    pub(crate) active_workspace_id: Signal<Option<String>>,
    pub(crate) selected_project_id: Signal<Option<String>>,
    pub(crate) active_workspace_name: Signal<String>,
    pub(crate) refresh: Signal<u64>,
    pub(crate) error: Signal<Option<String>>,
    pub(crate) notice: Signal<Option<String>>,
    pub(crate) operation: Signal<Option<String>>,
}

#[derive(Clone, Copy)]
pub(crate) struct AppState {
    pub(crate) shell: ShellStore,
    pub(crate) catalog: CatalogStore,
}

pub(crate) fn use_app_state() -> AppState {
    AppState {
        shell: ShellStore {
            sidebar_visible: use_signal(|| true),
            sidebar_width: use_signal(|| 280.0),
            resizing: use_signal(|| None),
        },
        catalog: CatalogStore {
            load: use_signal(|| LoadState::Loading),
            workspaces: use_signal(Vec::new),
            projects: use_signal(Vec::new),
            active_workspace_id: use_signal(|| None),
            selected_project_id: use_signal(|| None),
            active_workspace_name: use_signal(|| "PolyGlid Projects".to_string()),
            refresh: use_signal(|| 0),
            error: use_signal(|| None),
            notice: use_signal(|| None),
            operation: use_signal(|| None),
        },
    }
}
