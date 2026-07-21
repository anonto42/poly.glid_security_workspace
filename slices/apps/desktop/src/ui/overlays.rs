use dioxus::prelude::*;

use crate::backend::DesktopBackend;

use super::commands::{execute, CommandDefinition, COMMANDS};
use super::components::SettingsButton;
use super::models::{PluginCard, SettingsTab};
use super::state::AppState;

#[component]
pub(crate) fn WorkspaceOverlays() -> Element {
    let state = use_context::<AppState>();
    rsx! {
        if *state.settings_open.read() { SettingsModal {} }
        if *state.command_open.read() { CommandPalette {} }
        PluginInstallOverlay {}
    }
}

#[component]
fn SettingsModal() -> Element {
    let mut state = use_context::<AppState>();
    rsx! {
        div { class: "modal-backdrop", onclick: move |_| state.settings_open.set(false),
            div { class: "settings-modal", onclick: move |event| event.stop_propagation(),
                div { class: "modal-header", strong { "⚒ PolyGlid settings" } button { onclick: move |_| state.settings_open.set(false), "×" } }
                div { class: "modal-body",
                    nav { class: "settings-nav",
                        SettingsButton { label: "Overview", active: *state.settings_tab.read() == SettingsTab::Overview, onclick: move |_| state.settings_tab.set(SettingsTab::Overview) }
                        SettingsButton { label: "Engine", active: *state.settings_tab.read() == SettingsTab::Engine, onclick: move |_| state.settings_tab.set(SettingsTab::Engine) }
                        SettingsButton { label: "Plugins", active: *state.settings_tab.read() == SettingsTab::Plugins, onclick: move |_| state.settings_tab.set(SettingsTab::Plugins) }
                    }
                    div { class: "settings-content",
                        match *state.settings_tab.read() {
                            SettingsTab::Overview => rsx! { SettingsOverview {} },
                            SettingsTab::Engine => rsx! { EngineSettings {} },
                            SettingsTab::Plugins => rsx! { PluginSettings {} },
                        }
                    }
                }
                div { class: "modal-footer", button { class: "primary small", onclick: move |_| state.settings_open.set(false), "Done" } }
            }
        }
    }
}

#[component]
fn SettingsOverview() -> Element {
    rsx! {
        h2 { "System overview" }
        p { class: "muted", "Status of the local sandbox and control plane." }
        div { class: "settings-grid", div { class: "setting-card", span { "Engine runtime" } strong { "◉ Wasmtime 46" } } div { class: "setting-card", span { "Sandbox model" } strong { "◈ WASI Preview 1" } } }
        h3 { "Active capabilities" }
        div { class: "setting-row", code { "dns-resolve" } span { class: "badge good", "Scoped" } }
        div { class: "setting-row", code { "report-write" } span { class: "badge good", "Scoped" } }
    }
}

