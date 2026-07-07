//! Shared host/plugin types mirrored from `wit/polyglid.wit`.

use std::fmt;
use std::str::FromStr;

/// Widget types that correspond to the `widget-type` enum in the WIT contract.
/// The host knows how to render each of these natively in both TUI and desktop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WidgetKind {
    Table,
    KeyValue,
    Tree,
    Log,
    ChartBar,
    TextBlock,
}

impl fmt::Display for WidgetKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Table => "table",
            Self::KeyValue => "key-value",
            Self::Tree => "tree",
            Self::Log => "log",
            Self::ChartBar => "chart-bar",
            Self::TextBlock => "text-block",
        };
        f.write_str(label)
    }
}

/// A single renderable widget inside a panel layout.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PanelWidget {
    pub widget_kind: WidgetKind,
    pub title: String,
    /// Rows of cells. Interpretation depends on `widget_kind`:
    /// - Table: first row is headers, rest are data rows
    /// - KeyValue: each row is [key, value]
    /// - Tree: each row is [indent-level, label]
    /// - Log: each row is [timestamp, message]
    /// - ChartBar: each row is [label, numeric-value]
    /// - TextBlock: each row is [line-of-text]
    pub data: Vec<Vec<String>>,
}

/// A structured panel layout returned by a plugin for host-native rendering.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PanelLayout {
    pub title: String,
    pub widgets: Vec<PanelWidget>,
}

/// Metadata embedded inside a plugin's WASM component.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ApiPluginMetadata {
    pub name: String,
    pub display_name: String,
    pub version: String,
    pub description: String,
    pub author: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Capability {
    NetworkConnect,
    NetworkListen,
    FilesystemRead,
    FilesystemWrite,
    ConfigRead,
    ReportWrite,
    Crypto,
    DnsResolve,
    ProcessSpawn,
    EnvironmentRead,
}

impl Capability {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NetworkConnect => "network-connect",
            Self::NetworkListen => "network-listen",
            Self::FilesystemRead => "filesystem-read",
            Self::FilesystemWrite => "filesystem-write",
            Self::ConfigRead => "config-read",
            Self::ReportWrite => "report-write",
            Self::Crypto => "crypto",
            Self::DnsResolve => "dns-resolve",
            Self::ProcessSpawn => "process-spawn",
            Self::EnvironmentRead => "environment-read",
        }
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Capability {
    type Err = ApiError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "network-connect" | "NetworkConnect" => Ok(Self::NetworkConnect),
            "network-listen" | "NetworkListen" => Ok(Self::NetworkListen),
            "filesystem-read" | "FilesystemRead" | "file-read" | "FileRead" => {
                Ok(Self::FilesystemRead)
            }
            "filesystem-write" | "FilesystemWrite" | "file-write" | "FileWrite" => {
                Ok(Self::FilesystemWrite)
            }
            "config-read" | "ConfigRead" => Ok(Self::ConfigRead),
            "report-write" | "ReportWrite" => Ok(Self::ReportWrite),
            "crypto" | "Crypto" => Ok(Self::Crypto),
            "dns-resolve" | "DnsResolve" => Ok(Self::DnsResolve),
            "process-spawn" | "ProcessSpawn" => Ok(Self::ProcessSpawn),
            "environment-read" | "EnvironmentRead" | "env-read" => Ok(Self::EnvironmentRead),
            _ => Err(ApiError::UnknownCapability(value.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PluginManifest {
    pub id: PluginId,
    pub name: String,
    pub version: String,
    pub requested_capabilities: Vec<CapabilityRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CapabilityRequest {
    pub capability: Capability,
    pub scope: CapabilityScope,
}

impl CapabilityRequest {
    pub fn new(capability: Capability, scope: CapabilityScope) -> Self {
        Self { capability, scope }
    }

    pub fn unscoped(capability: Capability) -> Self {
        Self {
            capability,
            scope: CapabilityScope::Any,
        }
    }
}

impl fmt::Display for CapabilityRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.scope {
            CapabilityScope::Any => write!(f, "{}", self.capability),
            scope => write!(f, "{} ({scope})", self.capability),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CapabilityScope {
    Any,
    Target(String),
    PathPrefix(String),
    HostPort { host: String, port: u16 },
}

impl fmt::Display for CapabilityScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Any => f.write_str("any"),
            Self::Target(target) => write!(f, "target={target}"),
            Self::PathPrefix(path) => write!(f, "path-prefix={path}"),
            Self::HostPort { host, port } => write!(f, "host={host},port={port}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    EmptyPluginId,
    UnknownCapability(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPluginId => f.write_str("plugin id cannot be empty"),
            Self::UnknownCapability(value) => write!(f, "unknown capability: {value}"),
        }
    }
}

impl std::error::Error for ApiError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_documented_capabilities() {
        assert_eq!(
            "network-connect".parse::<Capability>().expect("capability"),
            Capability::NetworkConnect
        );
        assert_eq!(
            "DnsResolve".parse::<Capability>().expect("capability"),
            Capability::DnsResolve
        );
        assert_eq!(Capability::FilesystemRead.to_string(), "filesystem-read");
    }

    #[test]
    fn formats_scoped_capability_requests() {
        let request = CapabilityRequest::new(
            Capability::NetworkConnect,
            CapabilityScope::HostPort {
                host: "example.com".to_string(),
                port: 443,
            },
        );

        assert_eq!(
            request.to_string(),
            "network-connect (host=example.com,port=443)"
        );
    }
}
