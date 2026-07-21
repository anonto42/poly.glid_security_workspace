use std::fs;

use dioxus::prelude::*;
use polyglid_desktop::client::{ClientGateway, LocalClient, Report, ReportFormat, Severity};

use crate::ui::models::{DialogError, Overlay};
use crate::ui::state::AppState;

#[component]
pub(crate) fn ReportsDashboard() -> Element {
    let mut state = use_context::<AppState>();
    let reports = state.runs.reports.read().clone();
    let selected_id = state.runs.selected_report_id.read().clone();
    let selected = selected_id
        .as_ref()
        .and_then(|id| reports.iter().find(|report| &report.id == id))
        .cloned()
        .or_else(|| reports.first().cloned());

    rsx! {
        div { class: "dashboard-scroll result-page",
            div { class: "page-heading",
                span { class: "eyebrow", "Persisted evidence" }
                h1 { "Reports" }
                p { "Review findings produced by completed local executions and export them in standard formats." }
            }
            if reports.is_empty() {
                div { class: "state-panel empty-state",
                    span { class: "state-icon", "▥" }
                    h2 { "No reports yet" }
                    p { "A successful scan creates a report here automatically." }
                }
            } else {
                div { class: "report-workspace",
                    nav { class: "report-list", aria_label: "Saved reports",
                        for report in &reports {
                            button {
                                class: if selected.as_ref().is_some_and(|value| value.id == report.id) { "report-item selected" } else { "report-item" },
                                onclick: {
                                    let id = report.id.clone();
                                    move |_| state.runs.selected_report_id.set(Some(id.clone()))
                                },
                                div { class: "report-summary",
                                    div { class: "report-title", span { class: "status-dot completed" } div { strong { "{report.target}" } p { "{report.plugin_id}" } } }
                                    span { class: "status-badge completed", "{report.issues.len()} findings" }
                                }
                                small { "{report.summary}" }
                            }
                        }
                    }
                    if let Some(report) = selected {
                        ReportDetail { report }
                    }
                }
            }
        }
    }
}

#[component]
fn ReportDetail(report: Report) -> Element {
    let critical = report
        .issues
        .iter()
        .filter(|issue| issue.severity == Severity::Critical)
        .count();
    let high = report
        .issues
        .iter()
        .filter(|issue| issue.severity == Severity::High)
        .count();

    rsx! {
        section { class: "surface-card report-detail",
            div { class: "report-summary",
                div {
                    span { class: "eyebrow", "Completed report" }
                    h2 { "{report.target}" }
                    p { "{report.summary}" }
                }
                div { class: "report-actions",
                    ExportButton { report_id: report.id.clone(), format: ReportFormat::Json }
                    ExportButton { report_id: report.id.clone(), format: ReportFormat::Markdown }
                    ExportButton { report_id: report.id.clone(), format: ReportFormat::Sarif }
                }
            }
            div { class: "report-meta",
                span { "Plugin: {report.plugin_id}" }
                span { "Job: {report.job_id}" }
                span { "Created: {report.created_at}" }
                span { "Critical: {critical}" }
                span { "High: {high}" }
            }
            if report.issues.is_empty() {
                div { class: "state-panel", h3 { "No findings" } p { "The component completed without reporting an issue." } }
            } else {
                div { class: "findings-list",
                    for issue in &report.issues {
                        article { class: "finding severity-{issue.severity}",
                            span { class: "severity", "{issue.severity}" }
                            div {
                                strong { "{issue.title}" }
                                p { "{issue.description}" }
                                small { "Recommendation: {issue.recommendation}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ExportButton(report_id: String, format: ReportFormat) -> Element {
    let mut state = use_context::<AppState>();
    let client = use_context::<LocalClient>();
    rsx! {
        button {
            class: "secondary",
            onclick: move |_| {
                let Some(path) = rfd::FileDialog::new()
                    .set_file_name(format!("polyglid-report.{}", extension(format)))
                    .save_file()
                else { return; };
                let client = client.clone();
                let report_id = report_id.clone();
                spawn(async move {
                    let result = tokio::task::spawn_blocking(move || {
                        let payload = client.export_report(&report_id, format)?;
                        fs::write(&path, payload).map_err(|error| {
                            polyglid_desktop::client::ClientError::Operation {
                                operation: "export report",
                                message: error.to_string(),
                            }
                        })?;
                        Ok::<_, polyglid_desktop::client::ClientError>(path)
                    }).await;
                    match result {
                        Ok(Ok(path)) => state.runs.activity.write().push(format!("Exported report to {}", path.display())),
                        Ok(Err(error)) => state.shell.overlay.set(Some(Overlay::Error(DialogError {
                            title: "Export failed".to_string(),
                            message: error.to_string(),
                        }))),
                        Err(error) => state.shell.overlay.set(Some(Overlay::Error(DialogError {
                            title: "Export failed".to_string(),
                            message: format!("export task failed: {error}"),
                        }))),
                    }
                });
            },
            "Export {format}"
        }
    }
}

fn extension(format: ReportFormat) -> &'static str {
    match format {
        ReportFormat::Json => "json",
        ReportFormat::Markdown => "md",
        ReportFormat::Html => "html",
        ReportFormat::Sarif => "sarif",
    }
}
