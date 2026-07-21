//! Product policy and orchestration for PolyGlid.

use std::fmt;
use std::path::{Path, PathBuf};

use polyglid_config::AppConfig;
use polyglid_events::{EventSink, PolyGlidEvent};
use polyglid_plugin_api::{
    Capability, CapabilityRequest, CapabilityScope, PluginId, PluginManifest, PluginReport,
};

#[cfg(test)]
pub mod benchmarks;
pub mod execution;
pub mod plugin_manager;
pub mod security;
pub mod services;
pub mod store;

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
    fn inspect_metadata(
        &self,
        plugin: &PluginRef,
    ) -> Result<polyglid_plugin_api::ApiPluginMetadata, CoreError>;
    fn execute(
        &self,
        request: &PluginRunRequest,
        config: &AppConfig,
    ) -> Result<PluginReport, CoreError>;
    fn cancel(&self, _job_id: uuid::Uuid) -> Result<(), CoreError> {
        Ok(())
    }
}

impl<R: PluginRuntime + ?Sized> PluginRuntime for std::sync::Arc<R> {
    fn inspect(&self, plugin: &PluginRef) -> Result<PluginManifest, CoreError> {
        (**self).inspect(plugin)
    }
    fn inspect_metadata(
        &self,
        plugin: &PluginRef,
    ) -> Result<polyglid_plugin_api::ApiPluginMetadata, CoreError> {
        (**self).inspect_metadata(plugin)
    }
    fn execute(
        &self,
        request: &PluginRunRequest,
        config: &AppConfig,
    ) -> Result<PluginReport, CoreError> {
        (**self).execute(request, config)
    }
    fn cancel(&self, job_id: uuid::Uuid) -> Result<(), CoreError> {
        (**self).cancel(job_id)
    }
}

pub trait PermissionStore {
    fn decide(
        &self,
        plugin_id: &PluginId,
        request: &CapabilityRequest,
    ) -> Result<PermissionDecision, CoreError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionDecision {
    Allow,
    Deny { reason: String },
}

#[derive(Debug, Default)]
pub struct InMemoryPermissionStore {
    plugin_grants: Vec<(PluginId, CapabilityRequest)>,
    global_grants: Vec<CapabilityRequest>,
}

impl InMemoryPermissionStore {
    pub fn grant(&mut self, plugin_id: PluginId, capability: Capability) {
        self.grant_request(plugin_id, CapabilityRequest::unscoped(capability));
    }

    pub fn grant_request(&mut self, plugin_id: PluginId, request: CapabilityRequest) {
        self.plugin_grants.push((plugin_id, request));
    }

    pub fn grant_for_all(&mut self, capability: Capability) {
        self.grant_request_for_all(CapabilityRequest::unscoped(capability));
    }

    pub fn grant_request_for_all(&mut self, request: CapabilityRequest) {
        self.global_grants.push(request);
    }
}

impl PermissionStore for InMemoryPermissionStore {
    fn decide(
        &self,
        plugin_id: &PluginId,
        request: &CapabilityRequest,
    ) -> Result<PermissionDecision, CoreError> {
        if self
            .global_grants
            .iter()
            .any(|grant| grant_covers(grant, request))
            || self.plugin_grants.iter().any(|(granted_plugin, grant)| {
                granted_plugin == plugin_id && grant_covers(grant, request)
            })
        {
            Ok(PermissionDecision::Allow)
        } else {
            Ok(PermissionDecision::Deny {
                reason: "capability request is denied by default".to_string(),
            })
        }
    }
}

fn grant_covers(grant: &CapabilityRequest, request: &CapabilityRequest) -> bool {
    grant.capability == request.capability
        && (grant.scope == CapabilityScope::Any || grant.scope == request.scope)
}

pub struct CoreEngine<R, P, E> {
    runtime: R,
    #[allow(dead_code)]
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

        let db_path = self
            .config
            .plugin_dir
            .parent()
            .unwrap_or(&self.config.plugin_dir)
            .join("polyglid.db");
        if db_path.exists() {
            if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                if let Ok(status) = conn.query_row(
                    "SELECT status FROM plugins WHERE id = ?",
                    [manifest.id.as_str()],
                    |row| row.get::<_, String>(0),
                ) {
                    if status == "Disabled" {
                        return Err(CoreError::Runtime(format!(
                            "Plugin '{}' is currently disabled in the workspace",
                            manifest.id.as_str()
                        )));
                    }
                }
            }

            let active_profile_name = {
                let mut profile_name = "Balanced".to_string();
                if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                    if let Ok(val) = conn.query_row(
                        "SELECT value FROM settings WHERE key = 'security_profile' AND scope = 'Workspace'",
                        [],
                        |row| row.get::<_, String>(0)
                    ) {
                        profile_name = val;
                    }
                }
                profile_name
            };

            let profile = match active_profile_name.as_str() {
                "Strict" => crate::security::profiles::SecurityProfile::strict(),
                "Development" => crate::security::profiles::SecurityProfile::development(),
                _ => crate::security::profiles::SecurityProfile::balanced(),
            };

