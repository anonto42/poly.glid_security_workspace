//! Main AI Engine for PolyGlid Workspace

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

use crate::core::context::WorkspaceContext;
use crate::core::models::{
    Suggestion, Analysis, CodeReview, SecurityReport,
    BuildOptimization, TestGeneration, Documentation
};
use crate::providers::{Provider, ProviderFactory};
use crate::features::{
    CodeAnalyzer, DependencyAdvisor, BuildOptimizer,
    TestGenerator, SecurityAnalyzer
};
use crate::cache::CacheManager;

/// Main AI Engine
pub struct AIEngine {
    /// Workspace context
    pub context: Arc<RwLock<WorkspaceContext>>,
    
    /// AI Provider (OpenAI, Local, etc.)
    pub provider: Arc<dyn Provider + Send + Sync>,
    
    /// Cache manager
    pub cache: Arc<CacheManager>,
    
    /// Feature modules
    pub code_analyzer: Arc<CodeAnalyzer>,
    pub dependency_advisor: Arc<DependencyAdvisor>,
    pub build_optimizer: Arc<BuildOptimizer>,
    pub test_generator: Arc<TestGenerator>,
    pub security_analyzer: Arc<SecurityAnalyzer>,
    
    /// Configuration
    pub config: EngineConfig,
}

/// Engine configuration
#[derive(Debug, Clone, Deserialize)]
pub struct EngineConfig {
    pub provider_type: ProviderType,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: usize,
    pub cache_enabled: bool,
    pub auto_suggestions: bool,
    pub suggestion_interval: u64, // seconds
}

/// Provider types
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum ProviderType {
    OpenAI,
    Anthropic,
    Local,
    Hybrid,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            provider_type: ProviderType::OpenAI,
            model: "gpt-4".to_string(),
            temperature: 0.7,
            max_tokens: 2000,
            cache_enabled: true,
            auto_suggestions: true,
            suggestion_interval: 3600,
        }
    }
}

impl AIEngine {
    /// Create new AI Engine
    pub async fn new(workspace_path: &Path) -> Result<Self> {
        info!("🚀 Initializing AI Engine...");
        let start = Instant::now();
        
        // Load configuration
        let config = Self::load_config(workspace_path).await?;
        
        // Create workspace context
        let context = Arc::new(RwLock::new(
            WorkspaceContext::new(workspace_path).await?
        ));
        
        // Create provider
        let provider = ProviderFactory::create(&config).await?;
        
        // Create cache manager
        let cache = Arc::new(CacheManager::new(workspace_path)?);
        
        // Create feature modules
        let code_analyzer = Arc::new(CodeAnalyzer::new(
            provider.clone(), cache.clone()
        ));
        let dependency_advisor = Arc::new(DependencyAdvisor::new(
            provider.clone(), cache.clone()
        ));
        let build_optimizer = Arc::new(BuildOptimizer::new(
            provider.clone(), cache.clone()
        ));
        let test_generator = Arc::new(TestGenerator::new(
            provider.clone(), cache.clone()
        ));
        let security_analyzer = Arc::new(SecurityAnalyzer::new(
            provider.clone(), cache.clone()
        ));
        
        let engine = Self {
            context,
            provider,
            cache,
            code_analyzer,
            dependency_advisor,
            build_optimizer,
            test_generator,
            security_analyzer,
            config,
        };
        
        let duration = start.elapsed();
        info!("✅ AI Engine initialized in {:?}", duration);
        
        Ok(engine)
    }
    
    /// Load configuration from file
    async fn load_config(workspace_path: &Path) -> Result<EngineConfig> {
        let config_path = workspace_path
            .join(".workspace")
            .join("ai")
            .join("configs")
            .join("ai-config.toml");
        
        if config_path.exists() {
            let content = tokio::fs::read_to_string(&config_path).await?;
            let config: EngineConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(EngineConfig::default())
        }
    }
    
