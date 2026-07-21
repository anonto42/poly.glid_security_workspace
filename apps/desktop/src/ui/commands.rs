use dioxus::prelude::*;
use polyglid_desktop::client::{ClientGateway, LocalClient, ShellPreferences};

use super::models::{Overlay, WorkspaceView};
use super::state::{activate_view, close_view, AppState};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ShellCommand {
    OpenView(WorkspaceView),
    ToggleSidebar,
    TogglePanel,
    OpenSettings,
}

#[derive(Clone, Copy)]
pub(crate) struct CommandDefinition {
    pub(crate) title: &'static str,
    pub(crate) category: &'static str,
    pub(crate) shortcut: &'static str,
    pub(crate) action: ShellCommand,
}

pub(crate) const COMMANDS: &[CommandDefinition] = &[
    command(
        "Open Projects",
        "View",
        "Ctrl+1",
        ShellCommand::OpenView(WorkspaceView::Projects),
    ),
    command(
        "Start a Scan",
        "View",
        "Ctrl+2",
        ShellCommand::OpenView(WorkspaceView::Scanner),
    ),
    command(
        "Open Executions",
        "View",
        "Ctrl+3",
        ShellCommand::OpenView(WorkspaceView::Executions),
    ),
    command(
        "Open Reports",
        "View",
        "Ctrl+4",
        ShellCommand::OpenView(WorkspaceView::Reports),
    ),
    command(
        "Open Plugins",
        "View",
        "Ctrl+5",
        ShellCommand::OpenView(WorkspaceView::Plugins),
    ),
    command(
        "Toggle Primary Side Bar",
        "View",
        "Ctrl+B",
        ShellCommand::ToggleSidebar,
    ),
    command(
        "Toggle Activity Panel",
        "View",
        "Ctrl+J",
        ShellCommand::TogglePanel,
    ),
    command(
        "Open Settings",
        "Preferences",
        "",
        ShellCommand::OpenSettings,
    ),
];

const fn command(
    title: &'static str,
    category: &'static str,
    shortcut: &'static str,
    action: ShellCommand,
) -> CommandDefinition {
    CommandDefinition {
        title,
        category,
        shortcut,
        action,
    }
}

pub(crate) fn execute(mut state: AppState, action: ShellCommand, client: LocalClient) {
    let persist = matches!(
        action,
        ShellCommand::ToggleSidebar | ShellCommand::TogglePanel
    );
    match action {
        ShellCommand::OpenView(view) => activate_view(state, view),
        ShellCommand::ToggleSidebar => state.shell.sidebar_visible.toggle(),
        ShellCommand::TogglePanel => state.shell.bottom_panel_visible.toggle(),
        ShellCommand::OpenSettings => state.shell.overlay.set(Some(Overlay::Settings)),
    }
    if !matches!(action, ShellCommand::OpenSettings) {
        state.shell.overlay.set(None);
    }
    if persist {
        persist_shell(state, client);
    }
}

pub(crate) fn handle_shortcut(event: KeyboardEvent, mut state: AppState, client: LocalClient) {
    let key = event.key().to_string().to_lowercase();
    let modifiers = event.modifiers();
    let primary = modifiers.ctrl() || modifiers.meta();
    let action = match key.as_str() {
        "escape" => {
            event.prevent_default();
            state.shell.overlay.set(None);
            return;
        }
        "f1" => {
            event.prevent_default();
            state.shell.overlay.set(Some(Overlay::Commands));
            return;
        }
        "p" if primary => {
            event.prevent_default();
            state.shell.overlay.set(Some(Overlay::Commands));
            return;
        }
        "b" if primary => Some(ShellCommand::ToggleSidebar),
        "j" if primary => Some(ShellCommand::TogglePanel),
        "1" if primary => Some(ShellCommand::OpenView(WorkspaceView::Projects)),
        "2" if primary => Some(ShellCommand::OpenView(WorkspaceView::Scanner)),
        "3" if primary => Some(ShellCommand::OpenView(WorkspaceView::Executions)),
        "4" if primary => Some(ShellCommand::OpenView(WorkspaceView::Reports)),
        "5" if primary => Some(ShellCommand::OpenView(WorkspaceView::Plugins)),
        "w" if primary => {
            event.prevent_default();
            let active = *state.shell.active_view.read();
            close_view(state, active);
            None
        }
        _ => None,
    };
    if let Some(action) = action {
        event.prevent_default();
        execute(state, action, client);
    }
}

pub(crate) fn persist_shell(state: AppState, client: LocalClient) {
    let preferences = ShellPreferences {
        sidebar_visible: *state.shell.sidebar_visible.read(),
        bottom_panel_visible: *state.shell.bottom_panel_visible.read(),
        sidebar_width: *state.shell.sidebar_width.read(),
        bottom_panel_height: *state.shell.bottom_panel_height.read(),
    };
    spawn(async move {
        let _ =
            tokio::task::spawn_blocking(move || client.save_shell_preferences(&preferences)).await;
    });
}
