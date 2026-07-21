use anyhow::Result;
use crate::core::engine::AIEngine;
use crate::tools::{ToolCall, ToolResult};
use crate::feedback::{PredictionStatus, Prediction};
use crate::pipelines::PipelineManager;
use std::path::Path;
use colored::*;
use std::collections::HashMap;
use tokio::fs;
use chrono::Utc;

fn report_dir(engine: &AIEngine, sub: &str) -> std::path::PathBuf {
    engine.workspace_path.join(".workspace").join(sub)
}

async fn ensure_dir(path: &std::path::Path) -> Result<()> {
    fs::create_dir_all(path).await?;
    Ok(())
}

pub async fn log_analytics(engine: &AIEngine, command: &str, duration_ms: u64, status: &str) -> Result<()> {
    let dir = report_dir(engine, "data/analytics");
    ensure_dir(&dir).await?;
    let path = dir.join("usage.jsonl");
    let entry = serde_json::json!({
        "command": command,
        "timestamp": Utc::now().to_rfc3339(),
        "duration_ms": duration_ms,
        "status": status,
    });
    let mut file = fs::OpenOptions::new()
        .create(true).append(true).open(&path).await?;
    use tokio::io::AsyncWriteExt;
    file.write_all(format!("{}\n", serde_json::to_string(&entry)?).as_bytes()).await?;
    Ok(())
}

pub async fn analyze_file(_engine: &AIEngine, path: &str) -> Result<()> {
    println!("Analyzing file: {}", path.bold().yellow());
    Ok(())
}

pub async fn analyze_workspace(engine: &AIEngine, _format: &str) -> Result<()> {
    println!("Analyzing workspace via AI...");
    let report = engine.analyze_workspace().await?;
    println!("Workspace analysis complete! Score: {:.1}", report.code_quality.average_score);

    let dir = report_dir(engine, "quality/reports");
    ensure_dir(&dir).await?;
    let path = dir.join(format!("{}-analysis.json", report.timestamp.format("%Y%m%d_%H%M%S")));
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, &json).await?;
    println!("{} Report saved to: {}", "✓".green(), path.display());

    Ok(())
}

pub async fn generate_code(engine: &AIEngine, desc: &str, lang: &str, output: Option<String>) -> Result<()> {
    println!("Generating code...");
    let code = engine.generate_code(desc, lang).await?;
    if let Some(out) = output {
        tokio::fs::write(&out, &code).await?;
        println!("Saved generated code to: {}", out.bold().green());
    } else {
        println!("\n{}\n", code);
    }
    Ok(())
}

pub async fn generate_tests(engine: &AIEngine, file: &str, output: Option<String>) -> Result<()> {
    println!("Generating tests...");
    let tests = engine.generate_tests(Path::new(file)).await?;
    if let Some(out) = output {
        tokio::fs::write(&out, &tests.raw_code).await?;
        println!("Saved generated tests to: {}", out.bold().green());
    } else {
        println!("\n{}\n", tests.raw_code);
    }
    Ok(())
}

pub async fn generate_docs(engine: &AIEngine, file: &str, output: Option<String>) -> Result<()> {
    println!("Generating documentation...");
    let doc = engine.provider.generate_documentation(file, "rust").await?;
    if let Some(out) = output {
        tokio::fs::write(&out, &doc).await?;
        println!("Saved documentation to: {}", out.bold().green());
    } else {
        println!("\n{}\n", doc);
    }
    Ok(())
}

pub async fn review_code(engine: &AIEngine, file: &str) -> Result<()> {
    println!("Reviewing code...");
    let review = engine.review_code(Path::new(file)).await?;
    println!("Code review for: {:?}", review.file);
    println!("Score: {:.1}", review.quality_score);
    Ok(())
}

