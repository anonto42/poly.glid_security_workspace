use dioxus::prelude::*;
use polyglid_desktop::client::{Execution, JobId};

use crate::ui::models::execution_state_class;

#[component]
pub(crate) fn ExecutionsDashboard(
    executions: Vec<Execution>,
    active_job_id: Option<JobId>,
    on_cancel: EventHandler<JobId>,
    on_open_report: EventHandler<JobId>,
    on_refresh: EventHandler<MouseEvent>,
) -> Element {
    let active = active_job_id.and_then(|id| executions.iter().find(|run| run.id == id));

    rsx! {
        div { class: "dashboard-scroll",
            div { class: "projects-hero",
                div { class: "page-heading",
                    span { class: "eyebrow", "Local job history" }
                    h1 { "Executions" }
                    p { "Follow queued and running work, cancel active jobs, and open persisted results." }
                }
                button { class: "secondary", onclick: move |event| on_refresh.call(event), "Refresh" }
            }

            if let Some(run) = active {
                ActiveExecution { run: run.clone(), on_cancel }
            }

            if executions.is_empty() {
                div { class: "state-panel empty-state",
                    span { class: "state-icon", "▷" }
                    h2 { "No executions yet" }
                    p { "Open New scan, choose a target and component, then approve its requested permissions." }
                }
            } else {
                div { class: "report-list",
                    for run in executions {
                        ExecutionRow {
                            key: "{run.id}",
                            run,
                            active_job_id,
                            on_cancel,
                            on_open_report,
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ActiveExecution(run: Execution, on_cancel: EventHandler<JobId>) -> Element {
    rsx! {
        section { class: "execution-status surface-card", "data-status": execution_state_class(run.state),
            div { class: "execution-status-header",
                div { class: "execution-status-title",
                    span { class: "status-dot running" }
                    div {
                        h2 { "{run.plugin_id} is {run.state}" }
                        p { "Target: {run.target}" }
                    }
                }
                span { class: "status-badge {execution_state_class(run.state)}", "{run.state}" }
            }
            div { class: "execution-progress",
                div { class: "execution-progress-label", span { "Sandbox job" } span { "{short_job_id(run.id)}" } }
                div { class: "execution-progress-track", div { class: "execution-progress-bar indeterminate" } }
            }
            div { class: "execution-actions",
                button { class: "danger-button", onclick: move |_| on_cancel.call(run.id), "Cancel execution" }
            }
        }
    }
}

#[component]
fn ExecutionRow(
    run: Execution,
    active_job_id: Option<JobId>,
    on_cancel: EventHandler<JobId>,
    on_open_report: EventHandler<JobId>,
) -> Element {
    let is_active = active_job_id == Some(run.id) && !run.state.is_terminal();
    let fuel_label = run
        .fuel_consumed
        .map_or_else(|| "not reported".to_string(), |value| value.to_string());
    rsx! {
        article { class: "report-item",
            div { class: "report-summary",
                div { class: "report-title",
                    span { class: "status-dot {execution_state_class(run.state)}" }
                    div {
                        strong { "{run.target}" }
                        p { "{run.plugin_id} · {short_job_id(run.id)}" }
                    }
                }
                span { class: "status-badge {execution_state_class(run.state)}", "{run.state}" }
            }
            div { class: "report-meta",
                span { "Duration: {duration_label(run.duration_ms)}" }
                span { "Fuel: {fuel_label}" }
                span { "Created: {run.created_at}" }
            }
            if let Some(error) = &run.error {
                p { class: "field-error", "{error}" }
            }
            div { class: "report-actions",
                if is_active {
                    button { class: "danger-button", onclick: move |_| on_cancel.call(run.id), "Cancel" }
                }
                if run.report.is_some() {
                    button { class: "secondary", onclick: move |_| on_open_report.call(run.id), "Open report" }
                }
            }
        }
    }
}

fn duration_label(milliseconds: u64) -> String {
    if milliseconds >= 1_000 {
        format!("{:.2}s", milliseconds as f64 / 1_000.0)
    } else {
        format!("{milliseconds}ms")
    }
}

fn short_job_id(id: JobId) -> String {
    id.to_string().chars().take(8).collect()
}
