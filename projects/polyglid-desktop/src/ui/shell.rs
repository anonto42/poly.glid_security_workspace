use dioxus::prelude::*;

use super::components::RailButton;
use super::models::WorkspaceView;
use super::state::AppState;

#[component]
pub(crate) fn TitleBar() -> Element {
    let mut state = use_context::<AppState>();
    rsx! {
        header { class: "titlebar",
            div { class: "wordmark", span { class: "wordmark-icon", "P" } strong { "polyglid" } span { "/ developer space" } }
            button { class: "command-trigger", onclick: move |_| state.command_open.set(true), span { "Search workspace or run a command" } kbd { "⌘ K" } }
            div { class: "window-context", span { class: "live-dot" } "local workspace" button { class: "avatar", "S" } }
        }
    }
}

#[component]
pub(crate) fn ActivityRail() -> Element {
    let mut state = use_context::<AppState>();
    let current = *state.active_view.read();
    rsx! {
        nav { class: "activity-rail", aria_label: "Developer space sections",
            RailButton { icon: "⌂", label: "Explorer", active: current == WorkspaceView::Explorer, onclick: move |_| state.active_view.set(WorkspaceView::Explorer) }
            RailButton { icon: "◇", label: "Plugins", active: current == WorkspaceView::Plugins, onclick: move |_| state.active_view.set(WorkspaceView::Plugins) }
            RailButton { icon: "☷", label: "Work tracks", active: current == WorkspaceView::Tracks, onclick: move |_| state.active_view.set(WorkspaceView::Tracks) }
            RailButton { icon: "⚙", label: "Automation", active: current == WorkspaceView::Automation, onclick: move |_| state.active_view.set(WorkspaceView::Automation) }
            RailButton { icon: "✦", label: "AI agents", active: current == WorkspaceView::Agents, onclick: move |_| state.active_view.set(WorkspaceView::Agents) }
            div { class: "rail-spacer" }
            RailButton { icon: "⚒", label: "Settings", active: false, onclick: move |_| state.settings_open.set(true) }
        }
    }
}

#[component]
pub(crate) fn StatusBar() -> Element {
    let state = use_context::<AppState>();
    let enabled_plugins = state
        .plugins
        .read()
        .iter()
        .filter(|plugin| plugin.enabled)
        .count();
    rsx! {
        footer { class: "statusbar",
            div { span { "◈" } " PolyGlid Core Ready" }
            div { span { "◉" } " Wasmtime Engine" }
            div { class: "status-spacer" }
            div { "Fuel: {state.fuel_limit}" }
            div { "Plugins: {enabled_plugins}" }
            div { "Rust · local" }
        }
    }
}