            let (sig_status, plugin_source) = {
                let mut status = "Missing".to_string();
                let mut source = "Local".to_string();
                if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                    if let Ok((s, src)) = conn.query_row(
                        "SELECT status, source FROM plugin_signatures LEFT JOIN plugins ON plugin_signatures.plugin_id = plugins.id WHERE plugins.id = ?",
                        [manifest.id.as_str()],
                        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                    ) {
                        status = s;
                        source = src;
                    } else if let Ok(src) = conn.query_row(
                        "SELECT source FROM plugins WHERE id = ?",
                        [manifest.id.as_str()],
                        |row| row.get::<_, String>(0)
                    ) {
                        source = src;
                    }
                }
                (status, source)
            };

            let require_sig = if plugin_source == "Local" {
                profile.name == "Strict"
            } else {
                profile.require_signature
            };

            if require_sig && (sig_status == "Missing" || sig_status == "Invalid") {
                return Err(CoreError::Runtime(format!(
                    "Signature check failed: plugin signature is {}",
                    sig_status
                )));
            }

            if profile.require_trusted_publisher && sig_status == "UnknownPublisher" {
                return Err(CoreError::Runtime(
                    "Signature check failed: publisher is untrusted".to_string(),
                ));
            }

            if sig_status == "Revoked" {
                return Err(CoreError::Runtime(
                    "Signature check failed: publisher key is revoked".to_string(),
                ));
            }

            let mut actual_config = self.config.clone();
            if let Some(fuel) = profile.max_fuel {
                actual_config.max_wasm_fuel = fuel;
            }

            for request_cap in &manifest.requested_capabilities {
                let mut approved = false;
                if profile
                    .allowed_capabilities
                    .contains(&request_cap.capability)
                {
                    approved = true;
                } else {
                    if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();

                        if let Ok((decision, expiration)) = conn.query_row(
                            "SELECT decision, expiration FROM permissions WHERE plugin_id = ? AND capability = ?",
                            [manifest.id.as_str(), &request_cap.capability.to_string()],
                            |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<u64>>(1)?))
                        ) {
                            let expired = match expiration {
                                Some(exp) => exp < now,
                                None => false,
                            };
                            if decision == "Allow" && !expired {
                                approved = true;
                            }
                        }
                    }
                }

                if !approved {
                    self.events.emit(PolyGlidEvent::CapabilityDenied {
                        plugin_id: manifest.id.clone(),
                        capability: request_cap.to_string(),
                        reason: "Not approved in permission engine or profile policy".to_string(),
                    });
                    return Err(CoreError::CapabilityDenied {
                        plugin_id: manifest.id,
                        request: request_cap.clone(),
                        reason: "Permission not approved in Workspace".to_string(),
                    });
                } else {
                    self.events.emit(PolyGlidEvent::CapabilityAllowed {
                        plugin_id: manifest.id.clone(),
                        capability: request_cap.to_string(),
                    });
                }
            }

            self.events.emit(PolyGlidEvent::PluginRunStarted {
                plugin_id: manifest.id.clone(),
                target: request.target.as_str().to_string(),
            });

            match self.runtime.execute(&request, &actual_config) {
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
        } else {
            for request_cap in &manifest.requested_capabilities {
                match self.permissions.decide(&manifest.id, request_cap) {
                    Ok(PermissionDecision::Allow) => {
                        self.events.emit(PolyGlidEvent::CapabilityAllowed {
                            plugin_id: manifest.id.clone(),
                            capability: request_cap.to_string(),
                        });
                    }
                    Ok(PermissionDecision::Deny { reason }) => {
                        self.events.emit(PolyGlidEvent::CapabilityDenied {
                            plugin_id: manifest.id.clone(),
                            capability: request_cap.to_string(),
                            reason: reason.clone(),
                        });
                        return Err(CoreError::CapabilityDenied {
                            plugin_id: manifest.id,
                            request: request_cap.clone(),
                            reason,
                        });
                    }
                    Err(err) => {
                        self.events.emit(PolyGlidEvent::CapabilityCheckFailed {
                            plugin_id: manifest.id.clone(),
                            capability: request_cap.to_string(),
                            message: err.to_string(),
                        });
                        return Err(err);
                    }
                }
            }

            self.events.emit(PolyGlidEvent::PluginRunStarted {
                plugin_id: manifest.id.clone(),
                target: request.target.as_str().to_string(),
            });

            match self.runtime.execute(&request, &self.config) {
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
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub fn events(&self) -> &E {
        &self.events
    }

    pub fn runtime(&self) -> &R {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut R {
        &mut self.runtime
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
        request: CapabilityRequest,
        reason: String,
    },
    PermissionStore(String),
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
                request,
                reason,
            } => write!(
                f,
                "plugin '{}' is missing required capability {request}: {reason}",
                plugin_id.as_str()
            ),
            Self::PermissionStore(message) => write!(f, "permission store error: {message}"),
        }
    }
}

