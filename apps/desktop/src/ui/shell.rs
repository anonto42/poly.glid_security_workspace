use dioxus::prelude::*;
use polyglid_desktop::client::LocalClient;

use super::commands::{execute, ShellCommand};
use super::components::RailButton;
use super::models::{execution_state_class, LoadState, Overlay, WorkspaceView};
use super::state::{activate_view, AppState};

#[component]
pub(crate) fn ActivityRail() -> Element {
    let mut state = use_context::<AppState>();
    let current = *state.shell.active_view.read();
    rsx! {
        nav { class: "activity-rail", aria_label: "PolyGlid product areas",
            RailButton { icon: "▦", label: "Projects", active: current == WorkspaceView::Projects, onclick: move |_| activate_view(state, WorkspaceView::Projects) }
            RailButton { icon: "⌕", label: "New scan", active: current == WorkspaceView::Scanner, onclick: move |_| activate_view(state, WorkspaceView::Scanner) }
            RailButton { icon: "▷", label: "Executions", active: current == WorkspaceView::Executions, onclick: move |_| activate_view(state, WorkspaceView::Executions) }
            RailButton { icon: "▥", label: "Reports", active: current == WorkspaceView::Reports, onclick: move |_| activate_view(state, WorkspaceView::Reports) }
            RailButton { icon: "◇", label: "Plugins", active: current == WorkspaceView::Plugins, onclick: move |_| activate_view(state, WorkspaceView::Plugins) }
            div { class: "rail-spacer" }
            RailButton { icon: "⚒", label: "Settings", active: matches!(&*state.shell.overlay.read(), Some(Overlay::Settings)), onclick: move |_| state.shell.overlay.set(Some(Overlay::Settings)) }
        }
    }
}

#[component]
pub(crate) fn StatusBar() -> Element {
    let state = use_context::<AppState>();
    let client = use_context::<LocalClient>();
    let catalog_status = match &*state.catalog.load.read() {
        LoadState::Loading => "Catalog indexing",
        LoadState::Error(_) => "Catalog error",
        LoadState::Empty => "Catalog empty",
        LoadState::Ready => "Catalog ready",
    };
    let active_state = state.runs.active_job_id.read().and_then(|id| {
        state
            .runs
            .executions
            .read()
            .iter()
            .find(|run| run.id == id)
            .map(|run| run.state)
    });
    let active_status = active_state.map_or("idle", |phase| phase.as_str());
    let active_class = active_state.map_or("idle", execution_state_class);
    rsx! {
        footer { class: "statusbar",
            div { span { "◈" } " {catalog_status}" }
            div { span { class: "status-dot {active_class}" } " Execution {active_status}" }
            div { class: "status-spacer" }
            div { "{state.catalog.projects.read().len()} projects" }
            div { "{state.runs.reports.read().len()} reports" }
            button { class: "status-control", title: "Toggle activity panel (Ctrl+J)", onclick: move |_| execute(state, ShellCommand::TogglePanel, client.clone()), "▱" }
        }
    }
}
