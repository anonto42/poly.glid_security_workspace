use std::path::Path;
use anyhow::Result;
use tokio::fs;
use serde::Serialize;

#[derive(Serialize)]
pub struct GitignoreConfig {
    pub content: String,
}

#[derive(Serialize)]
pub struct EditorconfigConfig {
    pub content: String,
}

#[derive(Serialize)]
pub struct VscodeSettings {
    pub content: String,
}

pub struct ConfigGenerator;

impl ConfigGenerator {
    pub async fn generate_gitignore(workspace_path: &Path) -> Result<GitignoreConfig> {
        let mut ignores = vec![
            "# PolyGlid AI workspace",
            ".workspace/state/cache/",
            ".workspace/state/temp/",
            ".workspace/state/logs/",
            "tools/ai/rust/target/",
            ".workspace/data/backups/",
            "",
            "# Dependencies",
            "node_modules/",
            "",
            "# Build outputs",
            "target/",
            "dist/",
            "build/",
            "",
            "# IDE",
            ".vscode/",
            ".idea/",
            "*.swp",
            "*.swo",
            "",
            "# Environment",
            ".env",
            ".env.local",
        ];

        // Auto-detect languages from canonical product roots.
        for root in ["apps", "crates", "plugins", "site", "sdk"] {
            let projects = workspace_path.join(root);
            if !projects.exists() { continue; }
            let mut rd = fs::read_dir(projects).await?;
            while let Some(entry) = rd.next_entry().await? {
                let path = entry.path();
                if path.is_dir() {
                    if path.join("Cargo.toml").exists() {
                        ignores.push("");
                        ignores.push("# Rust");
                        ignores.push("target/");
                        ignores.push("Cargo.lock");
                    }
                    if path.join("package.json").exists() {
                        ignores.push("");
                        ignores.push("# Node.js");
                        ignores.push("node_modules/");
                        ignores.push("dist/");
                        ignores.push(".next/");
                    }
                    if path.join("requirements.txt").exists() || path.join("pyproject.toml").exists() {
                        ignores.push("");
                        ignores.push("# Python");
                        ignores.push("__pycache__/");
                        ignores.push("*.pyc");
                        ignores.push(".venv/");
                        ignores.push("venv/");
                    }
                }
            }
        }

        Ok(GitignoreConfig { content: ignores.join("\n") })
    }

    pub async fn generate_editorconfig() -> EditorconfigConfig {
        let content = r#"root = true

[*]
indent_style = space
indent_size = 4
end_of_line = lf
charset = utf-8
trim_trailing_whitespace = true
insert_final_newline = true

[*.{yml,yaml,toml,json}]
indent_size = 2

[Makefile]
indent_style = tab
"#.to_string();

        EditorconfigConfig { content }
    }

    pub async fn generate_vscode_settings() -> VscodeSettings {
        let content = r#"{
  "editor.formatOnSave": true,
  "editor.rulers": [100],
  "editor.tabSize": 4,
  "files.trimTrailingWhitespace": true,
  "files.insertFinalNewline": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "[toml]": {
    "editor.defaultFormatter": "tamasfe.even-better-toml"
  }
}"#.to_string();

        VscodeSettings { content }
    }
}
