use dioxus::prelude::*;
use polyglid_desktop::client::{Plugin, PluginStatus};

use crate::ui::models::{capability_explanation, capability_risk};

#[component]
pub(crate) fn ScannerDashboard(
    target: String,
    selected_plugin: Option<String>,
    plugins: Vec<Plugin>,
    running: bool,
    on_target: EventHandler<String>,
    on_plugin: EventHandler<String>,
    on_review: EventHandler<MouseEvent>,
) -> Element {
    let selected = selected_plugin
        .as_ref()
        .and_then(|id| plugins.iter().find(|plugin| &plugin.id == id));
    let selected_enabled = selected.is_some_and(|plugin| plugin.status == PluginStatus::Enabled);
    let can_review = !target.trim().is_empty() && selected_enabled && !running;

    rsx! {
        div { class: "dashboard-scroll scanner-page",
            div { class: "page-heading centered",
                span { class: "eyebrow", "Permission-aware local execution" }
                h1 { "Start a security scan" }
                p { "Choose a saved target and installed component. PolyGlid will show every requested capability before anything runs." }
            }
            div { class: "scanner-card",
                label { class: "field-label", r#for: "scan-target", "Target domain or IP" }
                input {
                    id: "scan-target",
                    value: "{target}",
                    placeholder: "example.com",
                    oninput: move |event| on_target.call(event.value())
                }
                label { class: "field-label", r#for: "scan-plugin", "Execution component" }
                select {
                    id: "scan-plugin",
                    value: selected_plugin.as_deref().unwrap_or_default(),
                    onchange: move |event| on_plugin.call(event.value()),
                    option { value: "", disabled: true, "Choose a plugin" }
                    for plugin in &plugins {
                        option {
                            value: "{plugin.id}",
                            disabled: plugin.status != PluginStatus::Enabled,
                            "{plugin.name} · {plugin.id}"
                        }
                    }
                }

                if let Some(plugin) = selected {
                    div { class: "permission-review-summary scanner-permissions",
                        div { class: "permission-header",
                            div {
                                strong { "Requested permissions" }
                                p { "These are requests, not automatic grants." }
                            }
                            span { class: "badge", "{plugin.requested_capabilities.len()} requested" }
                        }
                        if plugin.requested_capabilities.is_empty() {
                            p { class: "muted", "This component requests no host capabilities." }
                        } else {
                            div { class: "permission-review-list compact",
                                for request in &plugin.requested_capabilities {
                                    div { class: "permission-item",
                                        span { class: "permission-icon", "◈" }
                                        div {
                                            code { "{request.capability}" }
                                            small { "{capability_explanation(request.capability)}" }
                                        }
                                        div { class: "permission-scope",
                                            span { class: "badge", "{capability_risk(request.capability)}" }
                                            small { "{request.scope}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if plugins.is_empty() {
                    p { class: "error-text", "Install and enable a WASM component before starting a scan." }
                } else if selected.is_some() && !selected_enabled {
                    p { class: "error-text", "This component is disabled. Enable it from Plugins." }
                }

                button {
                    class: "primary run-button",
                    disabled: !can_review,
                    onclick: move |event| on_review.call(event),
                    if running { "Execution in progress…" } else { "Review permissions →" }
                }
                p { class: "field-help", "No capability is granted until you approve it in the next step." }
            }
        }
    }
}
