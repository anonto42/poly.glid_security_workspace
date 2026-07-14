use dioxus::prelude::*;
use polyglid_desktop::WorkspaceOverview;

use super::models::{
    BottomTab, EditorTab, PluginCard, ScanReport, SettingsTab, TrackFilter, WorkspaceView,
};
use super::preview::{seed_overview, seed_plugins};

#[derive(Clone, Copy)]
pub(crate) struct AppState {
    pub(crate) active_view: Signal<WorkspaceView>,
    pub(crate) editor_tab: Signal<EditorTab>,
    pub(crate) bottom_tab: Signal<BottomTab>,
    pub(crate) settings_tab: Signal<SettingsTab>,
    pub(crate) settings_open: Signal<bool>,
    pub(crate) command_open: Signal<bool>,
    pub(crate) selected_target: Signal<String>,
    pub(crate) new_target: Signal<String>,
    pub(crate) targets: Signal<Vec<String>>,
    pub(crate) plugins: Signal<Vec<PluginCard>>,
    pub(crate) selected_plugin: Signal<String>,
    pub(crate) report: Signal<Option<ScanReport>>,
    pub(crate) fuel_limit: Signal<u64>,
    pub(crate) track_filter: Signal<TrackFilter>,
    pub(crate) selected_track: Signal<Option<usize>>,
    pub(crate) overview: Signal<WorkspaceOverview>,
}

pub(crate) fn use_app_state() -> AppState {
    AppState {
        active_view: use_signal(|| WorkspaceView::Explorer),
        editor_tab: use_signal(|| EditorTab::Scanner),
        bottom_tab: use_signal(|| BottomTab::Problems),
        settings_tab: use_signal(|| SettingsTab::Overview),
        settings_open: use_signal(|| false),
        command_open: use_signal(|| false),
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
        fuel_limit: use_signal(|| 25_000_000),
        track_filter: use_signal(|| TrackFilter::All),
        selected_track: use_signal(|| None),
        overview: use_signal(seed_overview),
    }
}
