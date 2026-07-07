use std::sync::Arc;
use anyhow::Result;
use crate::providers::Provider;
use crate::cache::CacheManager;
use crate::core::models::TestGeneration;

pub struct TestGenerator {
    _provider: Arc<dyn Provider + Send + Sync>,
    _cache: Arc<CacheManager>,
}

impl TestGenerator {
    pub fn new(provider: Arc<dyn Provider + Send + Sync>, cache: Arc<CacheManager>) -> Self {
        Self { _provider: provider, _cache: cache }
    }

    pub async fn generate(&self, _code: &str, _language: &str) -> Result<TestGeneration> {
        Ok(TestGeneration {
            raw_code: "fn test() {}".to_string(),
        })
    }
}
