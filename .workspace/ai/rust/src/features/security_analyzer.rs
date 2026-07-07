use std::sync::Arc;
use std::path::Path;
use anyhow::Result;
use crate::providers::Provider;
use crate::cache::CacheManager;
use crate::core::models::SecurityReport;

pub struct SecurityAnalyzer {
    provider: Arc<dyn Provider + Send + Sync>,
    cache: Arc<CacheManager>,
}

impl SecurityAnalyzer {
    pub fn new(provider: Arc<dyn Provider + Send + Sync>, cache: Arc<CacheManager>) -> Self {
        Self { provider, cache }
    }

    pub async fn analyze_workspace(&self) -> Result<SecurityReport> {
        Ok(SecurityReport {
            vulnerabilities: vec![],
        })
    }

    pub async fn analyze_file(&self, path: &Path) -> Result<SecurityReport> {
        let content = tokio::fs::read_to_string(path).await?;
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let prompt = format!(
            r#"Scan this code for security vulnerabilities. Return ONLY valid JSON:

{{
  "vulnerabilities": [
    {{
      "title": "vulnerability name",
      "description": "what it is and why it matters",
      "severity": <1-10>,
      "fix_action": "how to fix it"
    }}
  ]
}}

If no vulnerabilities found, return: {{"vulnerabilities": []}}

Code ({}):
```{}
{}
```"#, ext, ext, content
        );

        let response = self.provider.generate(&prompt).await?;
        let cleaned = response.trim()
            .strip_prefix("```json").unwrap_or(response.trim())
            .strip_prefix("```").unwrap_or(response.trim())
            .strip_suffix("```").unwrap_or(response.trim())
            .trim()
            .to_string();

        if let Ok(report) = serde_json::from_str::<SecurityReport>(&cleaned) {
            return Ok(report);
        }

        Ok(SecurityReport { vulnerabilities: vec![] })
    }
}
