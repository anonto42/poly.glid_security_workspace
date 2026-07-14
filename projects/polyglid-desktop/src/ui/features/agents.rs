use dioxus::prelude::*;

use crate::ui::components::AgentCard;

#[component]
pub(crate) fn AgentsDashboard() -> Element {
    rsx! {
        div { class: "dashboard-scroll agents-page",
            div { class: "page-heading", span { class: "eyebrow", "Human-controlled assistance" } h1 { "AI agent workspace" } p { "Delegate bounded work while the executive preserves context, approvals, and evidence." } }
            div { class: "agent-layout",
                section { class: "agent-roster",
                    AgentCard { initials: "EX", name: "Executive", role: "Plans, routes, and verifies work", state: "Online", active: true }
                    AgentCard { initials: "CA", name: "Code analyst", role: "Maps code and explains behavior", state: "Idle", active: false }
                    AgentCard { initials: "SR", name: "Security reviewer", role: "Finds risks and validates boundaries", state: "Idle", active: false }
                    AgentCard { initials: "TH", name: "Test helper", role: "Runs focused verification", state: "Idle", active: false }
                }
                section { class: "agent-console",
                    div { class: "agent-console-head", div { span { class: "agent-avatar active", "EX" } strong { "Executive agent" } } span { class: "badge good", "Ready" } }
                    div { class: "conversation",
                        div { class: "message system", span { "System" } p { "Workspace context loaded: architecture, plans, recent changes, and verification rules." } }
                        div { class: "message assistant", span { "Executive" } p { "Choose a bounded task. I will create a plan, route specialist work, and require evidence before completion." } }
                    }
                    div { class: "suggestions", button { "Analyze current phase" } button { "Review security" } button { "Prepare next task" } }
                    div { class: "composer", textarea { placeholder: "Describe the outcome you want…" } button { "Send ↗" } }
                }
            }
        }
    }
}
