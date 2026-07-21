use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use polyglid_config::plugin_registry::{
    PluginRegistryEntry, PluginSource as CorePluginSource, PluginStatus as CorePluginStatus,
};
use polyglid_config::AppConfig;
use polyglid_core::execution::{
    ExecutionConfig, ExecutionEvent as CoreExecutionEvent, ExecutionManager, Job as CoreExecution,
    JobMetrics as CoreExecutionMetrics, JobState as CoreExecutionState,
};
use polyglid_core::plugin_manager::PluginManager;
use polyglid_core::services::WorkspaceCatalogService;
use polyglid_core::store::{DbProject, DbWorkspace, WorkspaceStore};
use polyglid_core::Target as CoreTarget;
use polyglid_plugin_api::{
    ApiPluginMetadata, Capability as CoreCapability, CapabilityRequest as CoreCapabilityRequest,
    CapabilityScope as CoreCapabilityScope, Issue as CoreIssue, PluginId, PluginManifest,
    PluginReport as CorePluginReport, Severity as CoreSeverity,
};
use polyglid_runtime::WasmRuntime;

use super::{
    BootstrapSnapshot, CapabilityKind, CapabilityRequest, CapabilityScope, ClientError,
    ClientGateway, ClientResult, Execution, ExecutionEvent, ExecutionMetrics, ExecutionReport,
    ExecutionState, ExecutionSubscription, Issue, JobId, Plugin, PluginInspection, PluginSource,
    PluginStatus, Project, Report, ReportFormat, SavedTarget, Severity, ShellPreferences,
    StartExecutionRequest, Workspace,
};

#[derive(Clone)]
pub struct LocalClient {
    inner: Arc<LocalClientInner>,
}

struct LocalClientInner {
    catalog: WorkspaceCatalogService,
    plugins: Arc<PluginManager<WasmRuntime>>,
    executions: Arc<ExecutionManager<WasmRuntime>>,
    store: WorkspaceStore,
    default_workspace_root: PathBuf,
    data_directory: PathBuf,
}

impl LocalClient {
    pub fn open_default() -> ClientResult<Self> {
        Self::open(data_directory()?, default_workspace_root()?)
    }

    /// Open a local desktop client with explicit paths. This is useful for
    /// portable installations, tests, and future profile selection.
    pub fn open(
        data_directory: impl AsRef<Path>,
        default_workspace_root: impl AsRef<Path>,
    ) -> ClientResult<Self> {
        let data_directory = data_directory.as_ref().to_path_buf();
        let default_workspace_root = default_workspace_root.as_ref().to_path_buf();
        let plugin_directory = data_directory.join("plugins");
        let reports_directory = data_directory.join("reports");

        for directory in [&data_directory, &plugin_directory, &reports_directory] {
            fs::create_dir_all(directory).map_err(|error| {
                ClientError::operation(
                    "create desktop data directory",
                    format!("{}: {error}", directory.display()),
                )
            })?;
        }

        let database_path = data_directory.join("polyglid.db");
        let config = AppConfig {
            plugin_dir: plugin_directory,
            reports_dir: reports_directory,
            ..AppConfig::development()
        };
        let catalog = WorkspaceCatalogService::open(&database_path)
            .map_err(|error| ClientError::operation("open workspace catalog", error))?;
        let store = WorkspaceStore::new(&database_path)
            .map_err(|error| ClientError::operation("open desktop database", error))?;
        let plugins = Arc::new(
            PluginManager::new(Arc::new(WasmRuntime::new()), &config, store.clone())
                .map_err(|error| ClientError::operation("open plugin manager", error))?,
        );
        plugins
            .sync_directory()
            .map_err(|error| ClientError::operation("synchronize plugin directory", error))?;
        let executions = Arc::new(ExecutionManager::new_with_config(
            WasmRuntime::new(),
            Some(store.clone()),
            config,
        ));

        Ok(Self {
            inner: Arc::new(LocalClientInner {
                catalog,
                plugins,
                executions,
                store,
                default_workspace_root,
                data_directory,
            }),
        })
    }

    pub fn data_directory(&self) -> &Path {
        &self.inner.data_directory
    }

