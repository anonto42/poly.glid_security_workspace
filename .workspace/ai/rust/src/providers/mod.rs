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
        match config.provider_type {
            ProviderType::Local => {
                let provider = LocalProvider::new(None)?;
                Ok(Arc::new(provider))
            }
            _ => {
                let provider = OpenAIProvider::new(None)?
                    .with_model(&config.model)
                    .with_temperature(config.temperature);
                Ok(Arc::new(provider))
            }
        }
    }
}
