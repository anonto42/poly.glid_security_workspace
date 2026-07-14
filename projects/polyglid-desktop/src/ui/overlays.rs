use dioxus::prelude::*;

use super::components::SettingsButton;
use super::models::{SettingsTab, WorkspaceView};
use super::state::AppState;

#[component]
pub(crate) fn WorkspaceOverlays() -> Element {
    let state = use_context::<AppState>();
    rsx! {
        if *state.settings_open.read() { SettingsModal {} }
        if *state.command_open.read() { CommandPalette {} }
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
    rsx! {
        div { class: "modal-backdrop command-backdrop", onclick: move |_| state.command_open.set(false),
            div { class: "command-palette", onclick: move |event| event.stop_propagation(),
                input { autofocus: true, placeholder: "Type a command or search the workspace…" }
                p { class: "section-label", "Quick navigation" }
                button { onclick: move |_| { state.active_view.set(WorkspaceView::Explorer); state.command_open.set(false); }, "⚡ Open scanner" span { "Explorer" } }
                button { onclick: move |_| { state.active_view.set(WorkspaceView::Tracks); state.command_open.set(false); }, "☷ Open work tracks" span { "Project" } }
                button { onclick: move |_| { state.active_view.set(WorkspaceView::Automation); state.command_open.set(false); }, "⚙ Run workspace verification" span { "Automation" } }
            }
        }
    }
}
