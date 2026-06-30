//! Shared host/plugin types mirrored from `wit/polyglid.wit`.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Info => "info",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    pub title: String,
    pub severity: Severity,
    pub description: String,
    pub recommendation: String,
}

impl Issue {
    pub fn new(
        title: impl Into<String>,
        severity: Severity,
        description: impl Into<String>,
        recommendation: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            severity,
            description: description.into(),
            recommendation: recommendation.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginReport {
    pub plugin_name: String,
    pub target_tested: String,
    pub issues: Vec<Issue>,
    pub summary: String,
}

impl PluginReport {
    pub fn clean(plugin_name: impl Into<String>, target_tested: impl Into<String>) -> Self {
        let target_tested = target_tested.into();
        Self {
            plugin_name: plugin_name.into(),
            summary: format!("No issues were reported for {target_tested}."),
            target_tested,
            issues: Vec::new(),
        }
    }

    pub fn has_findings(&self) -> bool {
        !self.issues.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PluginId(String);

impl PluginId {
    pub fn new(value: impl Into<String>) -> Result<Self, ApiError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(ApiError::EmptyPluginId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Capability {
    NetworkConnect,
    FileRead,
    FileWrite,
    ProcessSpawn,
    EnvironmentRead,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginManifest {
    pub id: PluginId,
    pub name: String,
    pub version: String,
    pub requested_capabilities: Vec<Capability>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    EmptyPluginId,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPluginId => f.write_str("plugin id cannot be empty"),
        }
    }
}

impl std::error::Error for ApiError {}
