use dioxus::prelude::*;
use polyglid_desktop::client::{ClientGateway, ClientResult, ExecutionEvent, LocalClient};

use super::bottom_panel::BottomPanel;
use super::commands::{handle_shortcut, persist_shell};
use super::editor::EditorWorkspace;
use super::models::{LoadState, ResizeAxis};
use super::overlays::WorkspaceOverlays;
use super::shell::{ActivityRail, StatusBar};
use super::sidebar::WorkspaceSidebar;
use super::state::{push_activity, use_app_state, AppState};
use super::top_bar::TitleBar;

const APP_CSS: &str = concat!(
    include_str!("../../assets/theme.css"),
    include_str!("../../assets/main.css"),
    include_str!("../../assets/shell.css"),
    include_str!("../../assets/projects.css"),
);

#[component]
pub(crate) fn App() -> Element {
    let state = use_app_state();
    use_context_provider(|| state);
    let opened = use_hook(LocalClient::open_default);
    let client = match opened {
        Ok(client) => client,
        Err(error) => {
            return rsx! {
                style { dangerous_inner_html: APP_CSS }
                main { class: "startup-failure",
                    div { class: "state-panel error-state",
                        span { class: "state-icon", "!" }
                        h1 { "PolyGlid could not start" }
                        p { "{error}" }
                        small { "Check POLYGLID_DATA_DIR and POLYGLID_WORKSPACE_ROOT, then restart the desktop client." }
                    }
                }
            };
        }
    };
    use_context_provider(|| client.clone());

    load_bootstrap(state, client.clone());
    refresh_execution_data(state, client.clone());
    subscribe_to_executions(state, client.clone());

    let mouse_client = client.clone();
    let shortcut_client = client.clone();
    let mut shell_state = state;
    rsx! {
        style { dangerous_inner_html: APP_CSS }
        div {
            class: if state.shell.resizing.read().is_some() { "developer-space resizing" } else { "developer-space" },
            tabindex: 0,
            autofocus: true,
            onkeydown: move |event| handle_shortcut(event, state, shortcut_client.clone()),
            onmousemove: move |event| resize_shell(shell_state, event),
            onmouseup: move |_| finish_resize(state, mouse_client.clone()),
            TitleBar {}
            div { class: "workspace-body",
                ActivityRail {}
                if *state.shell.sidebar_visible.read() {
                    WorkspaceSidebar {}
                    div { class: "resize-handle vertical", onmousedown: move |_| shell_state.shell.resizing.set(Some(ResizeAxis::Sidebar)) }
                }
                div { class: "main-column",
                    EditorWorkspace {}
                    if *state.shell.bottom_panel_visible.read() {
                        div { class: "resize-handle horizontal", onmousedown: move |_| shell_state.shell.resizing.set(Some(ResizeAxis::BottomPanel)) }
                        BottomPanel {}
                    }
                }
            }
            StatusBar {}
            WorkspaceOverlays {}
        }
    }
}

fn load_bootstrap(mut state: AppState, client: LocalClient) {
    use_effect(move || {
        let refresh = *state.catalog.refresh.read();
        let _ = refresh;
        state.catalog.load.set(LoadState::Loading);
        let client = client.clone();
        spawn(async move {
            let result = tokio::task::spawn_blocking(move || client.bootstrap()).await;
            match result {
                Ok(Ok(snapshot)) => {
                    let load = if snapshot.projects.is_empty() {
                        LoadState::Empty
                    } else {
                        LoadState::Ready
                    };
                    let selected_plugin_is_valid =
                        state.plugins.selected_id.read().as_ref().is_some_and(|id| {
                            snapshot.plugins.iter().any(|plugin| &plugin.id == id)
                        });
                    let selected_target_is_valid = snapshot
                        .targets
                        .iter()
                        .any(|target| target.name == *state.runs.selected_target.read());
                    let selected_report_is_valid = state
                        .runs
                        .selected_report_id
                        .read()
                        .as_ref()
                        .is_some_and(|id| snapshot.reports.iter().any(|report| &report.id == id));

                    state
                        .catalog
                        .active_workspace_id
                        .set(Some(snapshot.active_workspace.id.clone()));
                    state
                        .catalog
                        .active_workspace_name
                        .set(snapshot.active_workspace.name.clone());
                    state.catalog.workspaces.set(snapshot.workspaces);
                    state.catalog.projects.set(snapshot.projects);
                    state.catalog.error.set(None);
                    state.catalog.load.set(load);

                    if !selected_plugin_is_valid {
                        state
                            .plugins
                            .selected_id
                            .set(snapshot.plugins.first().map(|plugin| plugin.id.clone()));
                    }
                    state.plugins.items.set(snapshot.plugins);

                    if !selected_target_is_valid && state.runs.selected_target.read().is_empty() {
                        if let Some(target) = snapshot.targets.first() {
                            state.runs.selected_target.set(target.name.clone());
                        }
                    }
                    state.runs.targets.set(snapshot.targets);
                    state.runs.executions.set(snapshot.executions);
                    if !selected_report_is_valid {
                        state
                            .runs
                            .selected_report_id
                            .set(snapshot.reports.first().map(|report| report.id.clone()));
                    }
                    state.runs.reports.set(snapshot.reports);

                    state
                        .shell
                        .sidebar_visible
                        .set(snapshot.shell.sidebar_visible);
                    state
                        .shell
                        .bottom_panel_visible
                        .set(snapshot.shell.bottom_panel_visible);
                    state.shell.sidebar_width.set(snapshot.shell.sidebar_width);
                    state
                        .shell
                        .bottom_panel_height
                        .set(snapshot.shell.bottom_panel_height);
                    push_activity(state, "Workspace and local services loaded");
                }
                Ok(Err(error)) => state.catalog.load.set(LoadState::Error(error.to_string())),
                Err(error) => state
                    .catalog
                    .load
                    .set(LoadState::Error(format!("bootstrap task failed: {error}"))),
            }
        });
    });
}

