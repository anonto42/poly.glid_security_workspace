//! Main AI Engine for PolyGlid Workspace

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

#[derive(Debug, Clone, Deserialize)]
struct PerDomainFile {
    model: String,
    temperature: Option<f32>,
    max_tokens: Option<usize>,
}

use crate::core::context::WorkspaceContext;
use crate::core::models::{
    Suggestion, Analysis, CodeReview, SecurityReport,
    BuildOptimization, TestGeneration, Documentation, CodeQualityAnalysis
};
use crate::providers::{Provider, ProviderFactory};
use crate::features::{
    CodeAnalyzer, DependencyAdvisor, BuildOptimizer,
    TestGenerator, SecurityAnalyzer, IngestService
};
use crate::cache::CacheManager;
use crate::tools::ToolExecutor;
use crate::feedback::FeedbackTracker;

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
    pub ingest_service: Arc<IngestService>,
    pub tool_executor: ToolExecutor,
    pub feedback_tracker: FeedbackTracker,
    
    /// Configuration
    pub config: EngineConfig,

    /// Workspace root path
    pub workspace_path: PathBuf,
}

/// Engine configuration
#[derive(Debug, Clone, Deserialize)]
pub struct EngineConfig {
    pub provider_type: ProviderType,
    pub api_base: Option<String>,
    pub api_key: Option<String>,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: usize,
    pub cache_enabled: bool,
    pub auto_suggestions: bool,
    pub suggestion_interval: u64, // seconds
    pub models: Option<DomainModels>,
}

/// Per-domain model overrides
#[derive(Debug, Clone, Deserialize)]
pub struct DomainModels {
    pub code: Option<String>,
    pub security: Option<String>,
    pub build: Option<String>,
    pub suggest: Option<String>,
}