impl std::error::Error for CoreError {}

#[cfg(test)]
mod tests {
    use super::*;
    use polyglid_events::VecEventSink;
    use polyglid_plugin_api::{Issue, Severity};

    struct FakeRuntime {
        capabilities: Vec<CapabilityRequest>,
    }

    impl PluginRuntime for FakeRuntime {
        fn inspect(&self, _plugin: &PluginRef) -> Result<PluginManifest, CoreError> {
            Ok(PluginManifest {
                id: PluginId::new("demo").expect("valid id"),
                name: "Demo".to_string(),
                version: "0.1.0".to_string(),
                requested_capabilities: self.capabilities.clone(),
            })
        }

        fn inspect_metadata(
            &self,
            _plugin: &PluginRef,
        ) -> Result<polyglid_plugin_api::ApiPluginMetadata, CoreError> {
            Ok(polyglid_plugin_api::ApiPluginMetadata {
                name: "demo".to_string(),
                display_name: "Demo".to_string(),
                version: "0.1.0".to_string(),
                description: "mocked runtime".to_string(),
                author: "mock author".to_string(),
            })
        }

        fn execute(
            &self,
            request: &PluginRunRequest,
            _config: &AppConfig,
        ) -> Result<PluginReport, CoreError> {
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
            FakeRuntime {
                capabilities: Vec::new(),
            },
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

    #[test]
    fn engine_denies_requested_capabilities_by_default() {
        let mut engine = CoreEngine::new(
            FakeRuntime {
                capabilities: vec![CapabilityRequest::unscoped(Capability::NetworkConnect)],
            },
            InMemoryPermissionStore::default(),
            VecEventSink::default(),
            AppConfig::development(),
        )
        .expect("valid engine");

        let err = engine
            .run_plugin(PluginRunRequest {
                plugin: PluginRef::from_path("demo.wasm"),
                target: Target::parse("example.com").expect("valid target"),
            })
            .expect_err("capability denied");

        assert!(matches!(err, CoreError::CapabilityDenied { .. }));
        assert!(matches!(
            engine.events().events().last(),
            Some(PolyGlidEvent::CapabilityDenied { capability, .. })
                if capability == "network-connect"
        ));
    }

    #[test]
    fn engine_runs_when_requested_capability_is_granted() {
        let mut permissions = InMemoryPermissionStore::default();
        permissions.grant_for_all(Capability::DnsResolve);
        let mut engine = CoreEngine::new(
            FakeRuntime {
                capabilities: vec![CapabilityRequest::unscoped(Capability::DnsResolve)],
            },
            permissions,
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
        assert!(engine.events().events().iter().any(|event| matches!(
            event,
            PolyGlidEvent::CapabilityAllowed { capability, .. } if capability == "dns-resolve"
        )));
    }

    #[test]
    fn unscoped_grant_covers_scoped_request() {
        let mut permissions = InMemoryPermissionStore::default();
        permissions.grant_for_all(Capability::NetworkConnect);
        let mut engine = CoreEngine::new(
            FakeRuntime {
                capabilities: vec![CapabilityRequest::new(
                    Capability::NetworkConnect,
                    CapabilityScope::HostPort {
                        host: "example.com".to_string(),
                        port: 443,
                    },
                )],
            },
            permissions,
            VecEventSink::default(),
            AppConfig::development(),
        )
        .expect("valid engine");

        engine
            .run_plugin(PluginRunRequest {
                plugin: PluginRef::from_path("demo.wasm"),
                target: Target::parse("example.com").expect("valid target"),
            })
            .expect("plugin runs");

        assert!(engine.events().events().iter().any(|event| matches!(
            event,
            PolyGlidEvent::CapabilityAllowed { capability, .. }
                if capability == "network-connect (host=example.com,port=443)"
        )));
    }

    struct FailingPermissionStore;

    impl PermissionStore for FailingPermissionStore {
        fn decide(
            &self,
            _plugin_id: &PluginId,
            _request: &CapabilityRequest,
        ) -> Result<PermissionDecision, CoreError> {
            Err(CoreError::PermissionStore("unavailable".to_string()))
        }
    }

    #[test]
    fn engine_audits_permission_check_failures() {
        let mut engine = CoreEngine::new(
            FakeRuntime {
                capabilities: vec![CapabilityRequest::unscoped(Capability::EnvironmentRead)],
            },
            FailingPermissionStore,
            VecEventSink::default(),
            AppConfig::development(),
        )
        .expect("valid engine");

        let err = engine
            .run_plugin(PluginRunRequest {
                plugin: PluginRef::from_path("demo.wasm"),
                target: Target::parse("example.com").expect("valid target"),
            })
            .expect_err("permission store fails");

        assert!(matches!(err, CoreError::PermissionStore(_)));
        assert!(matches!(
            engine.events().events().last(),
            Some(PolyGlidEvent::CapabilityCheckFailed { capability, .. })
                if capability == "environment-read"
        ));
    }
}
