//! PolyGlid AI CLI - Main Entry Point

use clap::{Parser, Subcommand};
use colored::*;
use anyhow::Result;

mod core;
mod providers;
mod features;
mod cache;
mod cli;

use crate::core::engine::AIEngine;
use crate::cli::commands::*;

/// PolyGlid AI Command Line Interface
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
    /// Analyze workspace or files
    Analyze {
        /// Specific file to analyze
        #[arg(long)]
        file: Option<String>,
        
        /// Output format (json, table)
        #[arg(long, default_value = "table")]
        format: String,
    },
    
    /// Generate code, tests, or documentation
    Generate {
        #[command(subcommand)]
        generate_type: GenerateType,
    },
    
    /// Review code
    Review {
        /// File to review
        #[arg()]
        file: String,
    },
    
    /// Get AI suggestions
    Suggest {
        /// Number of suggestions
        #[arg(long, default_value = "10")]
        limit: usize,
    },
    
    /// Optimize builds
    Optimize {
        /// Target to optimize
        #[arg(default_value = "build")]
        target: String,
    },
    
    /// Security analysis
    Security {
        /// File to scan
        #[arg(long)]
        file: Option<String>,
    },
    
    /// Get workspace status
    Status,
}

#[derive(Subcommand)]
enum GenerateType {
    /// Generate code
    Code {
        /// Description of code to generate
        #[arg()]
        description: String,
        
        /// Programming language
        #[arg(long)]
        language: String,
        
        /// Output file
        #[arg(long)]
        output: Option<String>,
    },
    
    /// Generate tests
    Tests {
        /// File to generate tests for
        #[arg()]
        file: String,
        
        /// Output file
        #[arg(long)]
        output: Option<String>,
    },
    
    /// Generate documentation
    Docs {
        /// File to generate docs for
        #[arg()]
        file: String,
        
        /// Output file
        #[arg(long)]
        output: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Parse CLI
    let cli = Cli::parse();
    
    // Initialize AI Engine
    let workspace_path = std::env::current_dir()?;
    let engine = AIEngine::new(&workspace_path).await?;
    
    // Execute command
    match cli.command {
        Commands::Analyze { file, format } => {
            if let Some(file_path) = file {
                analyze_file(&engine, &file_path).await?;
            } else {
                analyze_workspace(&engine, &format).await?;
            }
        }
        
        Commands::Generate { generate_type } => {
            match generate_type {
                GenerateType::Code { description, language, output } => {
                    generate_code(&engine, &description, &language, output).await?;
                }
                GenerateType::Tests { file, output } => {
                    generate_tests(&engine, &file, output).await?;
                }
                GenerateType::Docs { file, output } => {
                    generate_docs(&engine, &file, output).await?;
                }
            }
        }
        
        Commands::Review { file } => {
            review_code(&engine, &file).await?;
        }
        
        Commands::Suggest { limit } => {
            get_suggestions(&engine, limit).await?;
        }
        
        Commands::Optimize { target } => {
            optimize_build(&engine, &target).await?;
        }
        
        Commands::Security { file } => {
            if let Some(file_path) = file {
                security_scan_file(&engine, &file_path).await?;
            } else {
                security_scan_workspace(&engine).await?;
            }
        }
        
        Commands::Status => {
            show_status(&engine).await?;
        }
    }
    
    Ok(())
}

/// Show workspace status
async fn show_status(engine: &AIEngine) -> Result<()> {
    println!("{}", "\n📊 AI Workspace Status".bold().cyan());
    println!("{}", "═".repeat(50).cyan());
    
    let context = engine.context.read().await;
    let projects = context.get_projects().await?;
    
    println!("  {}: {}", "Workspace".green(), context.workspace_path().display());
    println!("  {}: {}", "Projects".green(), projects.len());
    println!("  {}: {}", "AI Provider".green(), "OpenAI"); // TODO: Get from config
    println!("  {}: {}", "Cache".green(), if engine.config.cache_enabled { "Enabled" } else { "Disabled" });
    
    Ok(())
}
