use dioxus::prelude::*;

use crate::ui::components::{MetricCard, PipelineCard};

#[component]
pub(crate) fn AutomationDashboard() -> Element {
    rsx! {
        div { class: "dashboard-scroll",
            div { class: "page-heading", span { class: "eyebrow", "Deterministic execution" } h1 { "Automation control" } p { "Run typed workspace workflows and keep evidence for every command." } }
            div { class: "metric-grid four",
                MetricCard { value: "4".to_string(), label: "pipelines", tone: "neutral" }
                MetricCard { value: "12".to_string(), label: "checks", tone: "neutral" }
                MetricCard { value: "11".to_string(), label: "passing", tone: "good" }
                MetricCard { value: "1".to_string(), label: "waiting", tone: "warning" }
            }
            div { class: "pipeline-grid",
                PipelineCard { icon: "✓", title: "Workspace verify", description: "Contracts, permissions, formatting, tests, and dependency boundaries.", state: "Ready", steps: "6 steps" }
                PipelineCard { icon: "Rs", title: "Rust quality", description: "Format, check, test, Clippy, audit, and build evidence.", state: "Passing", steps: "4 steps" }
                PipelineCard { icon: "◈", title: "Security review", description: "Threat review, unsafe scan, dependency audit, and signed report.", state: "Review", steps: "3 steps" }
                PipelineCard { icon: "↗", title: "Release gate", description: "Package artifacts only after required evidence and approvals exist.", state: "Draft", steps: "5 steps" }
            }
        }
    }
}