    fn load_active_catalog(&self) -> ClientResult<(Vec<Workspace>, Workspace, Vec<Project>)> {
        let mut workspaces = self
            .inner
            .catalog
            .list_workspaces()
            .map_err(|error| ClientError::operation("list workspaces", error))?;
        if workspaces.is_empty() {
            self.inner
                .catalog
                .register_workspace("PolyGlid Projects", &self.inner.default_workspace_root)
                .map_err(|error| ClientError::operation("create default workspace", error))?;
            workspaces = self
                .inner
                .catalog
                .list_workspaces()
                .map_err(|error| ClientError::operation("reload workspaces", error))?;
        }

        let active_id = if let Some(workspace) = workspaces.iter().find(|item| item.is_active) {
            workspace.id.clone()
        } else {
            let first = workspaces
                .first()
                .ok_or_else(|| ClientError::Unavailable("no workspace is available".to_string()))?;
            self.inner
                .catalog
                .activate(&first.id)
                .map_err(|error| ClientError::operation("activate default workspace", error))?;
            first.id.clone()
        };

        let projects = self
            .inner
            .catalog
            .discover(&active_id)
            .map_err(|error| ClientError::operation("discover projects", error))?;
        let workspaces = self
            .inner
            .catalog
            .list_workspaces()
            .map_err(|error| ClientError::operation("reload active workspace", error))?;
        let active = workspaces
            .iter()
            .find(|item| item.is_active)
            .cloned()
            .ok_or_else(|| {
                ClientError::Unavailable("active workspace was not found".to_string())
            })?;

        Ok((
            workspaces.into_iter().map(workspace_from_db).collect(),
            workspace_from_db(active),
            projects.into_iter().map(project_from_db).collect(),
        ))
    }

    fn load_shell_preferences(&self) -> ClientResult<ShellPreferences> {
        let settings = self.inner.store.settings();
        let defaults = ShellPreferences::default();
        Ok(ShellPreferences {
            sidebar_visible: setting_bool(
                settings.get("ui.sidebar_visible"),
                defaults.sidebar_visible,
            )?,
            bottom_panel_visible: setting_bool(
                settings.get("ui.bottom_panel_visible"),
                defaults.bottom_panel_visible,
            )?,
            sidebar_width: setting_number(
                settings.get("ui.sidebar_width"),
                defaults.sidebar_width,
                180.0,
                480.0,
            )?,
            bottom_panel_height: setting_number(
                settings.get("ui.bottom_panel_height"),
                defaults.bottom_panel_height,
                120.0,
                520.0,
            )?,
        })
    }

    fn execution(&self, job_id: JobId) -> ClientResult<Execution> {
        self.list_executions()?
            .into_iter()
            .find(|execution| execution.id == job_id)
            .ok_or_else(|| ClientError::NotFound {
                resource: "execution",
                id: job_id.to_string(),
            })
    }
}

impl ClientGateway for LocalClient {
    fn bootstrap(&self) -> ClientResult<BootstrapSnapshot> {
        let (workspaces, active_workspace, projects) = self.load_active_catalog()?;
        Ok(BootstrapSnapshot {
            workspaces,
            active_workspace,
            projects,
            plugins: self.list_plugins()?,
            targets: self.list_targets()?,
            executions: self.list_executions()?,
            reports: self.list_reports()?,
            shell: self.load_shell_preferences()?,
        })
    }

    fn list_workspaces(&self) -> ClientResult<Vec<Workspace>> {
        self.inner
            .catalog
            .list_workspaces()
            .map(|items| items.into_iter().map(workspace_from_db).collect())
            .map_err(|error| ClientError::operation("list workspaces", error))
    }

    fn register_workspace(&self, name: &str, root_path: &str) -> ClientResult<Workspace> {
        self.inner
            .catalog
            .register_workspace(name, Path::new(root_path))
            .map(workspace_from_db)
            .map_err(|error| ClientError::operation("register workspace", error))
    }

    fn activate_workspace(&self, workspace_id: &str) -> ClientResult<()> {
        self.inner
            .catalog
            .activate(workspace_id)
            .map_err(|error| ClientError::operation("activate workspace", error))
    }

    fn refresh_workspace(&self, workspace_id: &str) -> ClientResult<Vec<Project>> {
        self.inner
            .catalog
            .discover(workspace_id)
            .map(|items| items.into_iter().map(project_from_db).collect())
            .map_err(|error| ClientError::operation("refresh workspace", error))
    }

    fn list_projects(&self, workspace_id: &str) -> ClientResult<Vec<Project>> {
        self.inner
            .catalog
            .list_projects(workspace_id)
            .map(|items| items.into_iter().map(project_from_db).collect())
            .map_err(|error| ClientError::operation("list projects", error))
    }

    fn create_project(&self, workspace_id: &str, name: &str) -> ClientResult<Project> {
        self.inner
            .catalog
            .create_project(workspace_id, name)
            .map(project_from_db)
            .map_err(|error| ClientError::operation("create project", error))
    }

