use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::str::FromStr;

use polyglid_config::AppConfig;
use polyglid_core::{
    CoreEngine, InMemoryPermissionStore, PluginRef, PluginRunRequest, PluginRuntime, Target,
};
use polyglid_events::VecEventSink;
use polyglid_plugin_api::Capability;
use polyglid_runtime::WasmRuntime;
use wasi_preview1_component_adapter_provider::{
    WASI_SNAPSHOT_PREVIEW1_ADAPTER_NAME, WASI_SNAPSHOT_PREVIEW1_REACTOR_ADAPTER,
};

pub(crate) mod tui;

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    match args.as_slice() {
        [] => tui::event_loop(),
        [flag] if flag == "--help" || flag == "-h" => {
            print_help();
            Ok(())
        }
        [command] if command == "doctor" => doctor(),
        [command] if command == "config" => config_help(),
        [command, subcommand] if command == "config" && subcommand == "validate" => {
            AppConfig::load_from_env().map_err(|err| err.to_string())?;
            println!("config: valid");
            Ok(())
        }
        [command] if command == "plugin" => plugin_help(),
        [command, subcommand] if command == "plugin" && subcommand == "list" => plugin_list(),
        [command, subcommand, path] if command == "plugin" && subcommand == "inspect" => {
            plugin_inspect(path)
        }
        [command, subcommand, path] if command == "plugin" && subcommand == "install" => {
            plugin_install(path)
        }
        [command, subcommand, id] if command == "plugin" && subcommand == "remove" => {
            plugin_remove(id)
        }
        [command, subcommand, id] if command == "plugin" && subcommand == "enable" => {
            plugin_enable(id, true)
        }
        [command, subcommand, id] if command == "plugin" && subcommand == "disable" => {
            plugin_enable(id, false)
        }
        [command, subcommand, input, output]
            if command == "plugin" && subcommand == "componentize" =>
        {
            plugin_componentize(input, output)
        }
        [command, subcommand, path, target_flag, target]
            if command == "plugin" && subcommand == "run" && target_flag == "--target" =>
        {
            plugin_run(path, target, &[])
        }
        [command, subcommand, path, target_flag, target, rest @ ..]
            if command == "plugin" && subcommand == "run" && target_flag == "--target" =>
        {
            plugin_run(path, target, rest)
        }
        _ => Err("unknown command; run `polyglid --help`".to_string()),
    }
}

fn doctor() -> Result<(), String> {
    AppConfig::load_from_env().map_err(|err| err.to_string())?;
    println!("polyglid doctor");
    println!("workspace: ok");
    println!("config: ok");
    println!("runtime: component execution available");
    Ok(())
}

fn manager() -> Result<polyglid_core::plugin_manager::PluginManager<WasmRuntime>, String> {
    let config = AppConfig::load_from_env().map_err(|err| err.to_string())?;
    let runtime = std::sync::Arc::new(WasmRuntime::new());
    let storage = polyglid_config::plugin_registry::JsonRegistryStorage;
    let pm = polyglid_core::plugin_manager::PluginManager::new(runtime, &config, storage)?;
    let _ = pm.sync_directory();
    Ok(pm)
}

fn plugin_list() -> Result<(), String> {
    let pm = manager()?;
    let plugins = pm.get_plugins();
    if plugins.is_empty() {
        println!("No plugins currently installed in workspace.");
    } else {
        println!(
            "{:<28} {:<10} {:<10} {:<25}",
            "ID", "Version", "Status", "Name"
        );
        println!("{:-<75}", "");
        for p in plugins {
            println!(
                "{:<28} {:<10} {:<10} {:<25}",
                p.id.as_str(),
                p.version.to_string(),
                p.status.to_string(),
                p.name
            );
        }
    }
    Ok(())
}

