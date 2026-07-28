use dioxus::prelude::*;
use polyglid_desktop::client::ShellPreferences;
use polyglid_desktop::controllers::DesktopControllers;

use super::state::AppState;

pub(crate) fn handle_shortcut(
    event: KeyboardEvent,
    state: AppState,
    controllers: DesktopControllers,
) {
    let key = event.key().to_string().to_lowercase();
    let modifiers = event.modifiers();
    if (modifiers.ctrl() || modifiers.meta()) && key == "b" {
        event.prevent_default();
        toggle_sidebar(state, controllers);
    }
}

pub(crate) fn toggle_sidebar(mut state: AppState, controllers: DesktopControllers) {
    state.shell.sidebar_visible.toggle();
    persist_shell(state, controllers);
}

pub(crate) fn persist_shell(mut state: AppState, controllers: DesktopControllers) {
    let preferences = ShellPreferences {
        sidebar_visible: *state.shell.sidebar_visible.read(),
        bottom_panel_visible: false,
        sidebar_width: *state.shell.sidebar_width.read(),
        bottom_panel_height: 210.0,
    };
    spawn(async move {
        let settings = controllers.settings;
        let result = tokio::task::spawn_blocking(move || settings.save_shell(&preferences)).await;
        match result {
            Ok(Ok(())) => {}
            Ok(Err(error)) => state.catalog.error.set(Some(error.to_string())),
            Err(error) => state
                .catalog
                .error
                .set(Some(format!("settings task failed: {error}"))),
        }
    });
}
