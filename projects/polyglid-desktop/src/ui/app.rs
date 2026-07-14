use dioxus::prelude::*;

use super::bottom_panel::BottomPanel;
use super::editor::EditorWorkspace;
use super::overlays::WorkspaceOverlays;
use super::shell::{ActivityRail, StatusBar};
use super::sidebar::WorkspaceSidebar;
use super::state::use_app_state;
use super::top_bar::TitleBar;

const APP_CSS: &str = concat!(
    include_str!("../../assets/theme.css"),
    include_str!("../../assets/main.css"),
);

#[component]
pub(crate) fn App() -> Element {
    let state = use_app_state();
    use_context_provider(|| state);
    rsx! {
        style { dangerous_inner_html: APP_CSS }
        div { class: "developer-space",
            TitleBar {}
            div { class: "workspace-body",
                ActivityRail {}
                WorkspaceSidebar {}
                div { class: "main-column", EditorWorkspace {} BottomPanel {} }
            }
            StatusBar {}
            WorkspaceOverlays {}
        }
    }
}