fn plugin_inspect(id_or_path: &str) -> Result<(), String> {
    let pm = manager()?;

    // Try registry lookup first
    if let Ok(id) = polyglid_plugin_api::PluginId::new(id_or_path) {
        if let Some(entry) = pm.get_plugin(&id) {
            println!("id: {}", entry.id.as_str());
            println!("name: {}", entry.name);
            println!("version: {}", entry.version);
            println!("author: {}", entry.author);
            println!("description: {}", entry.description);
            println!("status: {}", entry.status.to_string());
            println!("source: {}", entry.source.to_string());
            println!("checksum: {}", entry.checksum);
            println!("size: {} bytes", entry.file_size);
            if entry.capabilities.is_empty() {
                println!("requested capabilities: none");
            } else {
                println!("requested capabilities:");
                for cap in entry.capabilities {
                    println!("- {cap}");
                }
            }
            return Ok(());
        }
    }

    // Fallback to inspecting raw local WASM file path
    let path = PathBuf::from(id_or_path);
    if path.exists() {
        let runtime = WasmRuntime::new();
        let plugin_ref = PluginRef::from_path(path);
        let manifest = runtime
            .inspect(&plugin_ref)
            .map_err(|err| err.to_string())?;
        let metadata = runtime
            .call_metadata(&plugin_ref)
            .map_err(|err| err.to_string())?;
        println!("id: {}", manifest.id.as_str());
        println!("name: {}", metadata.display_name);
        println!("version: {}", metadata.version);
        println!("author: {}", metadata.author);
        println!("description: {}", metadata.description);
        if manifest.requested_capabilities.is_empty() {
            println!("requested capabilities: none");
        } else {
            println!("requested capabilities:");
            for cap in manifest.requested_capabilities {
                println!("- {cap}");
            }
        }
        return Ok(());
    }

    Err(format!(
        "Plugin '{}' not found in registry and is not a valid wasm file path",
        id_or_path
    ))
}

fn plugin_install(wasm_path: &str) -> Result<(), String> {
    let pm = manager()?;
    let entry = pm.install_plugin(
        Path::new(wasm_path),
        polyglid_config::plugin_registry::PluginSource::LocalPath(PathBuf::from(wasm_path)),
    )?;
    println!(
        "successfully installed plugin '{}' (version {})",
        entry.id.as_str(),
        entry.version
    );
    Ok(())
}

fn plugin_remove(plugin_id: &str) -> Result<(), String> {
    let pm = manager()?;
    let id = polyglid_plugin_api::PluginId::new(plugin_id).map_err(|err| err.to_string())?;
    pm.uninstall_plugin(&id)?;
    println!("successfully uninstalled plugin '{plugin_id}'");
    Ok(())
}

fn plugin_enable(plugin_id: &str, enabled: bool) -> Result<(), String> {
    let pm = manager()?;
    let id = polyglid_plugin_api::PluginId::new(plugin_id).map_err(|err| err.to_string())?;
    pm.toggle_plugin_enabled(&id, enabled)?;
    let state = if enabled { "enabled" } else { "disabled" };
    println!("successfully {state} plugin '{plugin_id}'");
    Ok(())
}

fn plugin_componentize(input: &str, output: &str) -> Result<(), String> {
    let module = fs::read(input).map_err(|err| format!("failed to read {input}: {err}"))?;
    let component = wit_component::ComponentEncoder::default()
        .module(&module)
        .map_err(|err| format!("failed to read component metadata from {input}: {err:#}"))?
        .adapter(
            WASI_SNAPSHOT_PREVIEW1_ADAPTER_NAME,
            WASI_SNAPSHOT_PREVIEW1_REACTOR_ADAPTER,
        )
        .map_err(|err| format!("failed to configure WASI preview1 adapter: {err:#}"))?
        .validate(true)
        .encode()
        .map_err(|err| format!("failed to encode component: {err:#}"))?;

    fs::write(output, component).map_err(|err| format!("failed to write {output}: {err}"))?;
    println!("component: {output}");
    Ok(())
}