    /// Analyze the entire workspace
    pub async fn analyze_workspace(&self) -> Result<Analysis> {
        info!("🔍 Analyzing workspace...");
        
        let context = self.context.read().await;
        
        // Analyze structure
        let structure = context.analyze_structure().await?;
        
        // Analyze dependencies
        let dependencies = self.dependency_advisor.analyze_all().await?;
        
        // Analyze code quality
        let code_quality = self.code_analyzer.analyze_all().await?;
        
        // Analyze security
        let security = self.security_analyzer.analyze_workspace().await?;
        
        // Generate recommendations
        let recommendations = self.generate_recommendations(
            &structure,
            &dependencies,
            &code_quality,
            &security
        ).await?;
        
        let analysis = Analysis {
            timestamp: chrono::Utc::now(),
            structure,
            dependencies,
            code_quality,
            security,
            recommendations,
        };
        
        // Save analysis
        self.save_analysis(&analysis).await?;
        
        Ok(analysis)
    }
    
    /// Generate code based on description
    pub async fn generate_code(
        &self,
        description: &str,
        language: &str,
    ) -> Result<String> {
        info!("🤖 Generating {} code: {}", language, description);
        
        let prompt = format!(
            "Generate {language} code for: {description}\n\n\
            Include:\n\
            1. Complete implementation\n\
            2. Error handling\n\
            3. Comments and documentation\n\
            4. Best practices\n\
            5. Performance considerations"
        );
        
        let response = self.provider.generate(&prompt).await?;
        Ok(response)
    }
    
    /// Generate tests for a file
    pub async fn generate_tests(&self, file_path: &Path) -> Result<TestGeneration> {
        info!("🧪 Generating tests for: {:?}", file_path);
        
        let content = tokio::fs::read_to_string(file_path).await?;
        let language = Self::detect_language(file_path)?;
        
        let tests = self.test_generator.generate(&content, &language).await?;
        Ok(tests)
    }
    
    /// Review code and provide feedback
    pub async fn review_code(&self, file_path: &Path) -> Result<CodeReview> {
        info!("📝 Reviewing code: {:?}", file_path);
        
        let content = tokio::fs::read_to_string(file_path).await?;
        let language = Self::detect_language(file_path)?;
        
        // Get analysis from provider
        let analysis = self.provider.analyze_code(&content, &language).await?;
        
        // Enhance with local analysis
        let local_analysis = self.code_analyzer.analyze_file(file_path).await?;
        
        Ok(CodeReview {
            file: file_path.to_path_buf(),
            language,
            issues: analysis.issues,
            suggestions: analysis.suggestions,
            quality_score: local_analysis.score,
            security_issues: analysis.security_issues,
            performance_issues: analysis.performance_issues,
        })
    }
    
    /// Optimize build process
    pub async fn optimize_build(&self) -> Result<BuildOptimization> {
        info!("⚡ Optimizing build...");
        
        let context = self.context.read().await;
        let build_history = context.get_build_history().await?;
        
        let optimization = self.build_optimizer.optimize(&build_history).await?;
        
        // Apply optimizations
        self.build_optimizer.apply_optimizations(&optimization).await?;
        
        Ok(optimization)
    }
    
    /// Get AI suggestions
    pub async fn get_suggestions(&self, limit: usize) -> Result<Vec<Suggestion>> {
        info!("💡 Generating suggestions...");
        
        // Check cache first
        if self.config.cache_enabled {
            if let Some(cached) = self.cache.get_suggestions(limit).await? {
                return Ok(cached);
            }
        }
        
        // Analyze workspace
        let analysis = self.analyze_workspace().await?;
        
        // Generate suggestions
        let suggestions = self.generate_recommendations(
            &analysis.structure,
            &analysis.dependencies,
            &analysis.code_quality,
            &analysis.security,
        ).await?;
        
        // Cache suggestions
        if self.config.cache_enabled {
            self.cache.store_suggestions(&suggestions).await?;
        }
        
        Ok(suggestions.into_iter().take(limit).collect())
    }
    
