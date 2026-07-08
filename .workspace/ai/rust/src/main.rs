use clap::{Parser, Subcommand};
use colored::*;
use anyhow::Result;
use std::time::Instant;

mod core;
mod providers;
mod features;
mod cache;
mod cli;
mod tools;
mod feedback;
mod pipelines;

use crate::core::engine::AIEngine;
use crate::cli::commands::*;
use crate::cli::commands::log_analytics;

#[derive(Parser)]
#[command(name = "polyglid-ai")]
#[command(about = "AI Assistant for PolyGlid Workspace")]
#[command(version = "1.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Analyze {
        #[arg(long)]
        file: Option<String>,
        #[arg(long, default_value = "table")]
        format: String,
    },
    Generate {
        #[command(subcommand)]
        generate_type: GenerateType,
    },
    Review {
        #[arg()]
        file: String,
    },
    Suggest {
        #[arg(long, default_value = "10")]
        limit: usize,
    },
    Optimize {
        #[arg(default_value = "build")]
        target: String,
    },
    Security {
        #[arg(long)]
        file: Option<String>,
    },
    Status,
    /// Ingest workspace code into vector search index
    Ingest {
        #[arg(long)]
        force: bool,
    },
    /// Search the code index by semantic similarity
    Search {
        #[arg()]
        query: String,
        #[arg(long, default_value = "5")]
        limit: usize,
    },
    /// Execute a workspace tool (read_file, search_code, etc.)
    Tool {
        #[arg()]
        name: String,
        #[arg()]
        args: Vec<String>,
    },
    /// List available tools
    Tools,
    /// Manage prediction feedback (list, show, accept, dismiss)
    Feedback {
        #[arg()]
        action: String,
        #[arg()]
        id_or_category: Option<String>,
    },
    /// Start the background pipeline daemon (file watcher + scheduler + auto-suggest)
    Daemon,
}

#[derive(Subcommand)]
enum GenerateType {
    Code {
        #[arg()]
        description: String,
        #[arg(long)]
        language: String,
        #[arg(long)]
        output: Option<String>,
    },
    Tests {
        #[arg()]
        file: String,
        #[arg(long)]
        output: Option<String>,
    },
    Docs {
        #[arg()]
        file: String,
        #[arg(long)]
        output: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    let workspace_path = std::env::current_dir()?;
    let engine = AIEngine::new(&workspace_path).await?;

    match cli.command {
        Commands::Analyze { file, format } => {
            let start = Instant::now();
            if let Some(f) = file { analyze_file(&engine, &f).await?; }
            else { analyze_workspace(&engine, &format).await?; }
            log_analytics(&engine, "analyze", start.elapsed().as_millis() as u64, "ok").await?;
        }
        Commands::Generate { generate_type } => {
            let start = Instant::now();
            match generate_type {
                GenerateType::Code { description, language, output } => {
                    generate_code(&engine, &description, &language, output).await?
                }
                GenerateType::Tests { file, output } => {
                    generate_tests(&engine, &file, output).await?
                }
                GenerateType::Docs { file, output } => {
                    generate_docs(&engine, &file, output).await?
                }
            }
            log_analytics(&engine, "generate", start.elapsed().as_millis() as u64, "ok").await?;
        }
        Commands::Review { file } => {
            let start = Instant::now();
            review_code(&engine, &file).await?;
            log_analytics(&engine, "review", start.elapsed().as_millis() as u64, "ok").await?;
        }
        Commands::Suggest { limit } => {
            let start = Instant::now();
            get_suggestions(&engine, limit).await?;
            log_analytics(&engine, "suggest", start.elapsed().as_millis() as u64, "ok").await?;
        }
        Commands::Optimize { target } => {
            let start = Instant::now();
            optimize_build(&engine, &target).await?;
            log_analytics(&engine, "optimize", start.elapsed().as_millis() as u64, "ok").await?;
        }
        Commands::Security { file } => {
            let start = Instant::now();
            if let Some(f) = file { security_scan_file(&engine, &f).await?; }
            else { security_scan_workspace(&engine).await?; }
            log_analytics(&engine, "security", start.elapsed().as_millis() as u64, "ok").await?;
        }
        Commands::Status => {
            let start = Instant::now();
            show_status(&engine).await?;
            log_analytics(&engine, "status", start.elapsed().as_millis() as u64, "ok").await?;
        }
        Commands::Ingest { force: _ } => {
            let start = Instant::now();
            ingest_workspace(&engine).await?;
            log_analytics(&engine, "ingest", start.elapsed().as_millis() as u64, "ok").await?;
        }
        Commands::Search { query, limit } => {
            let start = Instant::now();
            search_index(&engine, &query, limit).await?;
            log_analytics(&engine, "search", start.elapsed().as_millis() as u64, "ok").await?;
        }
        Commands::Tool { name, args } => {
            let start = Instant::now();
            run_tool(&engine, &name, &args).await?;
            log_analytics(&engine, "tool", start.elapsed().as_millis() as u64, "ok").await?;
        }
        Commands::Tools => {
            let start = Instant::now();
            list_tools(&engine).await?;
            log_analytics(&engine, "tools", start.elapsed().as_millis() as u64, "ok").await?;
        }
        Commands::Feedback { action, id_or_category } => {
            let start = Instant::now();
            handle_feedback(&engine, &action, id_or_category.as_deref()).await?;
            log_analytics(&engine, "feedback", start.elapsed().as_millis() as u64, "ok").await?;
        }
        Commands::Daemon => start_daemon(&engine).await?,
    }

    Ok(())
}

async fn show_status(engine: &AIEngine) -> Result<()> {
    println!("{}", "\n📊 AI Workspace Status".bold().cyan());
    println!("{}", "═".repeat(50).cyan());
    let ctx = engine.context.read().await;
    let projects = ctx.get_projects().await?;
    println!("  {}: {}", "Workspace".green(), ctx.workspace_path().display());
    println!("  {}: {}", "Projects".green(), projects.len());
    println!("  {}: {:?}", "Provider".green(), engine.config.provider_type);
    println!("  {}: {}", "Model".green(), engine.config.model);
    println!("  {}: {}", "API Base".green(), engine.config.api_base.as_deref().unwrap_or("default"));
    println!("  {}: {}", "Cache".green(), if engine.config.cache_enabled { "Enabled" } else { "Disabled" });
    let index = crate::features::ingest::check_index_exists(&workspace_path());
    println!("  {}: {}", "Code Index".green(), if index { "Ready" } else { "Not built (run: polyglid-ai ingest)" });
    Ok(())
}

fn workspace_path() -> std::path::PathBuf {
    std::env::current_dir().unwrap_or_default()
}