    fn rename_project(&self, project_id: &str, name: &str) -> ClientResult<Project> {
        self.inner
            .catalog
            .rename_project(project_id, name)
            .map(project_from_db)
            .map_err(|error| ClientError::operation("rename project", error))
    }

    fn remove_project(&self, project_id: &str, delete_files: bool) -> ClientResult<()> {
        self.inner
            .catalog
            .remove_project(project_id, delete_files)
            .map_err(|error| ClientError::operation("remove project", error))
    }

    fn list_plugins(&self) -> ClientResult<Vec<Plugin>> {
        Ok(self
            .inner
            .plugins
            .get_plugins()
            .into_iter()
            .map(|entry| {
                let requests = self
                    .inner
                    .plugins
                    .validate_plugin(&entry.path)
                    .ok()
                    .map_or_else(Vec::new, |(manifest, _)| {
                        manifest
                            .requested_capabilities
                            .into_iter()
                            .map(capability_request_from_core)
                            .collect()
                    });
                plugin_from_registry(entry, requests)
            })
            .collect())
    }

    fn inspect_plugin(&self, path: &str) -> ClientResult<PluginInspection> {
        let (manifest, metadata) = self
            .inner
            .plugins
            .validate_plugin(Path::new(path))
            .map_err(|error| ClientError::operation("inspect plugin", error))?;
        Ok(plugin_inspection(manifest, metadata))
    }

    fn install_plugin(&self, path: &str) -> ClientResult<Plugin> {
        let entry = self
            .inner
            .plugins
            .install_plugin(
                Path::new(path),
                CorePluginSource::LocalPath(PathBuf::from(path)),
            )
            .map_err(|error| ClientError::operation("install plugin", error))?;
        let requests = self
            .inner
            .plugins
            .validate_plugin(&entry.path)
            .map_err(|error| ClientError::operation("reload installed plugin", error))?
            .0
            .requested_capabilities
            .into_iter()
            .map(capability_request_from_core)
            .collect();
        Ok(plugin_from_registry(entry, requests))
    }

    fn set_plugin_enabled(&self, plugin_id: &str, enabled: bool) -> ClientResult<()> {
        let id = parse_plugin_id(plugin_id)?;
        self.inner
            .plugins
            .toggle_plugin_enabled(&id, enabled)
            .map_err(|error| ClientError::operation("change plugin status", error))
    }

    fn uninstall_plugin(&self, plugin_id: &str) -> ClientResult<()> {
        let id = parse_plugin_id(plugin_id)?;
        self.inner
            .plugins
            .uninstall_plugin(&id)
            .map_err(|error| ClientError::operation("uninstall plugin", error))
    }

    fn list_targets(&self) -> ClientResult<Vec<SavedTarget>> {
        self.inner
            .store
            .targets()
            .list()
            .map(|items| {
                items
                    .into_iter()
                    .map(|(name, group)| SavedTarget { name, group })
                    .collect()
            })
            .map_err(|error| ClientError::operation("list targets", error))
    }

    fn add_target(&self, name: &str, group: Option<&str>) -> ClientResult<SavedTarget> {
        let target = CoreTarget::parse(name).map_err(|error| ClientError::InvalidInput {
            field: "target",
            message: error.to_string(),
        })?;
        let name = target.as_str().to_string();
        let group = group
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        self.inner
            .store
            .targets()
            .add(&name, group.as_deref())
            .map_err(|error| ClientError::operation("save target", error))?;
        Ok(SavedTarget { name, group })
    }

    fn remove_target(&self, name: &str) -> ClientResult<()> {
        self.inner
            .store
            .targets()
            .remove(name)
            .map_err(|error| ClientError::operation("remove target", error))
    }

