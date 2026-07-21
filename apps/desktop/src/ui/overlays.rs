use std::time::Duration;

use dioxus::prelude::*;
use polyglid_desktop::client::{
    CapabilityKind, ClientGateway, LocalClient, PluginStatus, StartExecutionRequest,
};

use super::commands::{execute, CommandDefinition, COMMANDS};
use super::components::SettingsButton;
use super::models::{
    capability_explanation, capability_risk, DialogError, Overlay, PendingPluginInstall,
    PermissionReview, SettingsTab, WorkspaceView,
};
use super::state::{activate_view, push_activity, refresh_operational_data, AppState};

#[component]
pub(crate) fn WorkspaceOverlays() -> Element {
    let state = use_context::<AppState>();
    let overlay = state.shell.overlay.read().clone();
    match overlay {
        Some(Overlay::Settings) => rsx! { SettingsModal {} },
        Some(Overlay::Commands) => rsx! { CommandPalette {} },
        Some(Overlay::PluginInstall(pending)) => rsx! { PluginInstallOverlay { pending } },
        Some(Overlay::PermissionReview(review)) => rsx! { PermissionReviewOverlay { review } },
        Some(Overlay::Error(error)) => rsx! { ErrorOverlay { error } },
        None => rsx! {},
    }
}

#[component]
fn SettingsModal() -> Element {
    let mut state = use_context::<AppState>();
    rsx! {
        div { class: "modal-backdrop", onclick: move |_| state.shell.overlay.set(None),
            div { class: "settings-modal", role: "dialog", aria_modal: "true", aria_labelledby: "settings-title", onclick: move |event| event.stop_propagation(),
                div { class: "modal-header", strong { id: "settings-title", "⚒ PolyGlid settings" } button { aria_label: "Close settings", onclick: move |_| state.shell.overlay.set(None), "×" } }
                div { class: "modal-body",
                    nav { class: "settings-nav",
                        SettingsButton { label: "Overview", active: *state.shell.settings_tab.read() == SettingsTab::Overview, onclick: move |_| state.shell.settings_tab.set(SettingsTab::Overview) }
                        SettingsButton { label: "Execution", active: *state.shell.settings_tab.read() == SettingsTab::Execution, onclick: move |_| state.shell.settings_tab.set(SettingsTab::Execution) }
                        SettingsButton { label: "Plugins", active: *state.shell.settings_tab.read() == SettingsTab::Plugins, onclick: move |_| state.shell.settings_tab.set(SettingsTab::Plugins) }
                    }
                    div { class: "settings-content",
                        match *state.shell.settings_tab.read() {
                            SettingsTab::Overview => rsx! { SettingsOverview {} },
                            SettingsTab::Execution => rsx! { ExecutionSettings {} },
                            SettingsTab::Plugins => rsx! { PluginSettings {} },
                        }
                    }
                }
                div { class: "modal-footer", button { class: "primary small", onclick: move |_| state.shell.overlay.set(None), "Done" } }
            }
        }
    }
}

#[component]
fn SettingsOverview() -> Element {
    let state = use_context::<AppState>();
    rsx! {
        h2 { "Local client overview" }
        p { class: "muted", "Live state loaded through the typed desktop client boundary." }
        div { class: "settings-grid",
            div { class: "setting-card", span { "Projects" } strong { "{state.catalog.projects.read().len()} indexed" } }
            div { class: "setting-card", span { "Saved targets" } strong { "{state.runs.targets.read().len()} available" } }
            div { class: "setting-card", span { "Executions" } strong { "{state.runs.executions.read().len()} recorded" } }
            div { class: "setting-card", span { "Reports" } strong { "{state.runs.reports.read().len()} persisted" } }
        }
        h3 { "Security model" }
        div { class: "setting-row", div { strong { "Per-run approval" } small { "Requested capabilities start denied and require explicit review." } } span { class: "badge good", "Enforced" } }
        div { class: "setting-row", div { strong { "Storage" } small { "Workspace catalog, executions, and reports are local." } } span { class: "badge good", "Local" } }
    }
}

