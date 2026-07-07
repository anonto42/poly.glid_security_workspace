pub mod traits;
pub mod openai;
pub mod local;

pub use traits::{Provider, CodeAnalysis};
pub use openai::OpenAIProvider;
pub use local::LocalProvider;

use anyhow::Result;
use crate::core::engine::{EngineConfig, ProviderType};
use std::sync::Arc;

pub struct ProviderFactory;

impl ProviderFactory {
    pub async fn create(config: &EngineConfig) -> Result<Arc<dyn Provider + Send + Sync>> {
        let api_base = config.api_base.as_deref()
            .unwrap_or("http://localhost:11434/v1");

        let api_key = config.api_key.clone();

        match config.provider_type {
            ProviderType::Local => {
                let provider = LocalProvider::new(None)?;
                Ok(Arc::new(provider))
            }
            _ => {
                let provider = OpenAIProvider::new(api_base, api_key)?
                    .with_model(&config.model)
                    .with_temperature(config.temperature)
                    .with_max_tokens(config.max_tokens);
                Ok(Arc::new(provider))
            }
        }
    }
}