    fn start_execution(&self, request: StartExecutionRequest) -> ClientResult<JobId> {
        if request.fuel_limit == 0 {
            return Err(ClientError::InvalidInput {
                field: "fuel limit",
                message: "must be greater than zero".to_string(),
            });
        }
        if request.timeout.is_zero() {
            return Err(ClientError::InvalidInput {
                field: "execution timeout",
                message: "must be greater than zero".to_string(),
            });
        }
        let plugin_id = parse_plugin_id(&request.plugin_id)?;
        let target =
            CoreTarget::parse(&request.target).map_err(|error| ClientError::InvalidInput {
                field: "target",
                message: error.to_string(),
            })?;
        let entry =
            self.inner
                .plugins
                .get_plugin(&plugin_id)
                .ok_or_else(|| ClientError::NotFound {
                    resource: "plugin",
                    id: request.plugin_id.clone(),
                })?;
        if entry.status != CorePluginStatus::Enabled {
            return Err(ClientError::Conflict(format!(
                "plugin '{}' is {}",
                request.plugin_id, entry.status
            )));
        }

        // Re-inspect the installed component so approval is checked against the
        // executable being launched, not only against cached registry metadata.
        let (manifest, _) = self
            .inner
            .plugins
            .validate_plugin(&entry.path)
            .map_err(|error| ClientError::operation("verify installed plugin", error))?;
        let requested: Vec<CapabilityKind> = manifest
            .requested_capabilities
            .iter()
            .map(|item| capability_from_core(item.capability))
            .collect();
        let (missing, unexpected) =
            capability_approval_gaps(&requested, &request.approved_capabilities);
        if !unexpected.is_empty() {
            return Err(ClientError::UnexpectedCapabilityApproval {
                plugin_id: request.plugin_id,
                capabilities: unexpected,
            });
        }
        if !missing.is_empty() {
            let _ = self.inner.store.audit_logger().log(
                "CapabilityApprovalRequired",
                Some(plugin_id.as_str()),
                serde_json::json!({
                    "target": target.as_str(),
                    "missing": missing.iter().map(|item| item.as_str()).collect::<Vec<_>>()
                }),
            );
            return Err(ClientError::CapabilityApprovalRequired {
                plugin_id: request.plugin_id,
                missing,
            });
        }

        let approved_capabilities: Vec<CoreCapability> =
            requested.iter().copied().map(capability_to_core).collect();
        let _ = self.inner.store.audit_logger().log(
            "CapabilityApprovalGranted",
            Some(plugin_id.as_str()),
            serde_json::json!({
                "target": target.as_str(),
                "duration": "once",
                "capabilities": requested.iter().map(|item| item.as_str()).collect::<Vec<_>>()
            }),
        );
        let job_id = self.inner.executions.submit_job(
            entry.path.to_string_lossy().into_owned(),
            target.as_str().to_string(),
            ExecutionConfig {
                fuel_limit: request.fuel_limit,
                timeout: request.timeout,
                memory_limit: request.memory_limit,
                allowed_capabilities: approved_capabilities,
            },
        );
        Ok(JobId::new(job_id))
    }

    fn subscribe_executions(&self) -> ClientResult<ExecutionSubscription> {
        Ok(ExecutionSubscription {
            receiver: self.inner.executions.subscribe(),
        })
    }

    fn wait_for_execution(&self, job_id: JobId, timeout: Duration) -> ClientResult<Execution> {
        let mut subscription = self.subscribe_executions()?;
        let current = self.execution(job_id)?;
        if current.state.is_terminal() {
            return Ok(current);
        }

        let deadline = Instant::now() + timeout;
        loop {
            let now = Instant::now();
            if now >= deadline {
                return Err(ClientError::WaitTimedOut { job_id });
            }
            let event = subscription.recv_timeout(deadline - now)?;
            if let Some(event) = event {
                if event.job_id() == job_id {
                    let execution = self.execution(job_id)?;
                    if execution.state.is_terminal() {
                        return Ok(execution);
                    }
                }
            } else {
                return Err(ClientError::WaitTimedOut { job_id });
            }
        }
    }

    fn cancel_execution(&self, job_id: JobId) -> ClientResult<()> {
        self.inner
            .executions
            .cancel_job(job_id.as_uuid())
            .map_err(|error| ClientError::operation("cancel execution", error))
    }

    fn list_executions(&self) -> ClientResult<Vec<Execution>> {
        let records = self
            .inner
            .store
            .executions()
            .list()
            .map_err(|error| ClientError::operation("list execution history", error))?;
        let by_id: HashMap<_, _> = records
            .into_iter()
            .map(|record| (record.job_id, record))
            .collect();
        let mut executions: Vec<_> = self
            .inner
            .executions
            .get_jobs()
            .into_iter()
            .map(|job| {
                let record = by_id.get(&job.id);
                execution_from_core(job, record)
            })
            .collect();
        executions.sort_by_key(|item| std::cmp::Reverse((item.started_at, item.created_at)));
        Ok(executions)
    }

    fn list_reports(&self) -> ClientResult<Vec<Report>> {
        self.inner
            .store
            .reports()
            .list()
            .map(|items| items.into_iter().map(report_from_db).collect())
            .map_err(|error| ClientError::operation("list reports", error))
    }