#[component]
fn EngineSettings() -> Element {
    let mut state = use_context::<AppState>();
    rsx! {
        h2 { "WASM engine" }
        p { class: "muted", "Configure safety thresholds for local component execution." }
        label { class: "field-label", "Maximum WASM fuel" }
        input { r#type: "number", value: "{state.fuel_limit}", oninput: move |event| if let Ok(value) = event.value().parse() { state.fuel_limit.set(value); } }
        p { class: "field-help", "Prevents CPU starvation and infinite guest loops." }
    }
}

#[component]
fn PluginSettings() -> Element {
    let state = use_context::<AppState>();
    rsx! {
        h2 { "Loaded plugins" }
        p { class: "muted", "Workspace components and their current runtime state." }
        for plugin in state.plugins.read().iter() {
            div { class: "setting-row", div { strong { "{plugin.name}" } small { "{plugin.id} · v{plugin.version}" } } span { class: if plugin.enabled { "badge good" } else { "badge" }, if plugin.enabled { "Enabled" } else { "Disabled" } } }
        }
    }
}

#[component]
fn CommandPalette() -> Element {
    let mut state = use_context::<AppState>();
    let backend = use_context::<DesktopBackend>();
    let mut query = use_signal(String::new);
    let mut selected = use_signal(|| 0usize);
    let commands = filtered_commands(&query.read());
    rsx! {
        div { class: "modal-backdrop command-backdrop", onclick: move |_| state.command_open.set(false),
            div { class: "command-palette", onclick: move |event| event.stop_propagation(),
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
                                execute(state, command.action, backend.clone());
                            },
                            "Escape" => state.command_open.set(false),
                            _ => {}
                        }
                    }
                }
                div { class: "command-results", role: "listbox",
                    if commands.is_empty() {
                        div { class: "command-empty", "No matching commands" }
                    }
                    for (index, command) in commands.into_iter().enumerate() {
                        button {
                            class: if index == *selected.read() { "selected" } else { "" },
                            role: "option",
                            aria_selected: index == *selected.read(),
                            onmouseenter: move |_| selected.set(index),
                            onclick: {
                                let backend = backend.clone();
                                move |_| execute(state, command.action, backend.clone())
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
fn PluginInstallOverlay() -> Element {
    let mut state = use_context::<AppState>();
    let backend = use_context::<DesktopBackend>();
    let pending = state.pending_install.read().clone();
    if let Some(info) = pending {
        rsx! {
            div { class: "modal-backdrop", onclick: move |_| state.pending_install.set(None),
                div { class: "settings-modal", onclick: move |event| event.stop_propagation(),
                    div { class: "modal-header", strong { "◇ Install plugin" } button { onclick: move |_| state.pending_install.set(None), "×" } }
                    div { class: "modal-body",
                        h2 { "{info.name}" }
                        p { class: "muted", "{info.id} · v{info.version} by {info.author}" }
                        p { "{info.description}" }
                        h3 { "Requested capabilities" }
                        if info.capabilities.is_empty() {
                            p { class: "muted", "This plugin does not request any capabilities." }
                        } else {
                            div { class: "capability-list",
                                for capability in &info.capabilities {
                                    div { class: "setting-row",
                                        code { "{capability}" }
                                        span { class: "badge good", "Sandboxed" }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "modal-footer",
                        button { class: "secondary", onclick: move |_| state.pending_install.set(None), "Cancel" }
                        button { class: "primary small", onclick: move |_| {
                            let path = info.path.clone();
                            state.pending_install.set(None);
                            state.install_error.set(None);
                            let backend = backend.clone();
                            spawn(async move {
                                let result = tokio::task::spawn_blocking(move || backend.install_plugin(&path))
                                    .await
                                    .map_err(|error| format!("install task failed: {error}"))
                                    .and_then(|result| result);
                                match result {
                                    Ok(entry) => {
                                        let card = PluginCard {
                                            id: entry.id.as_str().to_string(),
                                            name: entry.name,
                                            version: entry.version.to_string(),
                                            description: entry.description,
                                            capabilities: entry.capabilities.into_iter()
                                                .map(|c| c.to_string())
                                                .collect(),
                                            enabled: true,
                                        };
                                        if !state.plugins.read().iter().any(|p| p.id == card.id) {
                                            state.plugins.write().push(card);
                                        }
                                        state.selected_plugin.set(entry.id.as_str().to_string());
                                    }
                                    Err(error) => state.install_error.set(Some(error)),
                                }
                            });
                        }, "Install" }
                    }
                }
            }
        }
    } else {
        rsx! {
            if let Some(error) = state.install_error.read().as_ref() {
                div { class: "modal-backdrop", onclick: move |_| state.install_error.set(None),
                    div { class: "settings-modal install-error", onclick: move |event| event.stop_propagation(),
                        div { class: "modal-header", strong { "✗ Install failed" } button { onclick: move |_| state.install_error.set(None), "×" } }
                        div { class: "modal-body",
                            p { "{error}" }
                        }
                        div { class: "modal-footer",
                            button { class: "primary small", onclick: move |_| state.install_error.set(None), "Dismiss" }
                        }
                    }
                }
            }
        }
    }
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
