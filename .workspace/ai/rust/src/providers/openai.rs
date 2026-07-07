//! OpenAI Provider Implementation

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use reqwest::Client;
use std::env;

use super::traits::{Provider, CodeAnalysis};

/// OpenAI Provider
pub struct OpenAIProvider {
    client: Client,
    api_key: String,
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
    /// Create new OpenAI provider
    pub fn new(api_key: Option<String>) -> Result<Self> {
        let api_key = api_key
            .or_else(|| env::var("OPENAI_API_KEY").ok())
            .ok_or_else(|| anyhow!("OPENAI_API_KEY not found"))?;
        
        Ok(Self {
            client: Client::new(),
            api_key,
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
        
        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("OpenAI API error: {}", e))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("OpenAI error: {}", error_text));
        }
        
        let response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse response: {}", e))?;
        
        if let Some(choice) = response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(anyhow!("No response from OpenAI"))
        }
    }
    
    async fn analyze_code(&self, code: &str, language: &str) -> Result<CodeAnalysis> {
        let prompt = format!(
            "Analyze this {language} code and provide:\n\
             1. Code quality assessment (score 0-100)\n\
             2. List of issues (bugs, problems)\n\
             3. Performance issues\n\
             4. Security vulnerabilities\n\
             5. Suggestions for improvement\n\n\
             Code:\n```{language}\n{code}\n```"
        );
        
        let response = self.generate(&prompt).await?;
        
        // Parse response into structured format
        // In production, use structured output or parse with regex
        let analysis = CodeAnalysis {
            quality_score: 75.0, // Placeholder - would parse from response
            issues: vec!["Example issue".to_string()],
            performance_issues: vec!["Example performance issue".to_string()],
            security_issues: vec!["Example security issue".to_string()],
            suggestions: vec!["Example suggestion".to_string()],
            raw_response: response,
        };
        
        Ok(analysis)
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
