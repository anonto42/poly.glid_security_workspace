//! OpenAI-compatible Provider (works with OpenAI API & Ollama local API)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use reqwest::Client;
use std::env;

use super::traits::{Provider, CodeAnalysis};

/// OpenAI-compatible Provider
///
/// Works with:
/// - OpenAI API (https://api.openai.com/v1)
/// - Ollama local API (http://localhost:11434/v1)
/// - Any OpenAI-compatible endpoint
pub struct OpenAIProvider {
    client: Client,
    api_key: Option<String>,
    api_base: String,
    model: String,
    temperature: f32,
    max_tokens: usize,
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
    max_tokens: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

impl OpenAIProvider {
    /// Create new OpenAI-compatible provider
    ///
    /// - `api_base`: API endpoint base URL (e.g. "https://api.openai.com/v1" or "http://localhost:11434/v1")
    /// - `api_key`: Optional API key (not needed for Ollama)
    pub fn new(api_base: &str, api_key: Option<String>) -> Result<Self> {
        let api_key = api_key
            .or_else(|| env::var("OPENAI_API_KEY").ok())
            .filter(|k| !k.is_empty());

        if api_base.contains("api.openai.com") && api_key.is_none() {
            return Err(anyhow!("OPENAI_API_KEY required for OpenAI API endpoint"));
        }
        
        Ok(Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()?,
            api_key,
            api_base: api_base.trim_end_matches('/').to_string(),
            model: "gpt-4".to_string(),
            temperature: 0.7,
            max_tokens: 2000,
        })
    }
    
    /// Set model
    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }
    
    /// Set temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = max_tokens;
        self
    }
}

#[async_trait]
impl Provider for OpenAIProvider {
    async fn generate(&self, prompt: &str) -> Result<String> {
        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: self.get_system_prompt(),
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            temperature: self.temperature,
            max_tokens: self.max_tokens,
        };
        
        let url = format!("{}/chat/completions", self.api_base);
        let mut req = self.client.post(&url).json(&request);
        
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }
        
        let response = req
            .send()
            .await
            .map_err(|e| anyhow!("API request failed: {}", e))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("API error {}: {}", status, error_text));
        }
        
        let response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse API response: {}", e))?;
        
        if let Some(choice) = response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(anyhow!("No response choices from API"))
        }
    }
    
    async fn analyze_code(&self, code: &str, language: &str) -> Result<CodeAnalysis> {
        let prompt = format!(
            r#"Analyze this {language} code and return ONLY valid JSON (no markdown, no explanation):

{{
  "quality_score": <0-100>,
  "issues": ["issue1", "issue2"],
  "performance_issues": ["perf1"],
  "security_issues": ["sec1"],
  "suggestions": ["suggestion1"]
}}

Code:
```{language}
{code}
```"#
        );
        
        let response = self.generate(&prompt).await?;

        let cleaned = response
            .trim()
            .strip_prefix("```json").unwrap_or(response.trim())
            .strip_prefix("```").unwrap_or(response.trim())
            .strip_suffix("```").unwrap_or(response.trim())
            .trim()
            .to_string();

        if let Ok(parsed) = serde_json::from_str::<CodeAnalysis>(&cleaned) {
            return Ok(CodeAnalysis {
                raw_response: response,
                ..parsed
            });
        }

        Ok(CodeAnalysis {
            quality_score: 50.0,
            issues: vec!["Could not parse LLM response — analysis incomplete".to_string()],
            performance_issues: vec![],
            security_issues: vec![],
            suggestions: vec![],
            raw_response: response,
        })
    }
    
    async fn generate_tests(&self, code: &str, language: &str) -> Result<String> {
        let prompt = format!(
            "Generate comprehensive tests for this {language} code:\n\n\
             ```{language}\n{code}\n```\n\n\
             Include:\n\
             1. Unit tests\n\
             2. Edge cases\n\
             3. Integration tests\n\
             4. Test fixtures\n\
             5. Assertions"
        );
        
        self.generate(&prompt).await
    }
    
    async fn generate_documentation(&self, code: &str, language: &str) -> Result<String> {
        let prompt = format!(
            "Generate comprehensive documentation for this {language} code:\n\n\
             ```{language}\n{code}\n```\n\n\
             Include:\n\
             1. Module overview\n\
             2. Function descriptions\n\
             3. Parameter details\n\
             4. Return values\n\
             5. Usage examples\n\
             6. Edge cases and notes"
        );
        
        self.generate(&prompt).await
    }

    async fn embed(&self, input: &str) -> Result<Vec<f32>> {
        let url = format!("{}/embeddings", self.api_base);
        let request = EmbeddingRequest {
            model: "nomic-embed-text".to_string(),
            input: input.to_string(),
        };

        let mut req = self.client.post(&url).json(&request);
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let response = req.send().await
            .map_err(|e| anyhow!("Embedding request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Embedding error {}: {}", status, text));
        }

        let result: EmbeddingResponse = response.json().await
            .map_err(|e| anyhow!("Failed to parse embedding response: {}", e))?;

        result.data.into_iter().next()
            .map(|d| d.embedding)
            .ok_or_else(|| anyhow!("No embedding returned"))
    }
}

impl OpenAIProvider {
    fn get_system_prompt(&self) -> String {
        r#"You are an expert AI assistant for a polyglot workspace.
You help developers write better code, optimize builds, and improve software quality.

Key principles:
1. Provide actionable, specific suggestions
2. Consider polyglot nature (Node.js, Python, Rust, Go, etc.)
3. Focus on code quality, performance, and security
4. Be practical and constructive
5. Explain reasoning behind suggestions"#.to_string()
    }
}

// Embedding support (OpenAI-compatible — works with Ollama's /api/embeddings)
#[derive(Serialize)]
struct EmbeddingRequest {
    model: String,
    input: String,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}