    fn get_report(&self, report_id: &str) -> ClientResult<Report> {
        self.inner
            .store
            .reports()
            .get(report_id)
            .map_err(|error| ClientError::operation("load report", error))?
            .map(report_from_db)
            .ok_or_else(|| ClientError::NotFound {
                resource: "report",
                id: report_id.to_string(),
            })
    }

    fn export_report(&self, report_id: &str, format: ReportFormat) -> ClientResult<String> {
        let report = self.get_report(report_id)?;
        let plugin = self
            .inner
            .plugins
            .get_plugins()
            .into_iter()
            .find(|item| item.id.as_str() == report.plugin_id);
        let execution = self.execution(report.job_id).ok();
        let security_profile = self
            .inner
            .store
            .settings()
            .get("security_profile")
            .map_err(|error| ClientError::operation("load security profile", error))?
            .unwrap_or_else(|| "Balanced".to_string());
        let payload = polyglid_core::execution::reports::ExportedReport {
            metadata: polyglid_core::execution::reports::ReportMetadata {
                polyglid_version: env!("CARGO_PKG_VERSION").to_string(),
                plugin_id: report.plugin_id.clone(),
                plugin_version: plugin
                    .as_ref()
                    .map_or_else(|| "unknown".to_string(), |item| item.version.to_string()),
                target: report.target.clone(),
                timestamp: report.created_at,
                security_profile,
                execution_duration_ms: execution.map_or(0, |item| item.duration_ms),
                report_format_version: "1.0".to_string(),
            },
            report: CorePluginReport {
                plugin_name: plugin.map_or_else(|| report.plugin_id.clone(), |item| item.name),
                target_tested: report.target.clone(),
                issues: report.issues.into_iter().map(issue_to_core).collect(),
                summary: report.summary,
            },
        };
        let exported = match format {
            ReportFormat::Json => polyglid_core::execution::reports::json::export(&payload),
            ReportFormat::Markdown => polyglid_core::execution::reports::markdown::export(&payload),
            ReportFormat::Html => polyglid_core::execution::reports::html::export(&payload),
            ReportFormat::Sarif => polyglid_core::execution::reports::sarif::export(&payload),
        };
        exported.map_err(|error| ClientError::operation("export report", error))
    }

    fn save_shell_preferences(&self, preferences: &ShellPreferences) -> ClientResult<()> {
        if !preferences.sidebar_width.is_finite() || !preferences.bottom_panel_height.is_finite() {
            return Err(ClientError::InvalidInput {
                field: "shell dimensions",
                message: "dimensions must be finite numbers".to_string(),
            });
        }
        let settings = self.inner.store.settings();
        let values = [
            (
                "ui.sidebar_visible",
                preferences.sidebar_visible.to_string(),
            ),
            (
                "ui.bottom_panel_visible",
                preferences.bottom_panel_visible.to_string(),
            ),
            (
                "ui.sidebar_width",
                preferences
                    .sidebar_width
                    .clamp(180.0, 480.0)
                    .round()
                    .to_string(),
            ),
            (
                "ui.bottom_panel_height",
                preferences
                    .bottom_panel_height
                    .clamp(120.0, 520.0)
                    .round()
                    .to_string(),
            ),
        ];
        for (key, value) in values {
            settings
                .set(key, &value, "Workspace")
                .map_err(|error| ClientError::operation("save shell preferences", error))?;
        }
        Ok(())
    }
}

pub(crate) fn execution_event_from_core(event: CoreExecutionEvent) -> ExecutionEvent {
    match event {
        CoreExecutionEvent::JobStateChanged { job_id, state } => ExecutionEvent::StateChanged {
            job_id: JobId::new(job_id),
            state: execution_state_from_core(state),
        },
        CoreExecutionEvent::JobFinished {
            job_id,
            report,
            metrics,
        } => ExecutionEvent::Finished {
            job_id: JobId::new(job_id),
            report: execution_report_from_core(report),
            metrics: execution_metrics_from_core(metrics),
        },
        CoreExecutionEvent::JobFailed {
            job_id,
            error,
            metrics,
        } => ExecutionEvent::Failed {
            job_id: JobId::new(job_id),
            error,
            metrics: metrics.map(execution_metrics_from_core),
        },
        CoreExecutionEvent::JobLog { job_id, message } => ExecutionEvent::Log {
            job_id: JobId::new(job_id),
            message,
        },
    }
}

fn workspace_from_db(value: DbWorkspace) -> Workspace {
    Workspace {
        id: value.id,
        name: value.name,
        root_path: value.root_path,
        is_active: value.is_active,
        discovery_state: value.discovery_state,
        last_error: value.last_error,
        created_at: value.created_at,
        updated_at: value.updated_at,
        last_opened_at: value.last_opened_at,
    }
}