pub async fn get_suggestions(engine: &AIEngine, limit: usize) -> Result<()> {
    println!("Fetching suggestions...");
    let suggestions = engine.get_suggestions(limit).await?;
    for s in &suggestions {
        println!("- [{}] Priority {}: {}", s.category.yellow(), s.priority, s.title.bold());
        println!("  {}", s.description);
    }
    Ok(())
}

pub async fn optimize_build(engine: &AIEngine, _target: &str) -> Result<()> {
    println!("Optimizing builds...");
    let opt = engine.optimize_build().await?;
    println!("Build optimization complete! {} suggestions found.", opt._suggestions.len());
    Ok(())
}

pub async fn security_scan_file(_engine: &AIEngine, path: &str) -> Result<()> {
    println!("Scanning file for security vulnerabilities: {}", path.bold().yellow());
    Ok(())
}

pub async fn security_scan_workspace(engine: &AIEngine) -> Result<()> {
    println!("Running workspace security scan...");
    let report = engine.security_analyzer.analyze_workspace().await?;
    println!("Workspace security scan complete! {} vulnerabilities found.", report.vulnerabilities.len());

    let dir = report_dir(engine, "security/audits");
    ensure_dir(&dir).await?;
    let ts = Utc::now().format("%Y%m%d_%H%M%S");
    let path = dir.join(format!("{}-audit.json", ts));
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&path, &json).await?;
    println!("{} Audit saved to: {}", "✓".green(), path.display());

    Ok(())
}

pub async fn ingest_workspace(engine: &AIEngine) -> Result<()> {
    println!("{}", "\n📥 Ingesting workspace code...".bold().cyan());
    let ws = engine.context.read().await.workspace_path().to_path_buf();
    let count = engine.ingest_service.ingest_workspace(&ws).await?;
    println!("  {} chunks indexed", count.len().to_string().green().bold());
    Ok(())
}

pub async fn search_index(engine: &AIEngine, query: &str, limit: usize) -> Result<()> {
    println!("{}", "\n🔍 Searching code index...".bold().cyan());
    let ws = engine.context.read().await.workspace_path().to_path_buf();
    let results = engine.ingest_service.search(query, &ws, limit).await?;
    if results.is_empty() {
        println!("  No results found. Run 'polyglid-ai ingest' first.");
        return Ok(());
    }
    for (i, r) in results.iter().enumerate() {
        println!("  {}. {}:{}", (i+1).to_string().green(), r.chunk.file, r.chunk.start_line);
        println!("     {}", r.chunk.content.lines().next().unwrap_or("").trim());
    }
    Ok(())
}

pub async fn run_tool(engine: &AIEngine, name: &str, args: &[String]) -> Result<()> {
    let mut map = HashMap::new();
    for pair in args {
        if let Some((k, v)) = pair.split_once('=') {
            map.insert(k.to_string(), v.to_string());
        }
    }
    let call = ToolCall { name: name.to_string(), args: map };
    match engine.tool_executor.execute(&call).await? {
        ToolResult::Text(t) => println!("{}", t),
        ToolResult::FileContent { path, content, line_count } => {
            println!("{} ({}, {} lines):", path.blue(), line_count.to_string().green(), content.lines().count());
            println!("{}", content);
        }
        ToolResult::SearchResults(results) => {
            println!("{} results:", results.len().to_string().green());
            for r in results.iter().take(20) {
                println!("  {}:{} — {}", r.file.blue(), r.line.to_string().yellow(), r.content.trim());
            }
        }
        ToolResult::FileList(files) => {
            for f in files {
                println!("{}", f);
            }
        }
        ToolResult::TestOutput { passed, failed, output } => {
            println!("{} passed, {} failed", passed.to_string().green(), failed.to_string().red());
            println!("{}", output);
        }
        ToolResult::GitLog(entries) => {
            for e in &entries {
                println!("{} {} ({})", e.hash[..8].yellow(), e.message, e.date.green());
            }
        }
    }
    Ok(())
}

