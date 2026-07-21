use std::path::Path;
use anyhow::Result;
use tokio::fs;
use serde::Serialize;

#[derive(Serialize)]
pub struct MermaidDiagram {
    pub filename: String,
    pub content: String,
}

pub struct DiagramGenerator;

impl DiagramGenerator {
    pub async fn generate_architecture_diagram(workspace_path: &Path) -> Result<MermaidDiagram> {
        let projects_dir = workspace_path.join("projects");
        let mut content = String::from("graph TD\n");

        if projects_dir.exists() {
            let mut rd = fs::read_dir(&projects_dir).await?;
            while let Some(entry) = rd.next_entry().await? {
                let name = entry.file_name().to_string_lossy().to_string();
                if entry.path().is_dir() && !name.starts_with('.') {
                    content.push_str(&format!("  {}[{}]\n", name.replace('-', "_").replace('.', "_"), name));
                    let mut sub = fs::read_dir(entry.path()).await?;
                    while let Some(sub_entry) = sub.next_entry().await? {
                        let sub_name = sub_entry.file_name().to_string_lossy().to_string();
                        if sub_entry.path().is_dir() && !sub_name.starts_with('.') {
                            let safe = sub_name.replace('-', "_").replace('.', "_");
                            content.push_str(&format!("  {}[{}]\n", safe, sub_name));
                            content.push_str(&format!("  {} --> {}\n", name.replace('-', "_").replace('.', "_"), safe));
                        }
                    }
                }
            }
        }

        Ok(MermaidDiagram {
            filename: "architecture-overview.md".to_string(),
            content: format!("```mermaid\n{}\n```\n\n## Projects\n\nAuto-generated dependency graph.", content),
        })
    }

    pub async fn generate_dependency_diagram(workspace_path: &Path) -> Result<MermaidDiagram> {
        let cargo = workspace_path.join("projects").join("Cargo.toml");
        let mut content = String::from("graph LR\n");

        if cargo.exists() {
            let text = fs::read_to_string(&cargo).await?;
            if let Ok(config) = toml::from_str::<toml::Value>(&text) {
                if let Some(deps) = config.get("dependencies").and_then(|d| d.as_table()) {
                    for (name, _) in deps {
                        let safe = name.replace('-', "_").replace('.', "_");
                        content.push_str(&format!("  {}[\"{}\"]\n", safe, name));
                    }
                }
            }
        }

        Ok(MermaidDiagram {
            filename: "dependency-graph.md".to_string(),
            content: format!("```mermaid\n{}\n```\n\n## Dependencies\n\nAuto-generated from Cargo.toml.", content),
        })
    }
}