/// Provider types
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum ProviderType {
    OpenAI,
    Ollama,
    Anthropic,
    Local,
    Hybrid,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            provider_type: ProviderType::Ollama,
            api_base: Some("http://localhost:11434/v1".to_string()),
            api_key: None,
            model: "codellama:7b".to_string(),
            temperature: 0.7,
            max_tokens: 4096,
            cache_enabled: true,
            auto_suggestions: true,
            suggestion_interval: 3600,
            models: None,
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
        let ingest_service = Arc::new(IngestService::new(
            provider.clone(), cache.clone()
        ));
        let tool_executor = ToolExecutor::new(workspace_path);
        let feedback_tracker = FeedbackTracker::new(workspace_path);

        let engine = Self {
            context,
            provider,
            cache,
            code_analyzer,
            dependency_advisor,
            build_optimizer,
            test_generator,
            security_analyzer,
            ingest_service,
            tool_executor,
            feedback_tracker,
            config,
            workspace_path: workspace_path.to_path_buf(),
        };
        
        let duration = start.elapsed();
        info!("✅ AI Engine initialized in {:?}", duration);
        
        Ok(engine)
    }
    
    /// Load configuration from file
    async fn load_config(workspace_path: &Path) -> Result<EngineConfig> {
        let config_dir = workspace_path
            .join(".workspace")
            .join("ai")
            .join("configs");
        let config_path = config_dir.join("ai-config.toml");
        
        let mut config = if config_path.exists() {
            let content = tokio::fs::read_to_string(&config_path).await?;
            let config: EngineConfig = toml::from_str(&content)?;
            config
        } else {
            EngineConfig::default()
        };

        // Load per-domain model configs from model-configs/ directory
        let model_configs_dir = config_dir.join("model-configs");
        if model_configs_dir.exists() {
            let domains = vec![
                ("code", "code-model.toml"),
                ("security", "security-model.toml"),
                ("build", "build-model.toml"),
                ("suggest", "suggest-model.toml"),
            ];
            let mut code = config.models.as_ref().and_then(|m| m.code.clone());
            let mut security = config.models.as_ref().and_then(|m| m.security.clone());
            let mut build = config.models.as_ref().and_then(|m| m.build.clone());
            let mut suggest = config.models.as_ref().and_then(|m| m.suggest.clone());

            for (domain, filename) in &domains {
                let path = model_configs_dir.join(filename);
                if path.exists() {
                    if let Ok(content) = tokio::fs::read_to_string(&path).await {
                        if let Ok(domain_cfg) = toml::from_str::<PerDomainFile>(&content) {
                            let model_name = Some(domain_cfg.model);
                            match *domain {
                                "code" => code = code.or(model_name),
                                "security" => security = security.or(model_name),
                                "build" => build = build.or(model_name),
                                "suggest" => suggest = suggest.or(model_name),
                                _ => {}
                            }
                        }
                    }
                }
            }

            config.models = Some(DomainModels { code, security, build, suggest });
        }

        Ok(config)
    }
    
    /// Analyze the entire workspace
    pub async fn analyze_workspace(&self) -> Result<Analysis> {
        info!("🔍 Analyzing workspace...");
        
        let context = self.context.read().await;
        
        // Analyze structure
        let structure = context.analyze_structure().await?;
        
        // Analyze dependencies
        let dependencies = self.dependency_advisor.analyze_all().await?;
        
        // Analyze code quality & security across real files
        let source_files = self.discover_source_files(&context).await?;
        let file_count = source_files.len();

        let code_quality = if file_count > 0 {
            let mut total_score = 0.0;
            for path in &source_files[..file_count.min(20)] {
                if let Ok(result) = self.code_analyzer.analyze_file(path).await {
                    total_score += result.score;
                }
            }
            CodeQualityAnalysis {
                average_score: total_score / file_count.min(20) as f32,
                files_analyzed: file_count,
            }
        } else {
            CodeQualityAnalysis { average_score: 0.0, files_analyzed: 0 }
        };

        let security = if file_count > 0 {
            let mut all_vulns = Vec::new();
            for path in &source_files[..file_count.min(10)] {
                if let Ok(report) = self.security_analyzer.analyze_file(path).await {
                    all_vulns.extend(report.vulnerabilities);
                }
            }
            SecurityReport { vulnerabilities: all_vulns }
        } else {
            SecurityReport { vulnerabilities: vec![] }
        };
        
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

    /// Detect changed files between git refs using detect-changes.sh
    pub async fn detect_changes(&self, base: Option<&str>) -> Result<Vec<std::path::PathBuf>> {
        let script = self.workspace_path
            .join(".workspace/automation/scripts/detect-changes.sh");
        if !script.exists() {
            return Ok(Vec::new());
        }

        let base_ref = base.unwrap_or("main");
        let output = tokio::process::Command::new("bash")
            .arg(&script)
            .arg(base_ref)
            .arg("HEAD")
            .current_dir(&self.workspace_path)
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut files = Vec::new();
        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('🔍') || trimmed.starts_with('📦') {
                continue;
            }
            for project in trimmed.split_whitespace() {
                let path = self.workspace_path.join("projects");
                // Search all project dirs for matching name
                let mut found = self.find_project_dir(&path, project).await;
                if let Some(p) = found {
                    files.push(p);
                }
            }
        }
        Ok(files)
    }

    async fn find_project_dir(&self, dir: &std::path::Path, name: &str) -> Option<std::path::PathBuf> {
        let mut stack = vec![dir.to_path_buf()];
        while let Some(current) = stack.pop() {
            if let Ok(mut rd) = tokio::fs::read_dir(&current).await {
                while let Ok(Some(entry)) = rd.next_entry().await {
                    let path = entry.path();
                    if path.is_dir() {
                        if path.file_name().and_then(|n| n.to_str()) == Some(name) {
                            return Some(path);
                        }
                        stack.push(path);
                    }
                }
            }
        }
        None
    }
    
    /// Generate code based on description
    pub async fn generate_code(
        &self,
        description: &str,
        language: &str,
    ) -> Result<String> {
        info!("🤖 Generating {} code: {}", language, description);
        
        let context = self.workspace_context_prompt().await;
        let context = self.rag_augment(description, &context).await;
        
        let prompt = format!(
            "{context}\n\n\
            Generate {language} code for: {description}\n\n\
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
        
        let context = self.workspace_context_prompt().await;
        let enriched = format!("{}\n\nGenerate tests for this {} file:\n```{}\n{}\n```", context, language, language, content);
        
        let tests = self.test_generator.generate(&enriched, &language).await?;
        Ok(tests)
    }
    
    /// Review code and provide feedback
    pub async fn review_code(&self, file_path: &Path) -> Result<CodeReview> {
        info!("📝 Reviewing code: {:?}", file_path);
        
        let content = tokio::fs::read_to_string(file_path).await?;
        let language = Self::detect_language(file_path)?;
        
        let context = self.workspace_context_prompt().await;
        let context = self.rag_augment("code review patterns", &context).await;
        let enriched = format!("{}\n\nReview this {} code:\n```{}\n{}\n```", context, language, language, content);
        
        let analysis = self.provider.analyze_code(&enriched, &language).await?;
        
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

    /// Discover source files in the workspace
    async fn discover_source_files(&self, context: &WorkspaceContext) -> Result<Vec<PathBuf>> {
        let ws = context.workspace_path();
        let mut files = Vec::new();
        let mut stack = vec![ws.join("projects")];

        while let Some(dir) = stack.pop() {
            if !dir.is_dir() { continue; }
            let mut read_dir = tokio::fs::read_dir(&dir).await?;
            while let Some(entry) = read_dir.next_entry().await? {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if !name.starts_with('.') && name != "node_modules" && name != "target" {
                        stack.push(path);
                    }
                } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    match ext {
                        "rs" | "py" | "js" | "ts" | "go" | "java" | "c" | "h" | "cpp" | "hpp" => {
                            files.push(path);
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(files)
    }

    /// Build a context string describing the workspace
    pub async fn workspace_context_prompt(&self) -> String {
        let ctx = self.context.read().await;
        let ws = ctx.workspace_path().to_string_lossy().to_string();

        let mut lines = vec![
            format!("You are analyzing the PolyGlid workspace at: {}", ws),
        ];

        if let Ok(files) = self.discover_source_files(&ctx).await {
            use std::collections::HashMap;
            let mut by_lang: HashMap<String, usize> = HashMap::new();
            for p in &files {
                if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                    *by_lang.entry(ext.to_string()).or_insert(0) += 1;
                }
            }
            let mut parts: Vec<String> = by_lang.iter()
                .map(|(ext, count)| format!("*.{}: {} files", ext, count))
                .collect();
            parts.sort();
            lines.push(format!("Source files: {}", parts.join(", ")));
            lines.push(format!("Total files: {}", files.len()));
        }

        if let Ok(deps) = self.dependency_advisor.analyze_all().await {
            if !deps.outdated.is_empty() {
                let names: Vec<String> = deps.outdated.keys().cloned().collect();
                lines.push(format!("Outdated deps: {}", names.join(", ")));
            }
        }

        lines.join("\n")
    }

    /// Augment a prompt with RAG context from the code index
    pub async fn rag_augment(&self, query: &str, context: &str) -> String {
        let ws = {
            let ctx = self.context.read().await;
            ctx.workspace_path().to_path_buf()
        };

        let chunks = self.ingest_service.search(query, &ws, 5).await.unwrap_or_default();
        if chunks.is_empty() {
            return context.to_string();
        }

        let mut parts = vec![context.to_string()];
        parts.push("\n--- Relevant code from workspace ---".to_string());
        for c in &chunks {
            let snippet: String = c.chunk.content.lines().take(10).collect::<Vec<_>>().join("\n");
            parts.push(format!("\n{}:{}\n{}", c.chunk.file, c.chunk.start_line, snippet));
        }

        parts.join("\n")
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
