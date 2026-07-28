use dioxus::prelude::*;

use super::features::ProjectsDashboard;

#[component]
pub(crate) fn EditorWorkspace() -> Element {
    rsx! {
        main { class: "editor",
            div { class: "workbench-tabs", role: "tablist", aria_label: "Open views",
                button {
                    class: "workbench-tab active",
                    role: "tab",
                    aria_selected: true,
                    span { class: "tab-icon", "▦" }
                    span { "Projects" }
                }
            }
            div { class: "editor-surface", ProjectsDashboard {} }
        }
    }
}
