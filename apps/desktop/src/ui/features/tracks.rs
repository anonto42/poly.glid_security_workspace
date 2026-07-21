use dioxus::prelude::*;
use polyglid_desktop::WorkspaceOverview;

use crate::ui::components::MetricCard;
use crate::ui::models::TrackFilter;

#[component]
pub(crate) fn TracksDashboard(
    overview: WorkspaceOverview,
    filter: TrackFilter,
    selected: Option<usize>,
    on_select: EventHandler<Option<usize>>,
) -> Element {
    let visible: Vec<_> = overview
        .tracks
        .iter()
        .cloned()
        .enumerate()
        .filter(|(_, track)| filter.matches(track.status))
        .collect();
    rsx! {
        div { class: "dashboard-scroll",
            div { class: "page-heading", span { class: "eyebrow", "Execution curriculum" } h1 { "Work tracks" } p { "Build the platform in ordered, verifiable stages with visible evidence and ownership." } }
            div { class: "metric-grid four",
                MetricCard { value: overview.tracks.len().to_string(), label: "tracks", tone: "neutral" }
                MetricCard { value: overview.total_tasks().to_string(), label: "tasks", tone: "neutral" }
                MetricCard { value: overview.active_tracks().to_string(), label: "active", tone: "good" }
                MetricCard { value: overview.completed_tasks().to_string(), label: "completed", tone: "neutral" }
            }
            div { class: "track-list",
                for (index, track) in visible {
                    article { class: if selected == Some(index) { "track-card selected" } else { "track-card" },
                        button { class: "track-main", onclick: move |_| on_select.call(if selected == Some(index) { None } else { Some(index) }),
                            span { class: "track-number", "{track.sequence:02}" }
                            div { class: "track-body",
                                div { class: "track-meta", span { "Platform track" } span { class: "badge", "{track.status.label()}" } }
                                h2 { "{track.title}" } p { "{track.description}" }
                                div { class: "capability-list", for tag in &track.tags { span { "#{tag}" } } }
                                div { class: "progress-label", span { "{track.completed_tasks} / {track.total_tasks} tasks" } span { "{track.progress_percent()}%" } }
                                div { class: "progress", div { style: "width: {track.progress_percent()}%" } }
                            }
                            span { class: "track-arrow", if selected == Some(index) { "−" } else { "→" } }
                        }
                        if selected == Some(index) { div { class: "track-detail", span { "Next action" } strong { "Open the track plan and complete its verification gate." } button { class: "secondary", "View tasks" } } }
                    }
                }
            }
        }
    }
}
