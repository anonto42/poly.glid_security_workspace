use dioxus::prelude::*;
use polyglid_desktop::client::{LocalClient, Report};

use super::commands::{execute, ShellCommand};
use super::components::BottomTabButton;
use super::models::BottomTab;
use super::state::AppState;

#[component]
pub(crate) fn BottomPanel() -> Element {
    let mut state = use_context::<AppState>();
    let client = use_context::<LocalClient>();
    let selected = selected_report(state);
    let issue_count = selected.as_ref().map_or(0, |report| report.issues.len());
    rsx! {
        section { class: "bottom-panel", style: "height: {state.shell.bottom_panel_height}px; flex-basis: {state.shell.bottom_panel_height}px",
            div { class: "bottom-tabs",
                BottomTabButton { label: "Findings", count: Some(issue_count), active: *state.shell.bottom_tab.read() == BottomTab::Findings, onclick: move |_| state.shell.bottom_tab.set(BottomTab::Findings) }
                BottomTabButton { label: "Activity", count: None, active: *state.shell.bottom_tab.read() == BottomTab::Activity, onclick: move |_| state.shell.bottom_tab.set(BottomTab::Activity) }
                div { class: "panel-actions",
                    button { title: "Collapse panel (Ctrl+J)", onclick: move |_| execute(state, ShellCommand::TogglePanel, client.clone()), "×" }
                }
            }
            div { class: "bottom-content",
                match *state.shell.bottom_tab.read() {
                    BottomTab::Findings => rsx! { FindingsPanel { report: selected } },
                    BottomTab::Activity => rsx! { ActivityPanel {} },
                }
            }
        }
    }
}

#[component]
fn FindingsPanel(report: Option<Report>) -> Element {
    rsx! {
        if let Some(value) = report {
            if value.issues.is_empty() {
                div { class: "panel-empty", "The selected report contains no findings." }
            } else {
                div { class: "problems-list",
                    for issue in value.issues {
                        div { class: "finding",
                            span { class: "finding-icon", "!" }
                            div {
                                div { strong { "{issue.title}" } span { class: "badge", "{issue.severity}" } }
                                p { "{issue.description}" }
                                small { "Recommendation: {issue.recommendation}" }
                            }
                        }
                    }
                }
            }
        } else {
            div { class: "panel-empty", "No persisted findings yet. Complete a scan to create a report." }
        }
    }
}

#[component]
fn ActivityPanel() -> Element {
    let state = use_context::<AppState>();
    rsx! {
        div { class: "execution-log", role: "log", aria_live: "polite",
            if let Some(error) = state.runs.error.read().as_ref() {
                p { span { class: "error", "[error]" } " {error}" }
            }
            for message in state.runs.activity.read().iter().rev() {
                p { span { class: "dim", "[local]" } " {message}" }
            }
        }
    }
}

fn selected_report(state: AppState) -> Option<Report> {
    let reports = state.runs.reports.read();
    state
        .runs
        .selected_report_id
        .read()
        .as_ref()
        .and_then(|id| reports.iter().find(|report| &report.id == id))
        .cloned()
        .or_else(|| reports.first().cloned())
}
