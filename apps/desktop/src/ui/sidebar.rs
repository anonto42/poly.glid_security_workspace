use dioxus::prelude::*;
use polyglid_desktop::client::{ClientGateway, LocalClient, PluginStatus};

use super::commands::{execute, ShellCommand};
use super::models::{
    execution_state_class, DialogError, Overlay, PendingPluginInstall, WorkspaceView,
};
use super::state::{push_activity, refresh_operational_data, AppState};

#[component]
pub(crate) fn WorkspaceSidebar() -> Element {
    let state = use_context::<AppState>();
    let client = use_context::<LocalClient>();
    let active_view = *state.shell.active_view.read();
    rsx! {
        aside { class: "sidebar", style: "width: {state.shell.sidebar_width}px; flex-basis: {state.shell.sidebar_width}px",
            div { class: "sidebar-heading", span { "{active_view.title()}" }
                button { title: "Hide side bar (Ctrl+B)", aria_label: "Hide side bar", onclick: move |_| execute(state, ShellCommand::ToggleSidebar, client.clone()), "‹" }
            }
            match active_view {
                WorkspaceView::Projects => rsx! { ProjectsSidebar {} },
                WorkspaceView::Scanner => rsx! { ScannerSidebar {} },
                WorkspaceView::Executions => rsx! { ExecutionsSidebar {} },
                WorkspaceView::Reports => rsx! { ReportsSidebar {} },
                WorkspaceView::Plugins => rsx! { PluginsSidebar {} },
            }
        }
    }
}

#[component]
fn ProjectsSidebar() -> Element {
    let mut state = use_context::<AppState>();
    rsx! {
        div { class: "sidebar-section",
            p { class: "section-label", "Workspace" }
            div { class: "workspace-summary",
                span { class: "live-dot" }
                div { strong { "{state.catalog.active_workspace_name}" } small { "local catalog" } }
            }
        }
        div { class: "sidebar-section grow",
            p { class: "section-label", "Projects · {state.catalog.projects.read().len()}" }
            for project in state.catalog.projects.read().iter() {
                div { class: "project-nav", span { "◇" } div { strong { "{project.name}" } small { "{project.kind}" } } }
            }
            button { class: "sidebar-option", onclick: move |_| {
                let next = *state.catalog.refresh.read() + 1;
                state.catalog.refresh.set(next);
            }, span { "Refresh discovery" } small { "↻" } }
        }
    }
}

#[component]
fn ScannerSidebar() -> Element {
    let mut state = use_context::<AppState>();
    let client = use_context::<LocalClient>();
    let add_client = client.clone();
    let remove_client = client.clone();
    rsx! {
        div { class: "sidebar-section",
            p { class: "section-label", "Saved targets" }
            div { class: "add-row",
                input {
                    value: "{state.runs.new_target}",
                    placeholder: "Add domain or IP",
                    aria_label: "New target",
                    oninput: move |event| state.runs.new_target.set(event.value())
                }
                button {
                    title: "Save target",
                    disabled: state.runs.new_target.read().trim().is_empty(),
                    onclick: move |_| {
                        let target = state.runs.new_target.read().trim().to_string();
                        if target.is_empty() { return; }
                        let client = add_client.clone();
                        spawn(async move {
                            let operation_target = target.clone();
                            let result = tokio::task::spawn_blocking(move || client.add_target(&operation_target, None)).await;
                            match result {
                                Ok(Ok(saved)) => {
                                    if !state.runs.targets.read().iter().any(|item| item.name == saved.name) {
                                        state.runs.targets.write().push(saved);
                                    }
                                    state.runs.selected_target.set(target);
                                    state.runs.new_target.set(String::new());
                                    push_activity(state, "Saved a scan target");
                                }
                                Ok(Err(error)) => show_error(state, "Target could not be saved", error.to_string()),
                                Err(error) => show_error(state, "Target could not be saved", format!("target task failed: {error}")),
                            }
                        });
                    },
                    "+"
                }
            }
            div { class: "target-list",
                if state.runs.targets.read().is_empty() {
                    p { class: "muted", "No saved targets. You can also type one directly in New scan." }
                }
                for saved in state.runs.targets.read().iter() {
                    div { class: "target-row",
                        button {
                            class: if *state.runs.selected_target.read() == saved.name { "target active" } else { "target" },
                            onclick: {
                                let target = saved.name.clone();
                                move |_| state.runs.selected_target.set(target.clone())
                            },
                            span { "◎" }
                            span { "{saved.name}" }
                        }
                        button {
                            class: "target-remove",
                            title: "Remove saved target",
                            aria_label: "Remove {saved.name}",
                            onclick: {
                                let target = saved.name.clone();
                                let client = remove_client.clone();
                                move |_| {
                                    let target = target.clone();
                                    let client = client.clone();
                                    spawn(async move {
                                        let operation_target = target.clone();
                                        let result = tokio::task::spawn_blocking(move || client.remove_target(&operation_target)).await;
                                        match result {
                                            Ok(Ok(())) => state.runs.targets.write().retain(|item| item.name != target),
                                            Ok(Err(error)) => show_error(state, "Target could not be removed", error.to_string()),
                                            Err(error) => show_error(state, "Target could not be removed", format!("target task failed: {error}")),
                                        }
                                    });
                                }
                            },
                            "×"
                        }
                    }
                }
            }
        }
        div { class: "sidebar-section grow",
            p { class: "section-label", "Enabled components" }
            for plugin in state.plugins.items.read().iter().filter(|plugin| plugin.status == PluginStatus::Enabled) {
                button {
                    class: if state.plugins.selected_id.read().as_ref() == Some(&plugin.id) { "plugin-nav active" } else { "plugin-nav" },
                    onclick: {
                        let id = plugin.id.clone();
                        move |_| state.plugins.selected_id.set(Some(id.clone()))
                    },
                    span { class: "live-dot" }
                    div { strong { "{plugin.name}" } small { "{plugin.id}" } }
                }
            }
        }
    }
}

