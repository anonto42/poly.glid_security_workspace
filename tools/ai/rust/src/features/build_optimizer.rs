use std::sync::Arc;
use anyhow::Result;
use crate::providers::Provider;
use crate::cache::CacheManager;
use crate::core::models::{BuildOptimization, BuildHistory};

pub struct BuildOptimizer {
    provider: Arc<dyn Provider + Send + Sync>,
    cache: Arc<CacheManager>,
}

impl BuildOptimizer {
    pub fn new(provider: Arc<dyn Provider + Send + Sync>, cache: Arc<CacheManager>) -> Self {
        Self { provider, cache }
    }

    pub async fn optimize(&self, _history: &BuildHistory) -> Result<BuildOptimization> {
        let build_files = self.read_build_files().await?;
        if build_files.is_empty() {
            return Ok(BuildOptimization { _suggestions: vec![] });
        }

        let prompt = format!(
            r#"Review these build configuration files and suggest optimizations.
Return ONLY valid JSON:
{{"suggestions": ["suggestion1", "suggestion2"]}}

Files:
```
{}
```"#, build_files
        );

        let response = self.provider.generate(&prompt).await?;
        let cleaned = response.trim()
            .strip_prefix("```json").unwrap_or(response.trim())
            .strip_prefix("```").unwrap_or(response.trim())
            .strip_suffix("```").unwrap_or(response.trim())
            .trim()
            .to_string();

        #[derive(serde::Deserialize)]
        struct Suggestions { suggestions: Vec<String> }

        if let Ok(parsed) = serde_json::from_str::<Suggestions>(&cleaned) {
            return Ok(BuildOptimization { _suggestions: parsed.suggestions });
        }

        Ok(BuildOptimization { _suggestions: vec![] })
    }

    pub async fn apply_optimizations(&self, opt: &BuildOptimization) -> Result<()> {
        for s in &opt._suggestions {
            println!("  💡 Optimization: {}", s);
        }
        Ok(())
    }

    async fn read_build_files(&self) -> Result<String> {
        let workspace = std::env::current_dir()?;
        let mut parts = Vec::new();

        for path in &["Makefile", "Cargo.toml", ".cargo/config.toml"] {
            let p = workspace.join(path);
            if p.exists() {
                let content = tokio::fs::read_to_string(&p).await?;
                parts.push(format!("--- {} ---\n{}", path, content));
            }
        }

        Ok(parts.join("\n\n"))
    }
}
