use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use crate::tools::ToolRegistry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub args: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum ToolResult {
    Text(String),
    FileContent { path: String, content: String, line_count: usize },
    SearchResults(Vec<SearchMatch>),
    FileList(Vec<String>),
    TestOutput { passed: usize, failed: usize, output: String },
    GitLog(Vec<CommitEntry>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMatch {
    pub file: String,
    pub line: usize,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitEntry {
    pub hash: String,
    pub author: String,
    pub message: String,
    pub date: String,
}

pub struct ToolExecutor {
    workspace: PathBuf,
}

impl ToolExecutor {
    pub fn new(workspace: &Path) -> Self {
        Self { workspace: workspace.to_path_buf() }
    }

    pub async fn execute(&self, call: &ToolCall) -> Result<ToolResult> {
        match call.name.as_str() {
            "read_file" => self.read_file(call).await,
            "search_code" => self.search_code(call).await,
            "list_files" => self.list_files(call).await,
            "run_test" => self.run_test(call).await,
            "read_git_log" => self.read_git_log(call).await,
            _ => Err(anyhow!("Unknown tool: {}", call.name)),
        }
    }

    pub fn describe_all(&self) -> String {
        let registry = ToolRegistry::new();
        registry.describe_all()
    }

    async fn read_file(&self, call: &ToolCall) -> Result<ToolResult> {
        let path = call.args.get("path").ok_or_else(|| anyhow!("Missing path"))?;
        let full = self.workspace.join(path);
        let content = tokio::fs::read_to_string(&full).await?;
        let lines: Vec<&str> = content.lines().collect();
        let start: usize = call.args.get("start_line").and_then(|s| s.parse().ok()).unwrap_or(1);
        let end: usize = call.args.get("end_line").and_then(|s| s.parse().ok()).unwrap_or(lines.len());

        let excerpt = lines[start.saturating_sub(1)..end.min(lines.len())].join("\n");
        Ok(ToolResult::FileContent {
            path: path.clone(),
            content: excerpt,
            line_count: end - start + 1,
        })
    }

    async fn search_code(&self, call: &ToolCall) -> Result<ToolResult> {
        let pattern = call.args.get("pattern").ok_or_else(|| anyhow!("Missing pattern"))?;
        let filter = call.args.get("path_filter").cloned().unwrap_or_default();
        let search_root = if filter.is_empty() {
            self.workspace.join("projects")
        } else {
            self.workspace.join(&filter)
        };

        let mut results = Vec::new();
        let mut stack = vec![search_root];
        while let Some(dir) = stack.pop() {
            if !dir.is_dir() { continue; }
            let mut rd = tokio::fs::read_dir(&dir).await?;
            while let Some(entry) = rd.next_entry().await? {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if !name.starts_with('.') && name != "node_modules" && name != "target" {
                        stack.push(path);
                    }
                } else if let Ok(content) = tokio::fs::read_to_string(&path).await {
                    for (i, line) in content.lines().enumerate() {
                        if line.contains(pattern) {
                            let rel = path.strip_prefix(&self.workspace)
                                .unwrap_or(&path).to_string_lossy().to_string();
                            results.push(SearchMatch { file: rel, line: i + 1, content: line.to_string() });
                            if results.len() >= 50 { break; }
                        }
                    }
                }
            }
        }

        Ok(ToolResult::SearchResults(results))
    }

    async fn list_files(&self, call: &ToolCall) -> Result<ToolResult> {
        let path = call.args.get("path").ok_or_else(|| anyhow!("Missing path"))?;
        let recursive = call.args.get("recursive").map(|s| s == "true").unwrap_or(false);
        let full = self.workspace.join(path);
        let mut files = Vec::new();

        if recursive {
            let mut stack = vec![full];
            while let Some(dir) = stack.pop() {
                if !dir.is_dir() { continue; }
                let mut rd = tokio::fs::read_dir(&dir).await?;
                while let Some(entry) = rd.next_entry().await? {
                    let p = entry.path();
                    let rel = p.strip_prefix(&self.workspace).unwrap_or(&p).to_string_lossy().to_string();
                    files.push(rel);
                    if p.is_dir() {
                        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        if !name.starts_with('.') && name != "node_modules" && name != "target" {
                            stack.push(p);
                        }
                    }
                }
            }
        } else {
            let mut rd = tokio::fs::read_dir(&full).await?;
            while let Some(entry) = rd.next_entry().await? {
                let p = entry.path();
                let rel = p.strip_prefix(&self.workspace).unwrap_or(&p).to_string_lossy().to_string();
                files.push(rel);
            }
        }

        files.sort();
        Ok(ToolResult::FileList(files))
    }

    async fn run_test(&self, call: &ToolCall) -> Result<ToolResult> {
        let target = call.args.get("target").ok_or_else(|| anyhow!("Missing target"))?;
        let _lang = call.args.get("language").map(|s| s.as_str()).unwrap_or("rust");

        let output = tokio::process::Command::new("cargo")
            .args(["test", target])
            .current_dir(&self.workspace)
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let passed = stdout.matches("ok").count();
        let failed = stdout.matches("FAILED").count();

        Ok(ToolResult::TestOutput { passed, failed, output: stdout })
    }

    async fn read_git_log(&self, call: &ToolCall) -> Result<ToolResult> {
        let limit = call.args.get("limit").and_then(|s| s.parse().ok()).unwrap_or(10);

        let output = tokio::process::Command::new("git")
            .args(["log", &format!("--max-count={}", limit), "--format=%H|%an|%s|%ar"])
            .current_dir(&self.workspace)
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let entries: Vec<CommitEntry> = stdout.lines().filter_map(|line| {
            let parts: Vec<&str> = line.splitn(4, '|').collect();
            if parts.len() == 4 {
                Some(CommitEntry {
                    hash: parts[0].to_string(),
                    author: parts[1].to_string(),
                    message: parts[2].to_string(),
                    date: parts[3].to_string(),
                })
            } else { None }
        }).collect();

        Ok(ToolResult::GitLog(entries))
    }
}