#[component]
fn ExecutionsSidebar() -> Element {
    let state = use_context::<AppState>();
    rsx! {
        div { class: "sidebar-section grow",
            p { class: "section-label", "Recent jobs · {state.runs.executions.read().len()}" }
            if state.runs.executions.read().is_empty() {
                p { class: "muted", "Execution history is empty." }
            }
            for run in state.runs.executions.read().iter().take(12) {
                div { class: "project-nav",
                    span { class: "status-dot {execution_state_class(run.state)}" }
                    div { strong { "{run.target}" } small { "{run.state} · {run.plugin_id}" } }
                }
            }
            button { class: "sidebar-option", onclick: move |_| refresh_operational_data(state), span { "Refresh history" } small { "↻" } }
        }
    }
}

#[component]
fn ReportsSidebar() -> Element {
    let mut state = use_context::<AppState>();
    rsx! {
        div { class: "sidebar-section grow",
            p { class: "section-label", "Saved reports · {state.runs.reports.read().len()}" }
            if state.runs.reports.read().is_empty() {
                p { class: "muted", "Completed scans will appear here." }
            }
            for report in state.runs.reports.read().iter() {
                button {
                    class: if state.runs.selected_report_id.read().as_ref() == Some(&report.id) { "project-nav active" } else { "project-nav" },
                    onclick: {
                        let id = report.id.clone();
                        move |_| state.runs.selected_report_id.set(Some(id.clone()))
                    },
                    span { "▥" }
                    div { strong { "{report.target}" } small { "{report.issues.len()} findings" } }
                }
            }
        }
    }
}

#[component]
fn PluginsSidebar() -> Element {
    let mut state = use_context::<AppState>();
    let client = use_context::<LocalClient>();
    let inspect_client = client.clone();
    rsx! {
        div { class: "sidebar-section",
            p { class: "section-label", "Install component" }
            div { class: "form-field",
                input {
                    value: "{state.plugins.install_path}",
                    placeholder: "/path/to/plugin.wasm",
                    aria_label: "WASM component path",
                    oninput: move |event| state.plugins.install_path.set(event.value()),
                    onkeydown: {
                        let client = inspect_client.clone();
                        move |event| {
                            if event.key().to_string() == "Enter" {
                                let path = state.plugins.install_path.read().trim().to_string();
                                if !path.is_empty() { inspect_plugin(state, client.clone(), path); }
                            }
                        }
                    }
                }
                button {
                    class: "secondary",
                    onclick: move |_| {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("WebAssembly Component", &["wasm"])
                            .set_title("Select a WASM plugin component")
                            .pick_file()
                        {
                            state.plugins.install_path.set(path.display().to_string());
                        }
                    },
                    "Browse"
                }
            }
            button {
                class: "primary small",
                disabled: state.plugins.install_path.read().trim().is_empty(),
                onclick: move |_| {
                    let path = state.plugins.install_path.read().trim().to_string();
                    if !path.is_empty() { inspect_plugin(state, client.clone(), path); }
                },
                "Validate and install…"
            }
        }
        div { class: "sidebar-section grow",
            p { class: "section-label", "Registry · {state.plugins.items.read().len()}" }
            for plugin in state.plugins.items.read().iter() {
                button {
                    class: if state.plugins.selected_id.read().as_ref() == Some(&plugin.id) { "plugin-nav active" } else { "plugin-nav" },
                    onclick: {
                        let id = plugin.id.clone();
                        move |_| state.plugins.selected_id.set(Some(id.clone()))
                    },
                    span { class: if plugin.status == PluginStatus::Enabled { "live-dot" } else { "live-dot off" } }
                    div { strong { "{plugin.name}" } small { "v{plugin.version} · {plugin.status}" } }
                }
            }
        }
    }
}

fn inspect_plugin(mut state: AppState, client: LocalClient, path: String) {
    spawn(async move {
        let inspected_path = path.clone();
        let result =
            tokio::task::spawn_blocking(move || client.inspect_plugin(&inspected_path)).await;
        match result {
            Ok(Ok(plugin)) => {
                state
                    .shell
                    .overlay
                    .set(Some(Overlay::PluginInstall(PendingPluginInstall {
                        path,
                        plugin,
                    })))
            }
            Ok(Err(error)) => show_error(state, "Component validation failed", error.to_string()),
            Err(error) => show_error(
                state,
                "Component validation failed",
                format!("validation task failed: {error}"),
            ),
        }
    });
}

fn show_error(mut state: AppState, title: impl Into<String>, message: impl Into<String>) {
    state.shell.overlay.set(Some(Overlay::Error(DialogError {
        title: title.into(),
        message: message.into(),
    })));
}
