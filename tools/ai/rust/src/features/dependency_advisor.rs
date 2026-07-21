use std::sync::Arc;
use std::collections::HashMap;
use std::path::Path;
use anyhow::Result;
use crate::providers::Provider;
use crate::cache::CacheManager;
use crate::core::models::DependencyAnalysis;

pub struct DependencyAdvisor {
    provider: Arc<dyn Provider + Send + Sync>,
    cache: Arc<CacheManager>,
}

impl DependencyAdvisor {
    pub fn new(provider: Arc<dyn Provider + Send + Sync>, cache: Arc<CacheManager>) -> Self {
        Self { provider, cache }
    }

    pub async fn analyze_all(&self) -> Result<DependencyAnalysis> {
        let mut outdated = HashMap::new();
        let workspace = std::env::current_dir()?;

        for manifest in &["Cargo.toml", "package.json", "requirements.txt", "go.mod"] {
            let path = workspace.join(manifest);
            if path.exists() {
                let content = tokio::fs::read_to_string(&path).await?;

                if let Ok(info) = self.check_manifest(manifest, &content).await {
                    outdated.extend(info);
                }
            }
        }

        Ok(DependencyAnalysis { outdated })
    }

    async fn check_manifest(&self, name: &str, content: &str) -> Result<HashMap<String, crate::core::models::OutdatedInfo>> {
        let prompt = format!(
            r#"Given this {} dependency file, identify outdated or risky dependencies.
Return ONLY valid JSON:
{{"dependencies": {{
  "dep_name": {{"current": "1.0.0", "latest": "2.0.0", "language": "rust"}}
}}}}

File:
```
{}
```"#, name, content
        );

        let response = self.provider.generate(&prompt).await?;
        let cleaned = response.trim()
            .strip_prefix("```json").unwrap_or(response.trim())
            .strip_prefix("```").unwrap_or(response.trim())
            .strip_suffix("```").unwrap_or(response.trim())
            .trim()
            .to_string();

        #[derive(serde::Deserialize)]
        struct Outer { dependencies: HashMap<String, crate::core::models::OutdatedInfo> }

        if let Ok(parsed) = serde_json::from_str::<Outer>(&cleaned) {
            return Ok(parsed.dependencies);
        }

        Ok(HashMap::new())
    }
}
