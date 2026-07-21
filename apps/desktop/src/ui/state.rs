use dioxus::prelude::*;
use polyglid_desktop::client::{Execution, JobId, Plugin, Project, Report, SavedTarget, Workspace};

use super::models::{BottomTab, LoadState, Overlay, ResizeAxis, SettingsTab, WorkspaceView};

/// Window chrome and navigation state. No product data belongs here.
#[derive(Clone, Copy)]
pub(crate) struct ShellStore {
    pub(crate) active_view: Signal<WorkspaceView>,
    pub(crate) open_views: Signal<Vec<WorkspaceView>>,
    pub(crate) sidebar_visible: Signal<bool>,
    pub(crate) bottom_panel_visible: Signal<bool>,
    pub(crate) sidebar_width: Signal<f64>,
    pub(crate) bottom_panel_height: Signal<f64>,
    pub(crate) resizing: Signal<Option<ResizeAxis>>,
    pub(crate) bottom_tab: Signal<BottomTab>,
    pub(crate) settings_tab: Signal<SettingsTab>,
    pub(crate) overlay: Signal<Option<Overlay>>,
}

/// Workspace catalog state owned by the projects feature.
#[derive(Clone, Copy)]
pub(crate) struct CatalogStore {
    pub(crate) load: Signal<LoadState>,
    pub(crate) workspaces: Signal<Vec<Workspace>>,
    pub(crate) projects: Signal<Vec<Project>>,
    pub(crate) active_workspace_id: Signal<Option<String>>,
    pub(crate) active_workspace_name: Signal<String>,
    pub(crate) refresh: Signal<u64>,
    pub(crate) error: Signal<Option<String>>,
}

/// Installed-component state owned by the plugins feature.
#[derive(Clone, Copy)]
pub(crate) struct PluginStore {
    pub(crate) items: Signal<Vec<Plugin>>,
    pub(crate) selected_id: Signal<Option<String>>,
    pub(crate) install_path: Signal<String>,
}

/// Target, execution, and report state shared by scanner-related features.
#[derive(Clone, Copy)]
pub(crate) struct RunStore {
    pub(crate) targets: Signal<Vec<SavedTarget>>,
    pub(crate) selected_target: Signal<String>,
    pub(crate) new_target: Signal<String>,
    pub(crate) executions: Signal<Vec<Execution>>,
    pub(crate) reports: Signal<Vec<Report>>,
    pub(crate) selected_report_id: Signal<Option<String>>,
    pub(crate) active_job_id: Signal<Option<JobId>>,
    pub(crate) activity: Signal<Vec<String>>,
    pub(crate) error: Signal<Option<String>>,
    pub(crate) fuel_limit: Signal<u64>,
    pub(crate) refresh: Signal<u64>,
}

/// Root state is deliberately a small composition of feature stores.
#[derive(Clone, Copy)]
pub(crate) struct AppState {
    pub(crate) shell: ShellStore,
    pub(crate) catalog: CatalogStore,
    pub(crate) plugins: PluginStore,
    pub(crate) runs: RunStore,
}

pub(crate) fn use_app_state() -> AppState {
    AppState {
        shell: ShellStore {
            active_view: use_signal(|| WorkspaceView::Projects),
            open_views: use_signal(|| vec![WorkspaceView::Projects]),
            sidebar_visible: use_signal(|| true),
            bottom_panel_visible: use_signal(|| true),
            sidebar_width: use_signal(|| 280.0),
            bottom_panel_height: use_signal(|| 210.0),
            resizing: use_signal(|| None),
            bottom_tab: use_signal(|| BottomTab::Findings),
            settings_tab: use_signal(|| SettingsTab::Overview),
            overlay: use_signal(|| None),
        },
        catalog: CatalogStore {
            load: use_signal(|| LoadState::Loading),
            workspaces: use_signal(Vec::new),
            projects: use_signal(Vec::new),
            active_workspace_id: use_signal(|| None),
            active_workspace_name: use_signal(|| "PolyGlid Projects".to_string()),
            refresh: use_signal(|| 0),
            error: use_signal(|| None),
        },
        plugins: PluginStore {
            items: use_signal(Vec::new),
            selected_id: use_signal(|| None),
            install_path: use_signal(String::new),
        },
        runs: RunStore {
            targets: use_signal(Vec::new),
            selected_target: use_signal(String::new),
            new_target: use_signal(String::new),
            executions: use_signal(Vec::new),
            reports: use_signal(Vec::new),
            selected_report_id: use_signal(|| None),
            active_job_id: use_signal(|| None),
            activity: use_signal(|| vec!["Local client initialized".to_string()]),
            error: use_signal(|| None),
            fuel_limit: use_signal(|| 25_000_000),
            refresh: use_signal(|| 0),
        },
    }
}

pub(crate) fn activate_view(mut state: AppState, view: WorkspaceView) {
    if !state.shell.open_views.read().contains(&view) {
        state.shell.open_views.write().push(view);
    }
    state.shell.active_view.set(view);
}

pub(crate) fn close_view(mut state: AppState, view: WorkspaceView) {
    let active = *state.shell.active_view.read();
    let mut views = state.shell.open_views.write();
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
        state.shell.active_view.set(next);
    }
}

pub(crate) fn refresh_operational_data(mut state: AppState) {
    let next = *state.runs.refresh.read() + 1;
    state.runs.refresh.set(next);
}

pub(crate) fn push_activity(mut state: AppState, message: impl Into<String>) {
    let mut activity = state.runs.activity.write();
    activity.push(message.into());
    const MAX_MESSAGES: usize = 100;
    if activity.len() > MAX_MESSAGES {
        let excess = activity.len() - MAX_MESSAGES;
        activity.drain(0..excess);
    }
}
