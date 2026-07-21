use async_trait::async_trait;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait Provider {
    async fn generate(&self, prompt: &str) -> Result<String>;
    async fn analyze_code(&self, code: &str, language: &str) -> Result<CodeAnalysis>;
    async fn generate_tests(&self, code: &str, language: &str) -> Result<String>;
    async fn generate_documentation(&self, code: &str, language: &str) -> Result<String>;
    async fn embed(&self, input: &str) -> Result<Vec<f32>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysis {
    pub quality_score: f32,
    pub issues: Vec<String>,
    pub performance_issues: Vec<String>,
    pub security_issues: Vec<String>,
    pub suggestions: Vec<String>,
    pub raw_response: String,
}
