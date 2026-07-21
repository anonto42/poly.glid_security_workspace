use dioxus::prelude::*;
use polyglid_desktop::client::{Plugin, PluginStatus};

#[component]
pub(crate) fn PluginDashboard(
    plugins: Vec<Plugin>,
    selected: Option<String>,
    on_select: EventHandler<String>,
    on_toggle: EventHandler<String>,
    on_uninstall: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "dashboard-scroll",
            div { class: "page-heading", span { class: "eyebrow", "Component registry" } h1 { "Plugin management" } p { "Inspect capabilities and control which signed components may execute." } }
            if plugins.is_empty() {
                div { class: "state-panel empty-state",
                    span { class: "state-icon", "◇" }
                    h2 { "No components installed" }
                    p { "Choose a signed WASM component from the side bar. PolyGlid validates its manifest before installation." }
                }
            } else {
                div { class: "plugin-grid",
                    for plugin in plugins {
                        article {
                            class: if selected.as_ref().is_some_and(|id| id == &plugin.id) { "plugin-card selected" } else { "plugin-card" },
                            onclick: {
                                let id = plugin.id.clone();
                                move |_| on_select.call(id.clone())
                            },
                            div { class: "plugin-card-head",
                                span { class: if plugin.status == PluginStatus::Enabled { "plugin-symbol enabled" } else { "plugin-symbol" }, "◇" }
                                span { class: if plugin.status == PluginStatus::Enabled { "badge good" } else { "badge" }, "{plugin.status}" }
                            }
                            h2 { "{plugin.name}" }
                            p { "{plugin.description}" }
                            small { "{plugin.id} · v{plugin.version} · {plugin.author}" }
                            p { class: "muted", "Source: {plugin.source}" }
                            div { class: "capability-list",
                                if plugin.capabilities.is_empty() {
                                    span { "No host capabilities" }
                                }
                                for capability in &plugin.capabilities { span { "{capability}" } }
                            }
                            div { class: "plugin-actions",
                                button {
                                    class: "secondary",
                                    onclick: {
                                        let id = plugin.id.clone();
                                        move |event| { event.stop_propagation(); on_toggle.call(id.clone()); }
                                    },
                                    if plugin.status == PluginStatus::Enabled { "Disable component" } else { "Enable component" }
                                }
                                button {
                                    class: "danger-button",
                                    onclick: {
                                        let id = plugin.id.clone();
                                        move |event| { event.stop_propagation(); on_uninstall.call(id.clone()); }
                                    },
                                    "Uninstall"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
