use dioxus::prelude::*;
use polyglid_core::store::{DbProject, DbWorkspace};
use polyglid_desktop::WorkspaceOverview;

use super::models::{
    BottomTab, EditorTab, PluginCard, ResizeAxis, ScanReport, SettingsTab, TopBarAction,
    TrackFilter, WorkspaceLoadState, WorkspaceView,
};
use super::preview::{seed_overview, seed_plugins, seed_top_bar_actions};

#[derive(Clone, Copy)]
pub(crate) struct AppState {
    pub(crate) active_view: Signal<WorkspaceView>,
    pub(crate) open_views: Signal<Vec<WorkspaceView>>,
    pub(crate) sidebar_visible: Signal<bool>,
    pub(crate) bottom_panel_visible: Signal<bool>,
    pub(crate) sidebar_width: Signal<f64>,
    pub(crate) bottom_panel_height: Signal<f64>,
    pub(crate) resizing: Signal<Option<ResizeAxis>>,
    pub(crate) workspace_load: Signal<WorkspaceLoadState>,
    pub(crate) workspaces: Signal<Vec<DbWorkspace>>,
    pub(crate) projects: Signal<Vec<DbProject>>,
    pub(crate) active_workspace_id: Signal<Option<String>>,
    pub(crate) workspace_refresh: Signal<u64>,
    pub(crate) workspace_mutation_error: Signal<Option<String>>,
    pub(crate) editor_tab: Signal<EditorTab>,
    pub(crate) bottom_tab: Signal<BottomTab>,
    pub(crate) settings_tab: Signal<SettingsTab>,
    pub(crate) settings_open: Signal<bool>,
    pub(crate) command_open: Signal<bool>,
    pub(crate) active_workspace: Signal<String>,
    pub(crate) top_bar_actions: Signal<Vec<TopBarAction>>,
    pub(crate) selected_target: Signal<String>,
    pub(crate) new_target: Signal<String>,
    pub(crate) targets: Signal<Vec<String>>,
    pub(crate) plugins: Signal<Vec<PluginCard>>,
    pub(crate) selected_plugin: Signal<String>,
    pub(crate) report: Signal<Option<ScanReport>>,
    pub(crate) execution_error: Signal<Option<String>>,
    pub(crate) execution_running: Signal<bool>,
    pub(crate) fuel_limit: Signal<u64>,
    pub(crate) track_filter: Signal<TrackFilter>,
    pub(crate) selected_track: Signal<Option<usize>>,
    pub(crate) overview: Signal<WorkspaceOverview>,
}

pub(crate) fn use_app_state() -> AppState {
    AppState {
        active_view: use_signal(|| WorkspaceView::Projects),
        open_views: use_signal(|| vec![WorkspaceView::Projects]),
        sidebar_visible: use_signal(|| true),
        bottom_panel_visible: use_signal(|| true),
        sidebar_width: use_signal(|| 280.0),
        bottom_panel_height: use_signal(|| 210.0),
        resizing: use_signal(|| None),
        workspace_load: use_signal(|| WorkspaceLoadState::Loading),
        workspaces: use_signal(Vec::new),
        projects: use_signal(Vec::new),
        active_workspace_id: use_signal(|| None),
        workspace_refresh: use_signal(|| 0),
        workspace_mutation_error: use_signal(|| None),
        editor_tab: use_signal(|| EditorTab::Scanner),
        bottom_tab: use_signal(|| BottomTab::Problems),
        settings_tab: use_signal(|| SettingsTab::Overview),
        settings_open: use_signal(|| false),
        command_open: use_signal(|| false),
        active_workspace: use_signal(|| "polyglid workspace".to_string()),
        top_bar_actions: use_signal(seed_top_bar_actions),
        selected_target: use_signal(|| "example.com".to_string()),
        new_target: use_signal(String::new),
        targets: use_signal(|| {
            vec![
                "example.com".to_string(),
                "google.com".to_string(),
                "github.com".to_string(),
            ]
        }),
        plugins: use_signal(seed_plugins),
        selected_plugin: use_signal(|| "recon-probe".to_string()),
        report: use_signal(|| None),
        execution_error: use_signal(|| None),
        execution_running: use_signal(|| false),
        fuel_limit: use_signal(|| 25_000_000),
        track_filter: use_signal(|| TrackFilter::All),
        selected_track: use_signal(|| None),
        overview: use_signal(seed_overview),
    }
}

pub(crate) fn activate_view(mut state: AppState, view: WorkspaceView) {
    if !state.open_views.read().contains(&view) {
        state.open_views.write().push(view);
    }
    state.active_view.set(view);
}

pub(crate) fn close_view(mut state: AppState, view: WorkspaceView) {
    let active = *state.active_view.read();
    let mut views = state.open_views.write();
    if views.len() == 1 {
        return;
    }
    let Some(index) = views.iter().position(|candidate| *candidate == view) else {
        return;
    };
    views.remove(index);
    if active == view {
        let next = views[index.saturating_sub(1).min(views.len() - 1)];
        drop(views);
        state.active_view.set(next);
    }
}