fn project_from_db(value: DbProject) -> Project {
    Project {
        id: value.id,
        workspace_id: value.workspace_id,
        name: value.name,
        path: value.path,
        kind: value.kind,
        archived: value.archived,
        created_at: value.created_at,
        updated_at: value.updated_at,
    }
}

fn plugin_from_registry(
    value: PluginRegistryEntry,
    requested_capabilities: Vec<CapabilityRequest>,
) -> Plugin {
    Plugin {
        id: value.id.as_str().to_string(),
        name: value.name,
        version: value.version.to_string(),
        author: value.author,
        description: value.description,
        requested_capabilities,
        capabilities: value
            .capabilities
            .into_iter()
            .map(capability_from_core)
            .collect(),
        status: match value.status {
            CorePluginStatus::Enabled => PluginStatus::Enabled,
            CorePluginStatus::Disabled => PluginStatus::Disabled,
            CorePluginStatus::Invalid => PluginStatus::Invalid,
            CorePluginStatus::UpdateAvailable => PluginStatus::UpdateAvailable,
        },
        source: match value.source {
            CorePluginSource::LocalPath(path) => {
                PluginSource::LocalPath(path.to_string_lossy().into_owned())
            }
            CorePluginSource::Marketplace(name) => PluginSource::Marketplace(name),
            CorePluginSource::Url(url) => PluginSource::Url(url),
        },
        file_size: value.file_size,
        installed_at: value.installed_at,
        last_updated: value.last_updated,
    }
}

fn plugin_inspection(manifest: PluginManifest, metadata: ApiPluginMetadata) -> PluginInspection {
    PluginInspection {
        id: manifest.id.as_str().to_string(),
        name: manifest.name,
        display_name: metadata.display_name,
        version: metadata.version,
        description: metadata.description,
        author: metadata.author,
        requested_capabilities: manifest
            .requested_capabilities
            .into_iter()
            .map(capability_request_from_core)
            .collect(),
    }
}

fn capability_request_from_core(value: CoreCapabilityRequest) -> CapabilityRequest {
    CapabilityRequest {
        capability: capability_from_core(value.capability),
        scope: match value.scope {
            CoreCapabilityScope::Any => CapabilityScope::Any,
            CoreCapabilityScope::Target(target) => CapabilityScope::Target { target },
            CoreCapabilityScope::PathPrefix(path) => CapabilityScope::PathPrefix { path },
            CoreCapabilityScope::HostPort { host, port } => {
                CapabilityScope::HostPort { host, port }
            }
        },
    }
}

fn capability_from_core(value: CoreCapability) -> CapabilityKind {
    match value {
        CoreCapability::NetworkConnect => CapabilityKind::NetworkConnect,
        CoreCapability::NetworkListen => CapabilityKind::NetworkListen,
        CoreCapability::FilesystemRead => CapabilityKind::FilesystemRead,
        CoreCapability::FilesystemWrite => CapabilityKind::FilesystemWrite,
        CoreCapability::ConfigRead => CapabilityKind::ConfigRead,
        CoreCapability::ReportWrite => CapabilityKind::ReportWrite,
        CoreCapability::Crypto => CapabilityKind::Crypto,
        CoreCapability::DnsResolve => CapabilityKind::DnsResolve,
        CoreCapability::ProcessSpawn => CapabilityKind::ProcessSpawn,
        CoreCapability::EnvironmentRead => CapabilityKind::EnvironmentRead,
    }
}

fn capability_to_core(value: CapabilityKind) -> CoreCapability {
    match value {
        CapabilityKind::NetworkConnect => CoreCapability::NetworkConnect,
        CapabilityKind::NetworkListen => CoreCapability::NetworkListen,
        CapabilityKind::FilesystemRead => CoreCapability::FilesystemRead,
        CapabilityKind::FilesystemWrite => CoreCapability::FilesystemWrite,
        CapabilityKind::ConfigRead => CoreCapability::ConfigRead,
        CapabilityKind::ReportWrite => CoreCapability::ReportWrite,
        CapabilityKind::Crypto => CoreCapability::Crypto,
        CapabilityKind::DnsResolve => CoreCapability::DnsResolve,
        CapabilityKind::ProcessSpawn => CoreCapability::ProcessSpawn,
        CapabilityKind::EnvironmentRead => CoreCapability::EnvironmentRead,
    }
}

