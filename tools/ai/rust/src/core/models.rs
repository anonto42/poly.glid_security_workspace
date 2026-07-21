use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub id: String,
    pub category: String,
    pub priority: u8,
    pub title: String,
    pub description: String,
    pub impact: String,
    pub action: Option<String>,
    pub confidence: f32,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutdatedInfo {
    pub current: String,
    pub latest: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAnalysis {
    pub outdated: HashMap<String, OutdatedInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityAnalysis {
    pub average_score: f32,
    pub files_analyzed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAnalysis {
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityVulnerability {
    pub title: String,
    pub description: String,
    pub severity: u8,
    pub fix_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityReport {
    pub vulnerabilities: Vec<SecurityVulnerability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureAnalysis {
    pub build_time: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analysis {
    pub timestamp: DateTime<Utc>,
    pub structure: StructureAnalysis,
    pub dependencies: DependencyAnalysis,
    pub code_quality: CodeQualityAnalysis,
    pub security: SecurityReport,
    pub recommendations: Vec<Suggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeReview {
    pub file: PathBuf,
    pub language: String,
    pub issues: Vec<String>,
    pub suggestions: Vec<String>,
    pub quality_score: f32,
    pub security_issues: Vec<String>,
    pub performance_issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildOptimization {
    pub _suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildHistory;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestGeneration {
    pub raw_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Documentation {
    pub raw_text: String,
}