fn plugin_run(id_or_path: &str, target: &str, flags: &[String]) -> Result<(), String> {
    let pm = manager()?;
    let mut resolved_path = PathBuf::from(id_or_path);

    if let Ok(id) = polyglid_plugin_api::PluginId::new(id_or_path) {
        if let Some(entry) = pm.get_plugin(&id) {
            resolved_path = entry.path;
        }
    }

    let mut engine = engine(parse_allow_flags(flags)?)?;
    let report = engine
        .run_plugin(PluginRunRequest {
            plugin: PluginRef::from_path(resolved_path),
            target: Target::parse(target).map_err(|err| err.to_string())?,
        })
        .map_err(|err| err.to_string())?;

    println!("plugin: {}", report.plugin_name);
    println!("target: {}", report.target_tested);
    println!("summary: {}", report.summary);
    if report.issues.is_empty() {
        println!("issues: none");
    } else {
        println!("issues:");
        for issue in report.issues {
            println!("- [{}] {}", issue.severity, issue.title);
            println!("  {}", issue.description);
            println!("  recommendation: {}", issue.recommendation);
        }
    }
    Ok(())
}

pub(crate) fn engine(
    allowed_capabilities: Vec<Capability>,
) -> Result<CoreEngine<WasmRuntime, InMemoryPermissionStore, VecEventSink>, String> {
    let config = AppConfig::load_from_env().map_err(|err| err.to_string())?;
    let mut permissions = InMemoryPermissionStore::default();
    for capability in &config.default_capabilities {
        permissions.grant_for_all(*capability);
    }
    for grant in &config.approved_capabilities {
        match &grant.plugin_id {
            Some(plugin_id) => permissions.grant_request(plugin_id.clone(), grant.request.clone()),
            None => permissions.grant_request_for_all(grant.request.clone()),
        }
    }
    for capability in allowed_capabilities {
        permissions.grant_for_all(capability);
    }

    CoreEngine::new(
        WasmRuntime::new(),
        permissions,
        VecEventSink::default(),
        config,
    )
    .map_err(|err| err.to_string())
}

pub(crate) fn parse_allow_flags(flags: &[String]) -> Result<Vec<Capability>, String> {
    let mut capabilities = Vec::new();
    let mut chunks = flags.chunks_exact(2);
    for chunk in &mut chunks {
        if chunk[0] != "--allow" {
            return Err(format!("unknown plugin run flag: {}", chunk[0]));
        }
        capabilities.push(Capability::from_str(&chunk[1]).map_err(|err| err.to_string())?);
    }
    if !chunks.remainder().is_empty() {
        return Err("expected `--allow <capability>`".to_string());
    }
    Ok(capabilities)
}

fn config_help() -> Result<(), String> {
    println!("config commands:");
    println!("  polyglid config validate");
    println!("  POLYGLID_CONFIG=/path/to/config.toml polyglid config validate");
    Ok(())
}

fn plugin_help() -> Result<(), String> {
    println!("plugin commands:");
    println!("  polyglid plugin list");
    println!("  polyglid plugin inspect <plugin_id_or_wasm_path>");
    println!("  polyglid plugin install <wasm_path>");
    println!("  polyglid plugin remove <plugin_id>");
    println!("  polyglid plugin enable <plugin_id>");
    println!("  polyglid plugin disable <plugin_id>");
    println!("  polyglid plugin componentize <module.wasm> <component.wasm>");
    println!("  polyglid plugin run <plugin_id_or_wasm_path> --target <target> [--allow <capability>...]");
    Ok(())
}

fn print_help() {
    println!("PolyGlid development CLI");
    println!();
    println!("usage:");
    println!("  polyglid doctor");
    println!("  polyglid config validate");
    println!("  polyglid plugin list");
    println!("  polyglid plugin inspect <plugin_id_or_wasm_path>");
    println!("  polyglid plugin install <wasm_path>");
    println!("  polyglid plugin remove <plugin_id>");
    println!("  polyglid plugin enable <plugin_id>");
    println!("  polyglid plugin disable <plugin_id>");
    println!("  polyglid plugin componentize <module.wasm> <component.wasm>");
    println!("  polyglid plugin run <plugin_id_or_wasm_path> --target <target> [--allow <capability>...]");
}