fn execution_from_core(
    value: CoreExecution,
    record: Option<&polyglid_core::store::execution_store::DbJobRecord>,
) -> Execution {
    let metrics = value.metrics.as_ref();
    let plugin_id = record.map_or_else(
        || {
            Path::new(&value.plugin_path).file_stem().map_or_else(
                || value.plugin_path.clone(),
                |name| name.to_string_lossy().into_owned(),
            )
        },
        |record| record.plugin_id.clone(),
    );
    Execution {
        id: JobId::new(value.id),
        plugin_id,
        target: value.target,
        state: execution_state_from_core(value.state),
        started_at: record.map_or_else(
            || metrics.map_or(0, |metrics| metrics.timestamp),
            |record| record.started_at,
        ),
        duration_ms: metrics.map_or_else(
            || record.map_or(0, |record| record.duration_ms),
            |metrics| metrics.duration.as_millis() as u64,
        ),
        error: value
            .error
            .or_else(|| record.and_then(|record| record.error_message.clone())),
        fuel_consumed: metrics
            .and_then(|metrics| metrics.fuel_consumed)
            .or_else(|| record.map(|record| record.fuel_consumed)),
        created_at: record.map_or(0, |record| record.created_at),
        report: value.report.map(execution_report_from_core),
    }
}

fn execution_state_from_core(value: CoreExecutionState) -> ExecutionState {
    match value {
        CoreExecutionState::Queued => ExecutionState::Queued,
        CoreExecutionState::Starting => ExecutionState::Starting,
        CoreExecutionState::Running => ExecutionState::Running,
        CoreExecutionState::Completed => ExecutionState::Completed,
        CoreExecutionState::Failed => ExecutionState::Failed,
        CoreExecutionState::Cancelled => ExecutionState::Cancelled,
        CoreExecutionState::TimedOut => ExecutionState::TimedOut,
    }
}

fn execution_metrics_from_core(value: CoreExecutionMetrics) -> ExecutionMetrics {
    ExecutionMetrics {
        duration_ms: value.duration.as_millis() as u64,
        fuel_consumed: value.fuel_consumed,
        memory_used: value.memory_used,
        timestamp: value.timestamp,
        stage: value.stage,
    }
}

fn execution_report_from_core(value: CorePluginReport) -> ExecutionReport {
    ExecutionReport {
        plugin_name: value.plugin_name,
        target_tested: value.target_tested,
        issues: value.issues.into_iter().map(issue_from_core).collect(),
        summary: value.summary,
    }
}

fn report_from_db(value: polyglid_core::store::report_store::DbReportRecord) -> Report {
    Report {
        id: value.id,
        job_id: JobId::new(value.job_id),
        plugin_id: value.plugin_id,
        target: value.target,
        summary: value.summary,
        issues: value.issues.into_iter().map(issue_from_core).collect(),
        filepath: value.filepath,
        created_at: value.created_at,
    }
}

fn issue_from_core(value: CoreIssue) -> Issue {
    Issue {
        title: value.title,
        severity: severity_from_core(value.severity),
        description: value.description,
        recommendation: value.recommendation,
    }
}

fn issue_to_core(value: Issue) -> CoreIssue {
    CoreIssue {
        title: value.title,
        severity: severity_to_core(value.severity),
        description: value.description,
        recommendation: value.recommendation,
    }
}

fn severity_from_core(value: CoreSeverity) -> Severity {
    match value {
        CoreSeverity::Info => Severity::Info,
        CoreSeverity::Low => Severity::Low,
        CoreSeverity::Medium => Severity::Medium,
        CoreSeverity::High => Severity::High,
        CoreSeverity::Critical => Severity::Critical,
    }
}

fn severity_to_core(value: Severity) -> CoreSeverity {
    match value {
        Severity::Info => CoreSeverity::Info,
        Severity::Low => CoreSeverity::Low,
        Severity::Medium => CoreSeverity::Medium,
        Severity::High => CoreSeverity::High,
        Severity::Critical => CoreSeverity::Critical,
    }
}

fn parse_plugin_id(value: &str) -> ClientResult<PluginId> {
    PluginId::new(value).map_err(|error| ClientError::InvalidInput {
        field: "plugin id",
        message: error.to_string(),
    })
}

fn capability_approval_gaps(
    requested: &[CapabilityKind],
    approved: &[CapabilityKind],
) -> (Vec<CapabilityKind>, Vec<CapabilityKind>) {
    let requested_set: HashSet<_> = requested.iter().copied().collect();
    let approved_set: HashSet<_> = approved.iter().copied().collect();
    let missing = requested
        .iter()
        .copied()
        .filter(|capability| !approved_set.contains(capability))
        .collect();
    let unexpected = approved
        .iter()
        .copied()
        .filter(|capability| !requested_set.contains(capability))
        .collect();
    (missing, unexpected)
}