    /// Generate recommendations based on analysis
    async fn generate_recommendations(
        &self,
        structure: &crate::core::models::StructureAnalysis,
        dependencies: &crate::core::models::DependencyAnalysis,
        code_quality: &crate::core::models::CodeQualityAnalysis,
        security: &SecurityReport,
    ) -> Result<Vec<Suggestion>> {
        let mut suggestions = Vec::new();
        
        // Code quality recommendations
        if code_quality.average_score < 70.0 {
            suggestions.push(Suggestion {
                id: uuid::Uuid::new_v4().to_string(),
                category: "code_quality".to_string(),
                priority: 7,
                title: "Improve code quality".to_string(),
                description: format!(
                    "Average code quality score is {:.1}/100. Consider refactoring.",
                    code_quality.average_score
                ),
                impact: "Better maintainability and fewer bugs".to_string(),
                action: Some("make refactor".to_string()),
                confidence: 0.85,
                metadata: std::collections::HashMap::new(),
            });
        }
        
        // Dependency recommendations
        for (name, info) in &dependencies.outdated {
            suggestions.push(Suggestion {
                id: uuid::Uuid::new_v4().to_string(),
                category: "dependency".to_string(),
                priority: 5,
                title: format!("Update dependency: {}", name),
                description: format!(
                    "New version {} available (current: {})",
                    info.latest, info.current
                ),
                impact: "Security fixes and performance improvements".to_string(),
                action: Some(format!("make update-{}", info.language)),
                confidence: 0.9,
                metadata: std::collections::HashMap::new(),
            });
        }
        
        // Security recommendations
        for vuln in &security.vulnerabilities {
            suggestions.push(Suggestion {
                id: uuid::Uuid::new_v4().to_string(),
                category: "security".to_string(),
                priority: vuln.severity,
                title: format!("Security issue: {}", vuln.title),
                description: vuln.description.clone(),
                impact: format!("Severity: {:?}", vuln.severity),
                action: vuln.fix_action.clone(),
                confidence: 0.95,
                metadata: std::collections::HashMap::new(),
            });
        }
        
        // Build performance recommendations
        if let Some(build_time) = structure.build_time {
            if build_time > 60.0 {
                suggestions.push(Suggestion {
                    id: uuid::Uuid::new_v4().to_string(),
                    category: "performance".to_string(),
                    priority: 6,
                    title: "Optimize build times".to_string(),
                    description: format!(
                        "Average build time is {:.1}s. Consider parallel builds.",
                        build_time
                    ),
                    impact: "Faster development cycles".to_string(),
                    action: Some("make optimize-build".to_string()),
                    confidence: 0.8,
                    metadata: std::collections::HashMap::new(),
                });
            }
        }
        
        // Sort by priority
        suggestions.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        Ok(suggestions)
    }
    
    /// Detect language from file extension
    fn detect_language(file_path: &Path) -> Result<String> {
        let ext = file_path.extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| anyhow!("No file extension"))?;
        
        let language = match ext {
            "rs" => "rust",
            "py" => "python",
            "js" => "javascript",
            "ts" => "typescript",
            "go" => "go",
            "java" => "java",
            "c" | "h" => "c",
            "cpp" | "hpp" => "cpp",
            _ => "unknown",
        };
        
        Ok(language.to_string())
    }
    
    /// Save analysis results
    async fn save_analysis(&self, analysis: &Analysis) -> Result<()> {
        let output_dir = self.context.read().await
            .workspace_path()
            .join(".workspace")
            .join("ai")
            .join("output")
            .join("reports");
        
        tokio::fs::create_dir_all(&output_dir).await?;
        
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = output_dir.join(format!("analysis_{}.json", timestamp));
        
        let json = serde_json::to_string_pretty(analysis)?;
        tokio::fs::write(filename, json).await?;
        
        Ok(())
    }
}
