//! Shared host/plugin types mirrored from `wit/polyglid.wit`.

use std::fmt;
use std::str::FromStr;

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
}