fn setting_bool(value: Result<Option<String>, String>, fallback: bool) -> ClientResult<bool> {
    let value = value.map_err(|error| ClientError::operation("load shell preferences", error))?;
    Ok(value.map_or(fallback, |value| value == "true"))
}

fn setting_number(
    value: Result<Option<String>, String>,
    fallback: f64,
    minimum: f64,
    maximum: f64,
) -> ClientResult<f64> {
    let value = value.map_err(|error| ClientError::operation("load shell preferences", error))?;
    Ok(value
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| value.is_finite())
        .unwrap_or(fallback)
        .clamp(minimum, maximum))
}

fn data_directory() -> ClientResult<PathBuf> {
    if let Some(path) = std::env::var_os("POLYGLID_DATA_DIR") {
        if !path.is_empty() {
            return Ok(PathBuf::from(path));
        }
    }
    #[cfg(target_os = "windows")]
    if let Some(path) = std::env::var_os("LOCALAPPDATA") {
        if !path.is_empty() {
            return Ok(PathBuf::from(path).join("PolyGlid"));
        }
    }
    home_directory().map(|home| home.join(".polyglid"))
}

fn default_workspace_root() -> ClientResult<PathBuf> {
    if let Some(path) = std::env::var_os("POLYGLID_WORKSPACE_ROOT") {
        if !path.is_empty() {
            return Ok(PathBuf::from(path));
        }
    }
    home_directory().map(|home| home.join("polyglid-projects"))
}

fn home_directory() -> ClientResult<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
        .ok_or_else(|| {
            ClientError::Unavailable(
                "no home directory is configured; set POLYGLID_DATA_DIR and \
                 POLYGLID_WORKSPACE_ROOT"
                    .to_string(),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary_directory(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!("polyglid-desktop-{label}-{}", uuid::Uuid::new_v4()))
    }

    #[test]
    fn bootstrap_returns_ui_models_and_persists_real_targets() {
        let root = temporary_directory("gateway");
        let data = root.join("data");
        let workspace = root.join("projects");
        let client = LocalClient::open(&data, &workspace).expect("open local client");

        let bootstrap = client.bootstrap().expect("bootstrap");
        assert_eq!(
            bootstrap.active_workspace.root_path,
            workspace.to_string_lossy()
        );
        assert!(bootstrap.projects.is_empty());
        assert!(bootstrap.targets.is_empty());

        let saved = client
            .add_target("example.com", Some("Production"))
            .expect("save target");
        assert_eq!(saved.group.as_deref(), Some("Production"));
        assert_eq!(client.list_targets().expect("list targets"), vec![saved]);

        fs::remove_dir_all(root).expect("clean temporary client");
    }

    #[test]
    fn shell_preferences_are_clamped_and_reloaded() {
        let root = temporary_directory("preferences");
        let client =
            LocalClient::open(root.join("data"), root.join("projects")).expect("open local client");
        client
            .save_shell_preferences(&ShellPreferences {
                sidebar_visible: false,
                bottom_panel_visible: true,
                sidebar_width: 900.0,
                bottom_panel_height: 20.0,
            })
            .expect("save preferences");

        let shell = client.bootstrap().expect("bootstrap").shell;
        assert!(!shell.sidebar_visible);
        assert_eq!(shell.sidebar_width, 480.0);
        assert_eq!(shell.bottom_panel_height, 120.0);

        fs::remove_dir_all(root).expect("clean temporary client");
    }

    #[test]
    fn execution_approval_must_match_every_requested_capability() {
        let requested = [CapabilityKind::DnsResolve, CapabilityKind::ReportWrite];

        let (missing, unexpected) =
            capability_approval_gaps(&requested, &[CapabilityKind::DnsResolve]);
        assert_eq!(missing, vec![CapabilityKind::ReportWrite]);
        assert!(unexpected.is_empty());

        let (missing, unexpected) = capability_approval_gaps(
            &requested,
            &[
                CapabilityKind::DnsResolve,
                CapabilityKind::ReportWrite,
                CapabilityKind::ProcessSpawn,
            ],
        );
        assert!(missing.is_empty());
        assert_eq!(unexpected, vec![CapabilityKind::ProcessSpawn]);

        let (missing, unexpected) = capability_approval_gaps(&requested, &requested);
        assert!(missing.is_empty());
        assert!(unexpected.is_empty());
    }
}
