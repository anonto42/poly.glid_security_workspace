use dioxus::prelude::*;

use crate::backend::{DesktopBackend, ShellPreferences};

use super::models::{BottomTab, WorkspaceView};
use super::state::{activate_view, close_view, AppState};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ShellCommand {
    OpenView(WorkspaceView),
    ToggleSidebar,
    TogglePanel,
    FocusTerminal,
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
        "Open My Projects",
        "View",
        "Ctrl+1",
        ShellCommand::OpenView(WorkspaceView::Projects),
    ),
    command(
        "Open Scanner",
        "View",
        "Ctrl+2",
        ShellCommand::OpenView(WorkspaceView::Explorer),
    ),
    command(
        "Open Plugins",
        "View",
        "Ctrl+3",
        ShellCommand::OpenView(WorkspaceView::Plugins),
    ),
    command(
        "Open Work Tracks",
        "View",
        "Ctrl+4",
        ShellCommand::OpenView(WorkspaceView::Tracks),
    ),
    command(
        "Open Automation",
        "View",
        "Ctrl+5",
        ShellCommand::OpenView(WorkspaceView::Automation),
    ),
    command(
        "Open AI Agents",
        "View",
        "Ctrl+6",
        ShellCommand::OpenView(WorkspaceView::Agents),
    ),
    command(
        "Toggle Primary Side Bar",
        "View",
        "Ctrl+B",
        ShellCommand::ToggleSidebar,
    ),
    command("Toggle Panel", "View", "Ctrl+J", ShellCommand::TogglePanel),
    command(
        "Focus Terminal",
        "Terminal",
        "Ctrl+`",
        ShellCommand::FocusTerminal,
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

pub(crate) fn execute(mut state: AppState, action: ShellCommand, backend: DesktopBackend) {
    let persist = matches!(
        action,
        ShellCommand::ToggleSidebar | ShellCommand::TogglePanel | ShellCommand::FocusTerminal
    );
    match action {
        ShellCommand::OpenView(view) => activate_view(state, view),
        ShellCommand::ToggleSidebar => state.sidebar_visible.toggle(),
        ShellCommand::TogglePanel => state.bottom_panel_visible.toggle(),
        ShellCommand::FocusTerminal => {
            state.bottom_panel_visible.set(true);
            state.bottom_tab.set(BottomTab::Terminal);
        }
        ShellCommand::OpenSettings => state.settings_open.set(true),
    }
    state.command_open.set(false);
    if persist {
        persist_shell(state, backend);
    }
}

pub(crate) fn handle_shortcut(event: KeyboardEvent, mut state: AppState, backend: DesktopBackend) {
    let key = event.key().to_string().to_lowercase();
    let modifiers = event.modifiers();
    let primary = modifiers.ctrl() || modifiers.meta();
    let action = match key.as_str() {
        "escape" => {
            event.prevent_default();
            state.command_open.set(false);
            state.settings_open.set(false);
            return;
        }
        "f1" => {
            event.prevent_default();
            state.command_open.set(true);
            return;
        }
        "p" if primary && modifiers.shift() => {
            event.prevent_default();
            state.command_open.set(true);
            return;
        }
        "p" if primary => {
            event.prevent_default();
            state.command_open.set(true);
            return;
        }
        "b" if primary => Some(ShellCommand::ToggleSidebar),
        "j" if primary => Some(ShellCommand::TogglePanel),
        "`" if primary => Some(ShellCommand::FocusTerminal),
        "1" if primary => Some(ShellCommand::OpenView(WorkspaceView::Projects)),
        "2" if primary => Some(ShellCommand::OpenView(WorkspaceView::Explorer)),
        "3" if primary => Some(ShellCommand::OpenView(WorkspaceView::Plugins)),
        "4" if primary => Some(ShellCommand::OpenView(WorkspaceView::Tracks)),
        "5" if primary => Some(ShellCommand::OpenView(WorkspaceView::Automation)),
        "6" if primary => Some(ShellCommand::OpenView(WorkspaceView::Agents)),
        "w" if primary => {
            event.prevent_default();
            let active = *state.active_view.read();
            close_view(state, active);
            None
        }
        _ => None,
    };
    if let Some(action) = action {
        event.prevent_default();
        execute(state, action, backend);
    }
}

pub(crate) fn persist_shell(state: AppState, backend: DesktopBackend) {
    let value = ShellPreferences {
        sidebar_visible: *state.sidebar_visible.read(),
        bottom_panel_visible: *state.bottom_panel_visible.read(),
        sidebar_width: *state.sidebar_width.read(),
        bottom_panel_height: *state.bottom_panel_height.read(),
    };
    spawn(async move {
        let _ = tokio::task::spawn_blocking(move || backend.save_shell_preferences(&value)).await;
    });
}
