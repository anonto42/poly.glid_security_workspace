use std::sync::Arc;
use std::path::Path;
use anyhow::Result;
use crate::providers::Provider;
use crate::cache::CacheManager;
use crate::core::models::{CodeQualityAnalysis, LocalAnalysis};

pub struct CodeAnalyzer {
    provider: Arc<dyn Provider + Send + Sync>,
    cache: Arc<CacheManager>,
}

impl CodeAnalyzer {
    pub fn new(provider: Arc<dyn Provider + Send + Sync>, cache: Arc<CacheManager>) -> Self {
        Self { provider, cache }
    }

    pub async fn analyze_all(&self) -> Result<CodeQualityAnalysis> {
        Ok(CodeQualityAnalysis {
            average_score: 0.0,
            files_analyzed: 0,
        })
    }

    pub async fn analyze_file(&self, path: &Path) -> Result<LocalAnalysis> {
        let content = tokio::fs::read_to_string(path).await?;
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let analysis = self.provider.analyze_code(&content, ext).await?;
        Ok(LocalAnalysis {
            score: analysis.quality_score,
        })
    }
}
