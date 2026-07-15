use dioxus::prelude::*;

use super::components::EditorTabButton;
use super::features::{
    AgentsDashboard, AutomationDashboard, PluginDashboard, ProjectsDashboard, ResultDashboard,
    ScannerDashboard, SourceDashboard, TracksDashboard,
};
use super::models::{BottomTab, EditorTab, WorkspaceView};
use super::preview::sample_report;
use super::state::{activate_view, close_view, AppState};

#[component]
pub(crate) fn EditorWorkspace() -> Element {
    let state = use_context::<AppState>();
    let active_view = *state.active_view.read();
    rsx! {
        main { class: "editor",
            WorkspaceEditorTabs {}
            div { class: "editor-surface",
                match active_view {
                    WorkspaceView::Projects => rsx! { ProjectsDashboard {} },
                    WorkspaceView::Explorer => rsx! { ExplorerEditor {} },
                    WorkspaceView::Plugins => rsx! { PluginsEditor {} },
                    WorkspaceView::Tracks => rsx! { TracksEditor {} },
                    WorkspaceView::Automation => rsx! { AutomationDashboard {} },
                    WorkspaceView::Agents => rsx! { AgentsDashboard {} },
                }
            }
        }
    }
}

#[component]
fn WorkspaceEditorTabs() -> Element {
    let state = use_context::<AppState>();
    let active = *state.active_view.read();
    let views = state.open_views.read().clone();
    rsx! {
        div { class: "workbench-tabs", role: "tablist", aria_label: "Open editors",
            for view in views {
                button {
                    class: if view == active { "workbench-tab active" } else { "workbench-tab" },
                    role: "tab",
                    aria_selected: view == active,
                    onclick: move |_| activate_view(state, view),
                    span { class: "tab-icon", "{view.icon()}" }
                    span { "{view.title()}" }
                    span {
                        class: "tab-close",
                        title: "Close editor",
                        onclick: move |event| { event.stop_propagation(); close_view(state, view); },
                        "×"
                    }
                }
            }
        }
    }
}

#[component]
fn ExplorerEditor() -> Element {
    let mut state = use_context::<AppState>();
    let report = state.report.read().clone();
    rsx! {
        div { class: "editor-tabs",
            EditorTabButton { label: "Scanner configuration", icon: "⚡", active: *state.editor_tab.read() == EditorTab::Scanner, onclick: move |_| state.editor_tab.set(EditorTab::Scanner) }
            if report.is_some() { EditorTabButton { label: "Result dashboard", icon: "▥", active: *state.editor_tab.read() == EditorTab::Result, onclick: move |_| state.editor_tab.set(EditorTab::Result) } }
            EditorTabButton { label: "recon_probe.rs", icon: "Rs", active: *state.editor_tab.read() == EditorTab::Source, onclick: move |_| state.editor_tab.set(EditorTab::Source) }
        }
        match *state.editor_tab.read() {
            EditorTab::Scanner => rsx! { ScannerDashboard {
                target: state.selected_target.read().clone(),
                selected_plugin: state.selected_plugin.read().clone(),
                plugins: state.plugins.read().clone(),
                on_target: move |value| state.selected_target.set(value),
                on_plugin: move |value| state.selected_plugin.set(value),
                on_run: move |_| {
                    state.report.set(Some(sample_report(state.selected_target.read().clone())));
                    state.editor_tab.set(EditorTab::Result);
                    state.bottom_tab.set(BottomTab::Problems);
                }
            } },
            EditorTab::Result => rsx! { ResultDashboard { report } },
            EditorTab::Source => rsx! { SourceDashboard {} },
        }
    }
}

#[component]
fn PluginsEditor() -> Element {
    let mut state = use_context::<AppState>();
    rsx! { PluginDashboard {
        plugins: state.plugins.read().clone(),
        selected: state.selected_plugin.read().clone(),
        on_toggle: move |id: String| {
            if let Some(plugin) = state.plugins.write().iter_mut().find(|plugin| plugin.id == id) {
                plugin.enabled = !plugin.enabled;
            }
        }
    } }
}

#[component]
fn TracksEditor() -> Element {
    let mut state = use_context::<AppState>();
    rsx! { TracksDashboard {
        overview: state.overview.read().clone(),
        filter: *state.track_filter.read(),
        selected: *state.selected_track.read(),
        on_select: move |index| state.selected_track.set(index),
    } }
}
