use std::sync::Arc;
use anyhow::Result;
use crate::providers::Provider;
use crate::cache::CacheManager;
use crate::core::models::TestGeneration;

pub struct TestGenerator {
    provider: Arc<dyn Provider + Send + Sync>,
    _cache: Arc<CacheManager>,
}

impl TestGenerator {
    pub fn new(provider: Arc<dyn Provider + Send + Sync>, cache: Arc<CacheManager>) -> Self {
        Self { provider, _cache: cache }
    }

    pub async fn generate(&self, code: &str, language: &str) -> Result<TestGeneration> {
        let raw_code = self.provider.generate_tests(code, language).await?;
        Ok(TestGeneration { raw_code })
    }
}
