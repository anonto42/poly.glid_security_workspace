use dioxus::prelude::*;

use crate::ui::components::{BarRow, MetricCard};
use crate::ui::models::{PluginCard, ScanReport};
use crate::ui::preview::PLUGIN_SOURCE;

#[component]
pub(crate) fn ScannerDashboard(
    target: String,
    selected_plugin: String,
    plugins: Vec<PluginCard>,
    on_target: EventHandler<String>,
    on_plugin: EventHandler<String>,
    on_run: EventHandler<MouseEvent>,
) -> Element {
    let state = use_context::<crate::ui::state::AppState>();
    let selected_enabled = plugins
        .iter()
        .find(|plugin| plugin.id == selected_plugin)
        .is_none_or(|plugin| plugin.enabled);
    rsx! {
        div { class: "dashboard-scroll scanner-page",
            div { class: "page-heading centered", span { class: "eyebrow", "Sandboxed execution" } h1 { "Security scanner" } p { "Configure a target and launch a permission-controlled WebAssembly component." } }
            div { class: "scanner-card",
                label { class: "field-label", "Target domain or IP" }
                input { value: "{target}", placeholder: "example.com", oninput: move |event| on_target.call(event.value()) }
                label { class: "field-label", "Selected plugin" }
                select { value: "{selected_plugin}", onchange: move |event| on_plugin.call(event.value()),
                    for plugin in plugins { option { value: "{plugin.id}", "{plugin.name} · {plugin.id}" } }
                }
                div { class: "permission-strip", span { "◈ WASI sandbox" } span { "◎ scoped DNS" } span { "▣ report write" } }
                if !selected_enabled { p { class: "error-text", "This plugin is disabled. Enable it from Plugin management." } }
                if let Some(error) = state.execution_error.read().as_ref() { p { class: "error-text", "{error}" } }
                button { class: "primary run-button", disabled: !selected_enabled || *state.execution_running.read(), onclick: move |event| on_run.call(event),
                    if *state.execution_running.read() { "Running…" } else { "▶ Run analysis" }
                }
            }
        }
    }
}

#[component]
pub(crate) fn ResultDashboard(report: Option<ScanReport>) -> Element {
    rsx! {
        div { class: "dashboard-scroll result-page",
            if let Some(value) = report {
                div { class: "result-hero", div { span { class: "eyebrow", "Analysis complete" } h1 { "{value.target}" } p { "{value.summary}" } } div { class: "risk-score", strong { "82" } span { "health score" } } }
                div { class: "metric-grid three",
                    MetricCard { value: value.findings.len().to_string(), label: "observations", tone: "warning" }
                    MetricCard { value: "0".to_string(), label: "critical", tone: "good" }
                    MetricCard { value: "148ms".to_string(), label: "runtime", tone: "neutral" }
                }
                div { class: "chart-card", div { class: "card-heading", strong { "Finding distribution" } span { "by confidence" } } BarRow { label: "Informational", value: 74, amount: "74%" } BarRow { label: "Low", value: 42, amount: "42%" } BarRow { label: "Medium", value: 18, amount: "18%" } }
            } else { div { class: "empty-state", "Run an analysis to create a result dashboard." } }
        }
    }
}

#[component]
pub(crate) fn SourceDashboard() -> Element {
    rsx! { div { class: "source-view", div { class: "source-note", "Read-only · projects/recon-probe/src/lib.rs" } pre { code { "{PLUGIN_SOURCE}" } } } }
}