pub async fn list_tools(engine: &AIEngine) -> Result<()> {
    let registry = engine.tool_executor.describe_all();
    println!("{}", registry);
    Ok(())
}

pub async fn handle_feedback(engine: &AIEngine, action: &str, arg: Option<&str>) -> Result<()> {
    let tracker = &engine.feedback_tracker;
    match action {
        "list" => {
            let predictions = tracker.list_predictions(arg).await?;
            if predictions.is_empty() {
                println!("{}", "No predictions found.".yellow());
                return Ok(());
            }
            for p in &predictions {
                let status_icon = match p.status {
                    PredictionStatus::Accepted => "✓".green(),
                    PredictionStatus::Dismissed => "✗".red(),
                    PredictionStatus::Pending => "○".yellow(),
                };
                println!("{} [{}] {}: {} — {}", p.id.blue(), status_icon, p.category, 
                    p.input.chars().take(60).collect::<String>(), p.timestamp);
            }
        }
        "show" => {
            let id = arg.ok_or_else(|| anyhow::anyhow!("Usage: feedback show <id>"))?;
            let p = tracker.get_prediction(id).await?;
            println!("ID:       {}", p.id.blue());
            println!("Category: {}", p.category.green());
            println!("Time:     {}", p.timestamp.yellow());
            println!("Status:   {:?}", p.status);
            println!("Input:\n{}", p.input);
            println!("Output:\n{}", p.output);
        }
        "accept" => {
            let id = arg.ok_or_else(|| anyhow::anyhow!("Usage: feedback accept <id>"))?;
            let p = tracker.update_status(id, PredictionStatus::Accepted).await?;
            println!("{} Accepted prediction {}", "✓".green(), p.id.blue());
        }
        "dismiss" => {
            let id = arg.ok_or_else(|| anyhow::anyhow!("Usage: feedback dismiss <id>"))?;
            let p = tracker.update_status(id, PredictionStatus::Dismissed).await?;
            println!("{} Dismissed prediction {}", "✗".red(), p.id.blue());
        }
        _ => {
            println!("Unknown action: {}. Use: list, show <id>, accept <id>, dismiss <id>", action);
        }
    }
    Ok(())
}

pub async fn start_daemon(engine: &AIEngine) -> Result<()> {
    let workspace = engine.workspace_path.clone();
    println!("{} Starting pipeline daemon for {}", "⟳".cyan(), workspace.display());
    println!("  {} File watcher (apps/crates/plugins)", "●".green());
    println!("  {} Scheduled tasks (hourly/daily)", "●".green());
    println!("  {} Auto-suggester (every 5 min)", "●".green());
    println!("{} Press Ctrl+C to stop", "!".yellow());

    let mut manager = PipelineManager::new(workspace);
    manager.start().await?;
    Ok(())
}

pub async fn generate_diagrams(engine: &AIEngine) -> Result<()> {
    println!("{} Generating architecture diagrams...", "📐".cyan());
    let ws = &engine.workspace_path;
    let dir = report_dir(engine, "docs/diagrams");
    ensure_dir(&dir).await?;

    let arch = crate::features::DiagramGenerator::generate_architecture_diagram(ws).await?;
    let dep = crate::features::DiagramGenerator::generate_dependency_diagram(ws).await?;

    fs::write(dir.join(&arch.filename), &arch.content).await?;
    println!("  {} {} saved", "✓".green(), arch.filename);
    fs::write(dir.join(&dep.filename), &dep.content).await?;
    println!("  {} {} saved", "✓".green(), dep.filename);
    Ok(())
}