fn refresh_execution_data(mut state: AppState, client: LocalClient) {
    use_effect(move || {
        let refresh = *state.runs.refresh.read();
        let _ = refresh;
        let client = client.clone();
        spawn(async move {
            let result = tokio::task::spawn_blocking(move || -> ClientResult<_> {
                Ok((
                    client.list_executions()?,
                    client.list_reports()?,
                    client.list_targets()?,
                ))
            })
            .await;
            match result {
                Ok(Ok((executions, reports, targets))) => {
                    let current_active = *state.runs.active_job_id.read();
                    let active = current_active
                        .filter(|id| {
                            executions
                                .iter()
                                .any(|run| run.id == *id && !run.state.is_terminal())
                        })
                        .or_else(|| {
                            executions
                                .iter()
                                .find(|run| !run.state.is_terminal())
                                .map(|run| run.id)
                        });
                    state.runs.active_job_id.set(active);
                    state.runs.executions.set(executions);
                    state.runs.reports.set(reports);
                    state.runs.targets.set(targets);
                    state.runs.error.set(None);
                }
                Ok(Err(error)) => state.runs.error.set(Some(error.to_string())),
                Err(error) => state
                    .runs
                    .error
                    .set(Some(format!("refresh task failed: {error}"))),
            }
        });
    });
}

fn subscribe_to_executions(mut state: AppState, client: LocalClient) {
    use_effect(move || {
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        let subscription_client = client.clone();
        spawn(async move {
            let worker = tokio::task::spawn_blocking(move || -> ClientResult<()> {
                let mut subscription = subscription_client.subscribe_executions()?;
                loop {
                    let event = subscription.blocking_recv()?;
                    if sender.send(event).is_err() {
                        return Ok(());
                    }
                }
            });

            while let Some(event) = receiver.recv().await {
                apply_execution_event(state, event);
            }

            if let Ok(Err(error)) = worker.await {
                state.runs.error.set(Some(error.to_string()));
            }
        });
    });
}

fn apply_execution_event(mut state: AppState, event: ExecutionEvent) {
    let job_id = event.job_id();
    let message = match event {
        ExecutionEvent::StateChanged { state: phase, .. } => {
            if phase.is_terminal() && *state.runs.active_job_id.read() == Some(job_id) {
                state.runs.active_job_id.set(None);
            }
            format!("Execution {job_id} is {phase}")
        }
        ExecutionEvent::Finished {
            report, metrics, ..
        } => {
            if *state.runs.active_job_id.read() == Some(job_id) {
                state.runs.active_job_id.set(None);
            }
            format!(
                "Execution {job_id} completed for {} in {}ms",
                report.target_tested, metrics.duration_ms
            )
        }
        ExecutionEvent::Failed { error, .. } => {
            if *state.runs.active_job_id.read() == Some(job_id) {
                state.runs.active_job_id.set(None);
            }
            state.runs.error.set(Some(error.clone()));
            format!("Execution {job_id} failed: {error}")
        }
        ExecutionEvent::Log { message, .. } => format!("Execution {job_id}: {message}"),
    };
    push_activity(state, message);
    let next = *state.runs.refresh.read() + 1;
    state.runs.refresh.set(next);
}

fn resize_shell(mut state: AppState, event: MouseEvent) {
    match *state.shell.resizing.read() {
        Some(ResizeAxis::Sidebar) => {
            let width = (event.client_coordinates().x - 48.0).clamp(180.0, 480.0);
            state.shell.sidebar_width.set(width);
        }
        Some(ResizeAxis::BottomPanel) => {
            let window = dioxus::desktop::window();
            let viewport_height = window.inner_size().height as f64 / window.scale_factor();
            let height =
                (viewport_height - event.client_coordinates().y - 26.0).clamp(120.0, 520.0);
            state.shell.bottom_panel_height.set(height);
        }
        None => {}
    }
}

fn finish_resize(mut state: AppState, client: LocalClient) {
    if state.shell.resizing.read().is_some() {
        state.shell.resizing.set(None);
        persist_shell(state, client);
    }
}
