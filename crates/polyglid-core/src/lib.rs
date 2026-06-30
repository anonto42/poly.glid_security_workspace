//! Product policy and orchestration for PolyGlid.

use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};

use polyglid_config::AppConfig;
use polyglid_events::{EventSink, PolyGlidEvent};
use polyglid_plugin_api::{Capability, PluginId, PluginManifest, PluginReport};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginRef {
    path: PathBuf,
}

impl PluginRef {
    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginRunRequest {
    pub plugin: PluginRef,
    pub target: Target,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target(String);

impl Target {
    pub fn parse(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(CoreError::InvalidTarget(
                "target cannot be empty".to_string(),
            ));
        }
        if trimmed.len() > 253 {
            return Err(CoreError::InvalidTarget("target is too long".to_string()));
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub trait PluginRuntime {
    fn inspect(&self, plugin: &PluginRef) -> Result<PluginManifest, CoreError>;
    fn execute(&self, request: &PluginRunRequest) -> Result<PluginReport, CoreError>;
}

pub trait PermissionStore {
    fn is_allowed(&self, plugin_id: &PluginId, capability: Capability) -> bool;
}

#[derive(Debug, Default)]
pub struct InMemoryPermissionStore {
    grants: HashSet<(PluginId, Capability)>,
}

impl InMemoryPermissionStore {
    pub fn grant(&mut self, plugin_id: PluginId, capability: Capability) {
        self.grants.insert((plugin_id, capability));
    }
}

impl PermissionStore for InMemoryPermissionStore {
    fn is_allowed(&self, plugin_id: &PluginId, capability: Capability) -> bool {
        self.grants.contains(&(plugin_id.clone(), capability))
    }
}

pub struct CoreEngine<R, P, E> {
    runtime: R,
    permissions: P,
    events: E,
    config: AppConfig,
}

impl<R, P, E> CoreEngine<R, P, E>
where
    R: PluginRuntime,
    P: PermissionStore,
    E: EventSink,
{
    pub fn new(
        runtime: R,
        permissions: P,
        events: E,
        config: AppConfig,
    ) -> Result<Self, CoreError> {
        config
            .validate()
            .map_err(|err| CoreError::Config(err.to_string()))?;
        Ok(Self {
            runtime,
            permissions,
            events,
            config,
        })
    }

    pub fn inspect_plugin(&mut self, plugin: PluginRef) -> Result<PluginManifest, CoreError> {
        self.events.emit(PolyGlidEvent::PluginInspectStarted {
            path: plugin.path().display().to_string(),
        });
        self.runtime.inspect(&plugin)
    }

    pub fn run_plugin(&mut self, request: PluginRunRequest) -> Result<PluginReport, CoreError> {
        let manifest = self.runtime.inspect(&request.plugin)?;
        for capability in &manifest.requested_capabilities {
            if !self.permissions.is_allowed(&manifest.id, *capability) {
                self.events.emit(PolyGlidEvent::CapabilityDenied {
                    plugin_id: manifest.id.clone(),
                    capability: format!("{capability:?}"),
                });
                return Err(CoreError::CapabilityDenied {
                    plugin_id: manifest.id,
                    capability: *capability,
                });
            }
        }

        self.events.emit(PolyGlidEvent::PluginRunStarted {
            plugin_id: manifest.id.clone(),
            target: request.target.as_str().to_string(),
        });

        match self.runtime.execute(&request) {
            Ok(report) => {
                self.events.emit(PolyGlidEvent::PluginRunCompleted {
                    plugin_id: manifest.id,
                    report: report.clone(),
                });
                Ok(report)
            }
            Err(err) => {
                self.events.emit(PolyGlidEvent::PluginRunFailed {
                    plugin_id: manifest.id,
                    message: err.to_string(),
                });
                Err(err)
            }
        }
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    Config(String),
    InvalidTarget(String),
    PluginNotFound(PathBuf),
    Runtime(String),
    CapabilityDenied {
        plugin_id: PluginId,
        capability: Capability,
    },
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(message) => write!(f, "invalid config: {message}"),
            Self::InvalidTarget(message) => write!(f, "invalid target: {message}"),
            Self::PluginNotFound(path) => write!(f, "plugin file not found: {}", path.display()),
            Self::Runtime(message) => write!(f, "runtime error: {message}"),
            Self::CapabilityDenied {
                plugin_id,
                capability,
            } => write!(
                f,
                "plugin '{}' is missing required capability {capability:?}",
                plugin_id.as_str()
            ),
        }
    }
}

impl std::error::Error for CoreError {}

#[cfg(test)]
mod tests {
    use super::*;
    use polyglid_events::VecEventSink;
    use polyglid_plugin_api::{Issue, Severity};

    struct FakeRuntime;

    impl PluginRuntime for FakeRuntime {
        fn inspect(&self, _plugin: &PluginRef) -> Result<PluginManifest, CoreError> {
            Ok(PluginManifest {
                id: PluginId::new("demo").expect("valid id"),
                name: "Demo".to_string(),
                version: "0.1.0".to_string(),
                requested_capabilities: Vec::new(),
            })
        }

        fn execute(&self, request: &PluginRunRequest) -> Result<PluginReport, CoreError> {
            Ok(PluginReport {
                plugin_name: "Demo".to_string(),
                target_tested: request.target.as_str().to_string(),
                issues: vec![Issue::new(
                    "demo issue",
                    Severity::Info,
                    "test issue",
                    "no action",
                )],
                summary: "demo complete".to_string(),
            })
        }
    }

    #[test]
    fn target_rejects_empty_values() {
        assert!(Target::parse("  ").is_err());
    }

    #[test]
    fn engine_runs_plugin_when_no_capabilities_are_required() {
        let mut engine = CoreEngine::new(
            FakeRuntime,
            InMemoryPermissionStore::default(),
            VecEventSink::default(),
            AppConfig::development(),
        )
        .expect("valid engine");

        let report = engine
            .run_plugin(PluginRunRequest {
                plugin: PluginRef::from_path("demo.wasm"),
                target: Target::parse("example.com").expect("valid target"),
            })
            .expect("plugin runs");

        assert_eq!(report.target_tested, "example.com");
        assert!(report.has_findings());
    }
}