pub async fn generate_release_manifests(engine: &AIEngine) -> Result<()> {
    println!("{} Generating release manifests...", "📦".cyan());
    let ws = &engine.workspace_path;
    let dir = report_dir(engine, "releases/manifests");
    ensure_dir(&dir).await?;

    let manifests = crate::features::ReleasePlanner::analyze(ws).await?;
    if manifests.is_empty() {
        println!("  {} No releasable components found", "!".yellow());
        return Ok(());
    }
    for m in &manifests {
        let filename = format!("{}-deployment.yaml", m.project);
        fs::write(dir.join(&filename), &m.manifest_yaml).await?;
        println!("  {} {} v{} → {}", "✓".green(), m.project.cyan(), m.version.yellow(), filename);
    }
    Ok(())
}

pub async fn init_configs(engine: &AIEngine) -> Result<()> {
    println!("{} Generating workspace configs...", "⚙".cyan());
    let ws = &engine.workspace_path;

    // .gitignore -> configs/git/
    let git_dir = report_dir(engine, "configs/git");
    ensure_dir(&git_dir).await?;
    let gitignore = crate::features::ConfigGenerator::generate_gitignore(ws).await?;
    let git_path = ws.join(".gitignore");
    fs::write(&git_path, &gitignore.content).await?;
    println!("  {} .gitignore written", "✓".green());
    fs::write(git_dir.join(".gitignore"), &gitignore.content).await?;
    println!("  {} configs/git/.gitignore written", "✓".green());

    // .editorconfig -> configs/ide/
    let ide_dir = report_dir(engine, "configs/ide");
    ensure_dir(&ide_dir).await?;
    let ec = crate::features::ConfigGenerator::generate_editorconfig().await;
    let ec_path = ws.join(".editorconfig");
    fs::write(&ec_path, &ec.content).await?;
    println!("  {} .editorconfig written", "✓".green());
    fs::write(ide_dir.join(".editorconfig"), &ec.content).await?;
    println!("  {} configs/ide/.editorconfig written", "✓".green());

    // .vscode/settings.json -> configs/ide/
    let vs = crate::features::ConfigGenerator::generate_vscode_settings().await;
    let vscode_dir = ws.join(".vscode");
    fs::create_dir_all(&vscode_dir).await?;
    fs::write(vscode_dir.join("settings.json"), &vs.content).await?;
    println!("  {} .vscode/settings.json written", "✓".green());
    fs::write(ide_dir.join("vscode-settings.json"), &vs.content).await?;
    println!("  {} configs/ide/vscode-settings.json written", "✓".green());

    Ok(())
}

pub async fn generate_mk_templates(engine: &AIEngine) -> Result<()> {
    println!("{} Generating Makefile templates...", "🔧".cyan());
    let ws = &engine.workspace_path;
    let out_dir = ws.join("tools/automation/includes/projects");
    fs::create_dir_all(&out_dir).await?;

    let projects = crate::features::TemplateEngine::generate_project_mk(ws).await?;
    let languages = crate::features::TemplateEngine::generate_language_mk(ws).await?;

    for p in &projects {
        let filename = format!("{}.mk", p.project);
        fs::write(out_dir.join(&filename), &p.content).await?;
        println!("  {} {} ({})", "✓".green(), filename, p.language);
    }
    for l in &languages {
        let filename = format!("{}.mk", l.project);
        fs::write(out_dir.join(&filename), &l.content).await?;
        println!("  {} {} (language)", "✓".green(), filename);
    }

    if projects.is_empty() && languages.is_empty() {
        println!("  {} No components found in canonical product roots", "!".yellow());
    }
    Ok(())
}

pub async fn detect_changes(engine: &AIEngine, base: &str) -> Result<()> {
    println!("{} Detecting changes from {}...", "🔍".cyan(), base);
    let changed = engine.detect_changes(Some(base)).await?;
    if changed.is_empty() {
        println!("  {} No changes detected", "○".yellow());
        return Ok(());
    }
    for path in &changed {
        let rel = path.strip_prefix(&engine.workspace_path).unwrap_or(path);
        println!("  {} {}", "📄".cyan(), rel.display());
    }
    println!("  {} {} projects changed", "✓".green(), changed.len());
    Ok(())
}
