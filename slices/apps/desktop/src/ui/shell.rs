use dioxus::prelude::*;

use crate::backend::DesktopBackend;

use super::commands::{execute, ShellCommand};
use super::components::RailButton;
use super::models::{WorkspaceLoadState, WorkspaceView};
use super::state::{activate_view, AppState};

#[component]
pub(crate) fn ActivityRail() -> Element {
    let mut state = use_context::<AppState>();
    let current = *state.active_view.read();
    rsx! {
        nav { class: "activity-rail", aria_label: "Developer space sections",
            RailButton { icon: "▦", label: "My Projects", active: current == WorkspaceView::Projects, onclick: move |_| activate_view(state, WorkspaceView::Projects) }
            RailButton { icon: "⌕", label: "Scanner", active: current == WorkspaceView::Explorer, onclick: move |_| activate_view(state, WorkspaceView::Explorer) }
            RailButton { icon: "◇", label: "Plugins", active: current == WorkspaceView::Plugins, onclick: move |_| activate_view(state, WorkspaceView::Plugins) }
            RailButton { icon: "☷", label: "Work tracks", active: current == WorkspaceView::Tracks, onclick: move |_| activate_view(state, WorkspaceView::Tracks) }
            RailButton { icon: "⚙", label: "Automation", active: current == WorkspaceView::Automation, onclick: move |_| activate_view(state, WorkspaceView::Automation) }
            RailButton { icon: "✦", label: "AI agents", active: current == WorkspaceView::Agents, onclick: move |_| activate_view(state, WorkspaceView::Agents) }
            div { class: "rail-spacer" }
            RailButton { icon: "⚒", label: "Settings", active: false, onclick: move |_| state.settings_open.set(true) }
        }
    }
}

#[component]
pub(crate) fn StatusBar() -> Element {
    let state = use_context::<AppState>();
    let backend = use_context::<DesktopBackend>();
    let catalog_status = match &*state.workspace_load.read() {
        WorkspaceLoadState::Loading => "Catalog indexing",
        WorkspaceLoadState::Error(_) => "Catalog error",
        WorkspaceLoadState::Empty => "Catalog empty",
        WorkspaceLoadState::Ready => "Catalog ready",
    };
    rsx! {
        footer { class: "statusbar",
            div { span { "◈" } " {catalog_status}" }
            div { span { "◉" } " SQLite local" }
            div { class: "status-spacer" }
            div { "Projects: {state.projects.read().len()}" }
            div { "Rust · Dioxus" }
            button { class: "status-control", title: "Toggle panel (Ctrl+J)", onclick: move |_| execute(state, ShellCommand::TogglePanel, backend.clone()), "▱" }
        }
    }
}
