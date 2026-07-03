use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::str::FromStr;

use polyglid_config::AppConfig;
use polyglid_core::{CoreEngine, InMemoryPermissionStore, PluginRef, PluginRunRequest, Target};
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

fn plugin_list() -> Result<(), String> {
    let config = AppConfig::load_from_env().map_err(|err| err.to_string())?;
    println!("plugin directory: {}", config.plugin_dir.display());
    println!("installed plugin discovery is pending");
    Ok(())
}

fn plugin_inspect(path: &str) -> Result<(), String> {
    let mut engine = engine(Vec::new())?;
    let manifest = engine
        .inspect_plugin(PluginRef::from_path(PathBuf::from(path)))
        .map_err(|err| err.to_string())?;

    println!("id: {}", manifest.id.as_str());
    println!("name: {}", manifest.name);
    println!("version: {}", manifest.version);
    if manifest.requested_capabilities.is_empty() {
        println!("requested capabilities: none");
    } else {
        println!("requested capabilities:");
        for capability in manifest.requested_capabilities {
            println!("- {capability}");
        }
    }
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

fn plugin_run(path: &str, target: &str, flags: &[String]) -> Result<(), String> {
    let mut engine = engine(parse_allow_flags(flags)?)?;
    let report = engine
        .run_plugin(PluginRunRequest {
            plugin: PluginRef::from_path(PathBuf::from(path)),
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
    println!("  polyglid plugin inspect <plugin.wasm>");
    println!("  polyglid plugin componentize <module.wasm> <component.wasm>");
    println!("  polyglid plugin run <plugin.wasm> --target <target> [--allow <capability>...]");
    Ok(())
}

fn print_help() {
    println!("PolyGlid development CLI");
    println!();
    println!("usage:");
    println!("  polyglid doctor");
    println!("  polyglid config validate");
    println!("  polyglid plugin list");
    println!("  polyglid plugin inspect <plugin.wasm>");
    println!("  polyglid plugin componentize <module.wasm> <component.wasm>");
    println!("  polyglid plugin run <plugin.wasm> --target <target> [--allow <capability>...]");
}
