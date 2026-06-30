//! Configuration defaults and validation for the host.

use std::path::PathBuf;

use polyglid_plugin_api::Capability;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub plugin_dir: PathBuf,
    pub reports_dir: PathBuf,
    pub default_capabilities: Vec<Capability>,
}

impl AppConfig {
    pub fn development() -> Self {
        Self {
            plugin_dir: PathBuf::from("plugins"),
            reports_dir: PathBuf::from("reports"),
            default_capabilities: Vec::new(),
        }
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.plugin_dir.as_os_str().is_empty() {
            return Err(ConfigError::EmptyPluginDir);
        }
        if self.reports_dir.as_os_str().is_empty() {
            return Err(ConfigError::EmptyReportsDir);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    EmptyPluginDir,
    EmptyReportsDir,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyPluginDir => f.write_str("plugin directory cannot be empty"),
            Self::EmptyReportsDir => f.write_str("reports directory cannot be empty"),
        }
    }
}

impl std::error::Error for ConfigError {}
