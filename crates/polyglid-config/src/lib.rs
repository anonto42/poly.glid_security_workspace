//! Configuration defaults and validation for the host.

use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use polyglid_plugin_api::{Capability, CapabilityRequest, CapabilityScope, PluginId};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub plugin_dir: PathBuf,
    pub reports_dir: PathBuf,
    pub max_wasm_fuel: u64,
    pub default_capabilities: Vec<Capability>,
    pub approved_capabilities: Vec<CapabilityGrant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityGrant {
    pub plugin_id: Option<PluginId>,
    pub request: CapabilityRequest,
}

impl AppConfig {
    pub fn development() -> Self {
        Self {
            plugin_dir: PathBuf::from("plugins"),
            reports_dir: PathBuf::from("reports"),
            max_wasm_fuel: 25_000_000,
            default_capabilities: Vec::new(),
            approved_capabilities: Vec::new(),
        }
    }

    pub fn load_from_env() -> Result<Self, ConfigError> {
        match std::env::var_os("POLYGLID_CONFIG") {
            Some(path) => Self::load_from_path(PathBuf::from(path)),
            None => Ok(Self::development()),
        }
    }

    pub fn load_from_path(path: PathBuf) -> Result<Self, ConfigError> {
        let raw = fs::read_to_string(&path)
            .map_err(|err| ConfigError::ReadFailed(path, err.to_string()))?;
        Self::from_toml_str(&raw)
    }

    pub fn from_toml_str(raw: &str) -> Result<Self, ConfigError> {
        let raw_config: RawAppConfig =
            toml::from_str(raw).map_err(|err| ConfigError::ParseFailed(err.to_string()))?;
        let mut config = Self {
            plugin_dir: raw_config
                .plugin_dir
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("plugins")),
            reports_dir: raw_config
                .reports_dir
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("reports")),
            max_wasm_fuel: raw_config.max_wasm_fuel.unwrap_or(25_000_000),
            default_capabilities: raw_config
                .default_capabilities
                .into_iter()
                .map(|capability| Capability::from_str(&capability))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|err| ConfigError::InvalidCapability(err.to_string()))?,
            approved_capabilities: Vec::new(),
        };

        for grant in raw_config.approved_capabilities {
            config
                .approved_capabilities
                .push(grant.into_capability_grant()?);
        }

        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.plugin_dir.as_os_str().is_empty() {
            return Err(ConfigError::EmptyPluginDir);
        }
        if self.reports_dir.as_os_str().is_empty() {
            return Err(ConfigError::EmptyReportsDir);
        }
        if self.max_wasm_fuel == 0 {
            return Err(ConfigError::EmptyFuelLimit);
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct RawAppConfig {
    #[serde(default)]
    plugin_dir: Option<String>,
    #[serde(default)]
    reports_dir: Option<String>,
    #[serde(default)]
    max_wasm_fuel: Option<u64>,
    #[serde(default)]
    default_capabilities: Vec<String>,
    #[serde(default)]
    approved_capabilities: Vec<RawCapabilityGrant>,
}

#[derive(Debug, Deserialize)]
struct RawCapabilityGrant {
    #[serde(default)]
    plugin_id: Option<String>,
    capability: String,
    #[serde(default)]
    target: Option<String>,
    #[serde(default)]
    path_prefix: Option<String>,
    #[serde(default)]
    host: Option<String>,
    #[serde(default)]
    port: Option<u16>,
}

impl RawCapabilityGrant {
    fn into_capability_grant(self) -> Result<CapabilityGrant, ConfigError> {
        let capability = Capability::from_str(&self.capability)
            .map_err(|err| ConfigError::InvalidCapability(err.to_string()))?;
        let scope = match (self.target, self.path_prefix, self.host, self.port) {
            (Some(target), None, None, None) => CapabilityScope::Target(target),
            (None, Some(path_prefix), None, None) => CapabilityScope::PathPrefix(path_prefix),
            (None, None, Some(host), Some(port)) => CapabilityScope::HostPort { host, port },
            (None, None, None, None) => CapabilityScope::Any,
            _ => return Err(ConfigError::InvalidCapabilityScope),
        };
        let plugin_id = self
            .plugin_id
            .map(PluginId::new)
            .transpose()
            .map_err(|err| ConfigError::InvalidPluginId(err.to_string()))?;

        Ok(CapabilityGrant {
            plugin_id,
            request: CapabilityRequest::new(capability, scope),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    EmptyPluginDir,
    EmptyReportsDir,
    EmptyFuelLimit,
    InvalidCapability(String),
    InvalidCapabilityScope,
    InvalidPluginId(String),
    ParseFailed(String),
    ReadFailed(PathBuf, String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyPluginDir => f.write_str("plugin directory cannot be empty"),
            Self::EmptyReportsDir => f.write_str("reports directory cannot be empty"),
            Self::EmptyFuelLimit => f.write_str("max wasm fuel must be greater than zero"),
            Self::InvalidCapability(message) => write!(f, "invalid capability: {message}"),
            Self::InvalidCapabilityScope => {
                f.write_str("capability grant must use one scope shape")
            }
            Self::InvalidPluginId(message) => write!(f, "invalid plugin id: {message}"),
            Self::ParseFailed(message) => write!(f, "failed to parse config: {message}"),
            Self::ReadFailed(path, message) => {
                write!(f, "failed to read config {}: {message}", path.display())
            }
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_persistent_capability_approvals() {
        let config = AppConfig::from_toml_str(
            r#"
plugin_dir = "plugins"
reports_dir = "reports"
max_wasm_fuel = 1000000
default_capabilities = ["dns-resolve"]

[[approved_capabilities]]
plugin_id = "polyglid.recon_probe"
capability = "network-connect"
host = "example.com"
port = 443

[[approved_capabilities]]
capability = "filesystem-read"
path_prefix = "/tmp/polyglid"
"#,
        )
        .expect("config parses");

        assert_eq!(config.default_capabilities, vec![Capability::DnsResolve]);
        assert_eq!(config.max_wasm_fuel, 1_000_000);
        assert_eq!(config.approved_capabilities.len(), 2);
        assert_eq!(
            config.approved_capabilities[0].request,
            CapabilityRequest::new(
                Capability::NetworkConnect,
                CapabilityScope::HostPort {
                    host: "example.com".to_string(),
                    port: 443,
                },
            )
        );
        assert_eq!(
            config.approved_capabilities[0]
                .plugin_id
                .as_ref()
                .expect("plugin id")
                .as_str(),
            "polyglid.recon_probe"
        );
    }

    #[test]
    fn rejects_ambiguous_capability_scope() {
        let err = AppConfig::from_toml_str(
            r#"
[[approved_capabilities]]
capability = "network-connect"
target = "example.com"
host = "example.com"
port = 443
"#,
        )
        .expect_err("scope rejected");

        assert_eq!(err, ConfigError::InvalidCapabilityScope);
    }

    #[test]
    fn rejects_zero_fuel_limit() {
        let err = AppConfig::from_toml_str("max_wasm_fuel = 0").expect_err("fuel rejected");

        assert_eq!(err, ConfigError::EmptyFuelLimit);
    }
}
