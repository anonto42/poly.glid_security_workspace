use dioxus::prelude::*;
use polyglid_desktop::controllers::DesktopControllers;

use super::commands::toggle_sidebar;
use super::models::LoadState;
use super::state::AppState;

#[component]
pub(crate) fn ActivityRail() -> Element {
    rsx! {
        nav { class: "activity-rail", aria_label: "PolyGlid product areas",
            button {
                class: "rail-button active",
                title: "Projects",
                aria_current: "page",
                span { "▦" }
                small { "Projects" }
            }
        }
    }
}

#[component]
pub(crate) fn StatusBar() -> Element {
    let state = use_context::<AppState>();
    let controllers = use_context::<DesktopControllers>();
    let catalog_status = match &*state.catalog.load.read() {
        LoadState::Loading => "Catalog indexing",
        LoadState::Error(_) => "Catalog error",
        LoadState::Empty => "Catalog empty",
        LoadState::Ready => "Catalog ready",
    };
    rsx! {
        footer { class: "statusbar",
            div { span { "◈" } " {catalog_status}" }
            div { class: "status-spacer" }
            div { "{state.catalog.projects.read().len()} projects" }
            button {
                class: "status-control",
                title: "Toggle project sidebar (Ctrl+B)",
                aria_label: "Toggle project sidebar",
                onclick: move |_| toggle_sidebar(state, controllers.clone()),
                "▤"
            }
        }
    }
}