#[component]
fn ExecutionSettings() -> Element {
    let mut state = use_context::<AppState>();
    rsx! {
        h2 { "Execution limits" }
        p { class: "muted", "Safety limits apply to each new local WASM execution." }
        label { class: "field-label", r#for: "fuel-limit", "Maximum WASM fuel" }
        input {
            id: "fuel-limit",
            r#type: "number",
            min: "1",
            value: "{state.runs.fuel_limit}",
            oninput: move |event| if let Ok(value) = event.value().parse() { state.runs.fuel_limit.set(value); }
        }
        p { class: "field-help", "Fuel bounds guest CPU work. The local timeout is 30 seconds." }
    }
}

#[component]
fn PluginSettings() -> Element {
    let state = use_context::<AppState>();
    rsx! {
        h2 { "Installed components" }
        p { class: "muted", "Current registry state; no preview entries are injected." }
        if state.plugins.items.read().is_empty() {
            div { class: "state-panel empty-state", p { "No components are installed." } }
        }
        for plugin in state.plugins.items.read().iter() {
            div { class: "setting-row",
                div { strong { "{plugin.name}" } small { "{plugin.id} · v{plugin.version}" } }
                span { class: if plugin.status == PluginStatus::Enabled { "badge good" } else { "badge" }, "{plugin.status}" }
            }
        }
    }
}

#[component]
fn CommandPalette() -> Element {
    let mut state = use_context::<AppState>();
    let client = use_context::<LocalClient>();
    let mut query = use_signal(String::new);
    let mut selected = use_signal(|| 0usize);
    let commands = filtered_commands(&query.read());
    rsx! {
        div { class: "modal-backdrop command-backdrop", onclick: move |_| state.shell.overlay.set(None),
            div { class: "command-palette", role: "dialog", aria_modal: "true", aria_label: "Command palette", onclick: move |event| event.stop_propagation(),
                input {
                    autofocus: true,
                    value: "{query}",
                    placeholder: "Type a command…",
                    aria_label: "Command palette search",
                    oninput: move |event| { query.set(event.value()); selected.set(0); },
                    onkeydown: move |event| {
                        let available = filtered_commands(&query.read());
                        match event.key().to_string().as_str() {
                            "ArrowDown" if !available.is_empty() => {
                                event.prevent_default();
                                let next = (*selected.read() + 1).min(available.len() - 1);
                                selected.set(next);
                            }
                            "ArrowUp" => {
                                event.prevent_default();
                                let next = selected.read().saturating_sub(1);
                                selected.set(next);
                            }
                            "Enter" => if let Some(command) = available.get(*selected.read()) {
                                event.prevent_default();
                                execute(state, command.action, client.clone());
                            },
                            "Escape" => state.shell.overlay.set(None),
                            _ => {}
                        }
                    }
                }
                div { class: "command-results", role: "listbox",
                    if commands.is_empty() { div { class: "command-empty", "No matching commands" } }
                    for (index, command) in commands.into_iter().enumerate() {
                        button {
                            class: if index == *selected.read() { "selected" } else { "" },
                            role: "option",
                            aria_selected: index == *selected.read(),
                            onmouseenter: move |_| selected.set(index),
                            onclick: {
                                let client = client.clone();
                                move |_| execute(state, command.action, client.clone())
                            },
                            div { strong { "{command.title}" } small { "{command.category}" } }
                            if !command.shortcut.is_empty() { kbd { "{command.shortcut}" } }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PluginInstallOverlay(pending: PendingPluginInstall) -> Element {
    let mut state = use_context::<AppState>();
    let client = use_context::<LocalClient>();
    rsx! {
        div { class: "modal-backdrop", onclick: move |_| state.shell.overlay.set(None),
            div { class: "settings-modal", role: "dialog", aria_modal: "true", aria_labelledby: "plugin-install-title", onclick: move |event| event.stop_propagation(),
                div { class: "modal-header", strong { "◇ Review component" } button { aria_label: "Cancel installation", onclick: move |_| state.shell.overlay.set(None), "×" } }
                div { class: "modal-body dialog-content",
                    h2 { id: "plugin-install-title", "{pending.plugin.display_name}" }
                    p { class: "muted", "{pending.plugin.id} · v{pending.plugin.version} by {pending.plugin.author}" }
                    p { "{pending.plugin.description}" }
                    h3 { "Manifest capability requests" }
                    if pending.plugin.requested_capabilities.is_empty() {
                        p { class: "muted", "This component requests no host capabilities." }
                    } else {
                        div { class: "permission-review-list",
                            for request in &pending.plugin.requested_capabilities {
                                div { class: "permission-item",
                                    span { class: "permission-icon", "◈" }
                                    div { class: "permission-copy", code { "{request.capability}" } small { "{request.capability.description()}" } }
                                    span { class: "permission-scope", "{request.scope}" }
                                }
                            }
                        }
                    }
                }
                div { class: "modal-footer",
                    button { class: "secondary", autofocus: true, onclick: move |_| state.shell.overlay.set(None), "Cancel" }
                    button { class: "primary small", onclick: move |_| {
                        let path = pending.path.clone();
                        let client = client.clone();
                        state.shell.overlay.set(None);
                        spawn(async move {
                            let result = tokio::task::spawn_blocking(move || client.install_plugin(&path)).await;
                            match result {
                                Ok(Ok(plugin)) => {
                                    let id = plugin.id.clone();
                                    let existing_index = state.plugins.items.read().iter().position(|item| item.id == id);
                                    if let Some(index) = existing_index {
                                        state.plugins.items.write()[index] = plugin;
                                    } else {
                                        state.plugins.items.write().push(plugin);
                                    }
                                    state.plugins.selected_id.set(Some(id));
                                    state.plugins.install_path.set(String::new());
                                    push_activity(state, "Installed a validated WASM component");
                                }
                                Ok(Err(error)) => show_error(state, "Installation failed", error.to_string()),
                                Err(error) => show_error(state, "Installation failed", format!("installation task failed: {error}")),
                            }
                        });
                    }, "Install component" }
                }
            }
        }
    }
}

#[component]
fn PermissionReviewOverlay(review: PermissionReview) -> Element {
    let mut state = use_context::<AppState>();
    let client = use_context::<LocalClient>();
    let all_approved = review
        .requested
        .iter()
        .all(|request| review.approved.contains(&request.capability));
    rsx! {
        div { class: "modal-backdrop", onclick: move |_| state.shell.overlay.set(None),
            div { class: "settings-modal permission-review", role: "dialog", aria_modal: "true", aria_labelledby: "permission-review-title", onclick: move |event| event.stop_propagation(),
                div { class: "modal-header", strong { "Permission review" } button { aria_label: "Cancel execution", onclick: move |_| state.shell.overlay.set(None), "×" } }
                div { class: "modal-body dialog-content permission-review-content",
                    div { class: "permission-review-header",
                        div { class: "permission-review-title", span { class: "permission-icon", "◈" } div { h2 { id: "permission-review-title", "Approve this execution" } p { "{review.plugin_name} will run only once against {review.target}." } } }
                    }
                    div { class: "permission-review-summary",
                        strong { "{review.requested.len()} requested · {review.approved.len()} approved" }
                        p { "Each approval applies only to this job. Nothing is preselected." }
                    }
                    if review.requested.is_empty() {
                        div { class: "state-panel", h3 { "No host permissions requested" } p { "This component can start without a capability grant." } }
                    } else {
                        div { class: "permission-review-list",
                            for request in &review.requested {
                                label { class: "permission-item permission-choice",
                                    input {
                                        r#type: "checkbox",
                                        checked: review.approved.contains(&request.capability),
                                        onchange: {
                                            let capability = request.capability;
                                            let review = review.clone();
                                            move |event| update_approval(state, review.clone(), capability, event.checked())
                                        }
                                    }
                                    div { class: "permission-copy",
                                        code { "{request.capability}" }
                                        small { "{capability_explanation(request.capability)}" }
                                    }
                                    div { class: "permission-scope", span { class: "badge", "{capability_risk(request.capability)}" } small { "{request.scope} · this run" } }
                                }
                            }
                        }
                    }
                    if !all_approved {
                        p { class: "field-help", "Approve every requested permission to enable execution, or cancel to deny the run." }
                    }
                }
                div { class: "modal-footer permission-review-actions",
                    button { class: "secondary", autofocus: true, onclick: move |_| state.shell.overlay.set(None), "Deny and cancel" }
                    button {
                        class: "primary small",
                        disabled: !all_approved,
                        onclick: move |_| start_execution(state, client.clone(), review.clone()),
                        "Approve and run"
                    }
                }
            }
        }
    }
}

fn update_approval(
    mut state: AppState,
    mut review: PermissionReview,
    capability: CapabilityKind,
    approved: bool,
) {
    if approved && !review.approved.contains(&capability) {
        review.approved.push(capability);
    } else if !approved {
        review.approved.retain(|candidate| *candidate != capability);
    }
    state
        .shell
        .overlay
        .set(Some(Overlay::PermissionReview(review)));
}

fn start_execution(mut state: AppState, client: LocalClient, review: PermissionReview) {
    state.shell.overlay.set(None);
    state.runs.error.set(None);
    let request = StartExecutionRequest {
        plugin_id: review.plugin_id.clone(),
        target: review.target.clone(),
        fuel_limit: *state.runs.fuel_limit.read(),
        timeout: Duration::from_secs(30),
        memory_limit: None,
        approved_capabilities: review.approved,
    };
    spawn(async move {
        let result = tokio::task::spawn_blocking(move || client.start_execution(request)).await;
        match result {
            Ok(Ok(job_id)) => {
                state.runs.active_job_id.set(Some(job_id));
                push_activity(
                    state,
                    format!(
                        "Started {} for {} ({job_id})",
                        review.plugin_name, review.target
                    ),
                );
                refresh_operational_data(state);
                activate_view(state, WorkspaceView::Executions);
            }
            Ok(Err(error)) => show_error(state, "Execution was not started", error.to_string()),
            Err(error) => show_error(
                state,
                "Execution was not started",
                format!("execution task failed: {error}"),
            ),
        }
    });
}

#[component]
fn ErrorOverlay(error: DialogError) -> Element {
    let mut state = use_context::<AppState>();
    rsx! {
        div { class: "modal-backdrop", onclick: move |_| state.shell.overlay.set(None),
            div { class: "settings-modal install-error", role: "alertdialog", aria_modal: "true", aria_labelledby: "error-dialog-title", onclick: move |event| event.stop_propagation(),
                div { class: "modal-header", strong { id: "error-dialog-title", "{error.title}" } button { aria_label: "Dismiss error", onclick: move |_| state.shell.overlay.set(None), "×" } }
                div { class: "modal-body dialog-content", p { "{error.message}" } }
                div { class: "modal-footer", button { class: "primary small", onclick: move |_| state.shell.overlay.set(None), "Dismiss" } }
            }
        }
    }
}

fn show_error(mut state: AppState, title: impl Into<String>, message: impl Into<String>) {
    state.shell.overlay.set(Some(Overlay::Error(DialogError {
        title: title.into(),
        message: message.into(),
    })));
}

fn filtered_commands(query: &str) -> Vec<CommandDefinition> {
    let query = query.trim().to_lowercase();
    COMMANDS
        .iter()
        .copied()
        .filter(|command| {
            query.is_empty()
                || fuzzy_match(&command.title.to_lowercase(), &query)
                || fuzzy_match(&command.category.to_lowercase(), &query)
        })
        .collect()
}

fn fuzzy_match(candidate: &str, query: &str) -> bool {
    let mut characters = candidate.chars();
    query
        .chars()
        .all(|needle| characters.by_ref().any(|candidate| candidate == needle))
}
