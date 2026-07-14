use dioxus::prelude::*;

use crate::ui::models::PluginCard;

#[component]
pub(crate) fn PluginDashboard(
    plugins: Vec<PluginCard>,
    selected: String,
    on_toggle: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "dashboard-scroll",
            div { class: "page-heading", span { class: "eyebrow", "Component registry" } h1 { "Plugin management" } p { "Inspect capabilities and control which signed components may execute." } }
            div { class: "plugin-grid",
                for plugin in plugins {
                    article { class: if plugin.id == selected { "plugin-card selected" } else { "plugin-card" },
                        div { class: "plugin-card-head", span { class: if plugin.enabled { "plugin-symbol enabled" } else { "plugin-symbol" }, "◇" } span { class: if plugin.enabled { "badge good" } else { "badge" }, if plugin.enabled { "Enabled" } else { "Disabled" } } }
                        h2 { "{plugin.name}" } p { "{plugin.description}" } small { "{plugin.id} · v{plugin.version}" }
                        div { class: "capability-list", for capability in &plugin.capabilities { span { "{capability}" } } }
                        button { class: "secondary", onclick: { let id = plugin.id.to_string(); move |_| on_toggle.call(id.clone()) }, if plugin.enabled { "Disable component" } else { "Enable component" } }
                    }
                }
            }
        }
    }
}
