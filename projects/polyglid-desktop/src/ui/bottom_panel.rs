use dioxus::prelude::*;

use crate::backend::DesktopBackend;

use super::commands::{execute, ShellCommand};
use super::components::BottomTabButton;
use super::models::{BottomTab, ScanReport};
use super::state::AppState;

#[component]
pub(crate) fn BottomPanel() -> Element {
    let mut state = use_context::<AppState>();
    let backend = use_context::<DesktopBackend>();
    let report = state.report.read().clone();
    let issue_count = report.as_ref().map_or(0, |value| value.findings.len());
    rsx! {
        section { class: "bottom-panel", style: "height: {state.bottom_panel_height}px; flex-basis: {state.bottom_panel_height}px",
            div { class: "bottom-tabs",
                BottomTabButton { label: "Problems", count: Some(issue_count), active: *state.bottom_tab.read() == BottomTab::Problems, onclick: move |_| state.bottom_tab.set(BottomTab::Problems) }
                BottomTabButton { label: "Output", count: None, active: *state.bottom_tab.read() == BottomTab::Output, onclick: move |_| state.bottom_tab.set(BottomTab::Output) }
                BottomTabButton { label: "Terminal", count: None, active: *state.bottom_tab.read() == BottomTab::Terminal, onclick: move |_| state.bottom_tab.set(BottomTab::Terminal) }
                div { class: "panel-actions",
                    button { title: "Collapse panel (Ctrl+J)", onclick: move |_| execute(state, ShellCommand::TogglePanel, backend.clone()), "×" }
                }
            }
            div { class: "bottom-content",
                match *state.bottom_tab.read() {
                    BottomTab::Problems => rsx! { ProblemsPanel { report } },
                    BottomTab::Output => rsx! { OutputPanel { report } },
                    BottomTab::Terminal => rsx! { TerminalPanel {} },
                }
            }
        }
    }
}

#[component]
fn ProblemsPanel(report: Option<ScanReport>) -> Element {
    rsx! {
        if let Some(value) = report {
            div { class: "problems-list", for finding in value.findings {
                div { class: "finding", span { class: "finding-icon", "!" } div { div { strong { "{finding.title}" } span { class: "badge", "{finding.severity}" } } p { "{finding.description}" } small { "→ {finding.recommendation}" } } }
            } }
        } else { div { class: "panel-empty", "No active findings. Choose a target and run analysis." } }
    }
}

#[component]
fn OutputPanel(report: Option<ScanReport>) -> Element {
    let state = use_context::<AppState>();
    rsx! { div { class: "console",
        p { span { class: "dim", "[shell]" } " VS Code-style workbench controls active" }
        p { span { class: "success", "[workspace]" } " {state.projects.read().len()} local projects indexed" }
        if let Some(value) = report { p { span { class: "accent", "[scan]" } " recon-probe completed for {value.target}" } }
    } }
}

#[component]
fn TerminalPanel() -> Element {
    rsx! { div { class: "terminal-reserved",
        div { class: "terminal-icon", ">_" }
        div { h3 { "Terminal integration is reserved" }
            p { "The panel is functional, but host command execution is disabled until the audited PTY service is implemented." }
            small { "Use Ctrl+` to focus this panel · no fake commands are executed" }
        }
    } }
}
