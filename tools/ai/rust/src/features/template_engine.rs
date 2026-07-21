use std::path::Path;
use anyhow::Result;
use tokio::fs;
use serde::Serialize;

#[derive(Serialize)]
pub struct GeneratedMakefile {
    pub project: String,
    pub language: String,
    pub content: String,
}

pub struct TemplateEngine;

impl TemplateEngine {
    async fn is_project_dir(path: &Path) -> bool {
        path.join("Cargo.toml").exists()
            || path.join("package.json").exists()
            || path.join("pyproject.toml").exists()
            || path.join("go.mod").exists()
    }

    async fn detect_language(path: &Path) -> Option<String> {
        if path.join("Cargo.toml").exists() { Some("rust".to_string()) }
        else if path.join("package.json").exists() { Some("node".to_string()) }
        else if path.join("pyproject.toml").exists() { Some("python".to_string()) }
        else if path.join("go.mod").exists() { Some("go".to_string()) }
        else { None }
    }

    pub async fn generate_project_mk(workspace_path: &Path) -> Result<Vec<GeneratedMakefile>> {
        let template_path = workspace_path
            .join("tools/automation/templates/project.mk.template");
        let project_roots = ["apps", "crates", "plugins", "site", "sdk"]
            .into_iter().map(|name| workspace_path.join(name)).filter(|path| path.exists()).collect::<Vec<_>>();

        if !template_path.exists() {
            return Err(anyhow::anyhow!("Template not found: {:?}", template_path));
        }
        if project_roots.is_empty() {
            return Ok(Vec::new());
        }

        let template = fs::read_to_string(&template_path).await?;
        let mut results = Vec::new();

        // Walk every canonical product root and detect real projects.
        let mut stack = project_roots;
        while let Some(current) = stack.pop() {
            if !current.is_dir() { continue; }
            let mut rd = fs::read_dir(&current).await?;
            while let Some(entry) = rd.next_entry().await? {
                let path = entry.path();
                if !path.is_dir() { continue; }
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                if name.starts_with('.') || name == "node_modules" || name == "target" || name == "dist" {
                    continue;
                }
            if Self::is_project_dir(&path).await {
                // Found a real project — generate template
                let language = Self::detect_language(&path).await.unwrap_or_else(|| "unknown".to_string());
                let project_dir = path.strip_prefix(workspace_path)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();
                let content = template
                    .replace("{{PROJECT_NAME}}", &name)
                    .replace("{{PROJECT_LANGUAGE}}", &language)
                    .replace("{{PROJECT_DIR}}", &project_dir);
                results.push(GeneratedMakefile {
                    project: name.clone(),
                    language,
                    content,
                });
            }
            // Always recurse to find nested projects (e.g. src-tauri inside desktop-tauri)
            stack.push(path);
            }
        }

        Ok(results)
    }

    pub async fn generate_language_mk(workspace_path: &Path) -> Result<Vec<GeneratedMakefile>> {
        let template_path = workspace_path
            .join("tools/automation/templates/language.mk.template");
        let project_roots = ["apps", "crates", "plugins", "site", "sdk"]
            .into_iter().map(|name| workspace_path.join(name)).filter(|path| path.exists()).collect::<Vec<_>>();

        if !template_path.exists() {
            return Err(anyhow::anyhow!("Template not found: {:?}", template_path));
        }
        if project_roots.is_empty() {
            return Ok(Vec::new());
        }

        let template = fs::read_to_string(&template_path).await?;
        let mut results = Vec::new();
        let mut seen_languages = std::collections::HashSet::new();

        // Walk and find all unique languages used by real projects
        let mut stack = project_roots;
        while let Some(current) = stack.pop() {
            if !current.is_dir() { continue; }
            if let Ok(mut rd) = fs::read_dir(&current).await {
                while let Ok(Some(entry)) = rd.next_entry().await {
                    let path = entry.path();
                    if !path.is_dir() { continue; }
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                    if name.starts_with('.') || name == "node_modules" || name == "target" || name == "dist" {
                        continue;
                    }
                    if Self::is_project_dir(&path).await {
                        if let Some(lang) = Self::detect_language(&path).await {
                            if seen_languages.insert(lang.clone()) {
                                let content = template.replace("{{LANGUAGE}}", &lang);
                                results.push(GeneratedMakefile {
                                    project: format!("{}-language", lang),
                                    language: lang,
                                    content,
                                });
                            }
                        }
                    } else {
                        stack.push(path);
                    }
                }
            }
        }

        Ok(results)
    }
}
