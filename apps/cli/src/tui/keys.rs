//! TUI keyboard input handling and command execution.

use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use polyglid_core::execution::ExecutionConfig;
use polyglid_core::PluginRuntime;
use polyglid_plugin_api::Capability;
use polyglid_runtime::WasmRuntime;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

use super::app::{App, Focus, Mode};

/// Handle a key event, updating application state and running commands if needed.
pub fn handle_key(app: &mut App, key: KeyEvent) {
    if app.show_help {
        if key.code == KeyCode::Char('?') || key.code == KeyCode::Esc {
            app.toggle_help();
        }
        return;
    }

    if app.show_panel {
        if key.code == KeyCode::Esc || key.code == KeyCode::Enter {
            app.show_panel = false;
        }
        return;
    }

    match app.mode {
        Mode::Normal => handle_normal_key(app, key),
        Mode::Command => handle_command_key(app, key),
    }
}

fn handle_normal_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => app.quit = true,
        KeyCode::Char('?') => app.toggle_help(),
        KeyCode::Char(':') => app.enter_command_mode(),
        KeyCode::Tab => app.focus = app.focus.next(),
        KeyCode::Enter => {
            if app.focus == Focus::PluginTable {
                app.toggle_panel();
            }
        }
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if let Some(job) = app.selected_job() {
                if let Err(err) = app.execution_manager.cancel_job(job.id) {
                    app.log_error(format!("Failed to cancel: {err}"));
                } else {
                    app.log_info(format!("Cancelled scan: {}", job.id));
                }
            }
        }
        KeyCode::Up => match app.focus {
            Focus::PluginTable => app.select_previous(),
            Focus::Logs => app.scroll_logs_up(),
            _ => {}
        },
        KeyCode::Down => match app.focus {
            Focus::PluginTable => app.select_next(),
            Focus::Logs => app.scroll_logs_down(),
            _ => {}
        },
        KeyCode::Esc => {
            app.show_help = false;
            app.show_panel = false;
        }
        _ => {}
    }
}

fn handle_command_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.exit_command_mode(),
        KeyCode::Char(c) => app.input.push(c),
        KeyCode::Backspace => {
            app.input.pop();
        }
        KeyCode::Enter => {
            let cmd = app.input.trim().to_string();
            app.exit_command_mode();
            if !cmd.is_empty() {
                execute_command(app, &cmd);
            }
        }
        _ => {}
    }
}

fn execute_command(app: &mut App, cmd: &str) {
    let tokens: Vec<&str> = cmd.split_whitespace().collect();
    if tokens.is_empty() {
        return;
    }

    let command = tokens[0];
    match command {
        "q" | "quit" => app.quit = true,
        "cancel" | "c" => {
            if let Some(job) = app.selected_job() {
                if let Err(err) = app.execution_manager.cancel_job(job.id) {
                    app.log_error(format!("Failed to cancel: {err}"));
                } else {
                    app.log_info(format!("Cancelled scan: {}", job.id));
                }
            } else {
                app.log_warn("No active scan selected to cancel");
            }
        }
        "inspect" => {
            if tokens.len() < 2 {
                app.log_error("Usage: :inspect <plugin.wasm>");
                return;
            }
            let path = tokens[1];
            app.log_info(format!("Inspecting plugin: {path}"));

            // Try lookup from manager registry first
            if let Ok(id) = polyglid_plugin_api::PluginId::new(path) {
                if let Some(entry) = app.plugin_manager.get_plugin(&id) {
                    app.log_info(format!("Plugin ID: {}", entry.id.as_str()));
                    app.log_info(format!("Display Name: {}", entry.name));
                    app.log_info(format!("Version: {}", entry.version));
                    app.log_info(format!("Status: {}", entry.status));
                    app.log_info(format!("Source: {}", entry.source));
                    app.log_info(format!("Checksum: {}", entry.checksum));
                    if entry.capabilities.is_empty() {
                        app.log_info("Requested capabilities: none");
                    } else {
                        app.log_info("Requested capabilities:");
                        for cap in &entry.capabilities {
                            app.log_info(format!("  - {cap}"));
                        }
                    }
                    return;
                }
            }

            let runtime = WasmRuntime::new();
            let plugin_ref = polyglid_core::PluginRef::from_path(PathBuf::from(path));
            match runtime.inspect(&plugin_ref) {
                Ok(manifest) => {
                    app.log_info(format!("Plugin ID: {}", manifest.id.as_str()));
                    app.log_info(format!("Display Name: {}", manifest.name));
                    app.log_info(format!("Version: {}", manifest.version));
                    if manifest.requested_capabilities.is_empty() {
                        app.log_info("Requested capabilities: none");
                    } else {
                        app.log_info("Requested capabilities:");
                        for req in manifest.requested_capabilities {
                            app.log_info(format!("  - {req}"));
                        }
                    }
                }
                Err(err) => app.log_error(format!("Failed to inspect: {err}")),
            }
        }
        "run" => {
            if tokens.len() < 4 {
                app.log_error(
                    "Usage: :run <plugin.wasm> --target <domain> [--allow <capability>...]",
                );
                return;
            }
            let path = tokens[1];
            let mut target_domain = None;
            let mut allowed_caps = Vec::new();

            let mut i = 2;
            while i < tokens.len() {
                match tokens[i] {
                    "--target" => {
                        if i + 1 < tokens.len() {
                            target_domain = Some(tokens[i + 1]);
                            i += 2;
                        } else {
                            app.log_error("Missing target domain after --target");
                            return;
                        }
                    }
                    "--allow" => {
                        if i + 1 < tokens.len() {
                            match Capability::from_str(tokens[i + 1]) {
                                Ok(cap) => allowed_caps.push(cap),
                                Err(err) => {
                                    app.log_error(format!(
                                        "Unknown capability: {}: {err}",
                                        tokens[i + 1]
                                    ));
                                    return;
                                }
                            }
                            i += 2;
                        } else {
                            app.log_error("Missing capability after --allow");
                            return;
                        }
                    }
                    _ => {
                        app.log_error(format!("Unknown option: {}", tokens[i]));
                        return;
                    }
                }
            }

            let target_str = match target_domain {
                Some(t) => t,
                None => {
                    app.log_error("Missing required --target <domain>");
                    return;
                }
            };

            run_scan(app, path, target_str, allowed_caps);
        }
        _ => {
            app.log_error(format!("Unknown command: {command}"));
        }
    }
}

fn run_scan(app: &mut App, plugin_path: &str, target_domain: &str, allowed_caps: Vec<Capability>) {
    let mut resolved_path = plugin_path.to_string();

    if let Ok(id) = polyglid_plugin_api::PluginId::new(plugin_path) {
        if let Some(entry) = app.plugin_manager.get_plugin(&id) {
            if entry.status == polyglid_config::plugin_registry::PluginStatus::Disabled {
                app.log_error(format!(
                    "Plugin '{}' is currently disabled in the workspace",
                    id.as_str()
                ));
                return;
            }
            resolved_path = entry.path.to_string_lossy().to_string();
        }
    }

    app.log_info(format!(
        "Submitting scan job for {plugin_path} on {target_domain}"
    ));

    let config = ExecutionConfig {
        fuel_limit: 25_000_000,
        timeout: Duration::from_secs(30),
        memory_limit: None,
        allowed_capabilities: allowed_caps,
    };

    let job_id = app
        .execution_manager
        .submit_job(resolved_path, target_domain.to_string(), config);

    app.log_info(format!("Submitted job ID: {job_id}"));
}
