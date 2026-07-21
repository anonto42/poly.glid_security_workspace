use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use polyglid_config::plugin_registry::{PluginSource, PluginStatus};
use polyglid_config::AppConfig;
use polyglid_core::execution::{ExecutionConfig, ExecutionEvent, ExecutionManager};
use polyglid_core::plugin_manager::PluginManager;
use polyglid_core::services::WorkspaceCatalogService;
use polyglid_core::store::{DbProject, DbWorkspace, WorkspaceStore};
use polyglid_plugin_api::{PluginId, PluginManifest, PluginReport};
use polyglid_runtime::WasmRuntime;

mod shell_preferences;
pub(crate) use shell_preferences::ShellPreferences;

#[derive(Clone, Debug)]
pub(crate) struct WorkspaceSnapshot {
    pub(crate) workspaces: Vec<DbWorkspace>,
    pub(crate) active: DbWorkspace,
    pub(crate) projects: Vec<DbProject>,
    pub(crate) shell: ShellPreferences,
    pub(crate) plugins: Vec<polyglid_config::plugin_registry::PluginRegistryEntry>,
}

#[derive(Clone)]
pub(crate) struct DesktopBackend {
    service: Option<WorkspaceCatalogService>,
    plugins: Option<Arc<PluginManager<WasmRuntime>>>,
    executions: Option<Arc<ExecutionManager<WasmRuntime>>>,
    startup_error: Option<String>,
    default_root: PathBuf,
    database_path: PathBuf,
}

impl DesktopBackend {
    pub(crate) fn open_default() -> Self {
        let default_root = default_workspace_root();
        let opened = data_directory()
            .map(|directory| directory.join("polyglid.db"))
            .and_then(|database_path| {
                open_services(&database_path).map(|services| (services, database_path))
            });
        match opened {
            Ok(((service, plugins, executions), database_path)) => Self {
                service: Some(service),
                plugins: Some(plugins),
                executions: Some(executions),
                startup_error: None,
                default_root,
                database_path,
            },
            Err(error) => Self {
                service: None,
                plugins: None,
                executions: None,
                startup_error: Some(error),
                default_root,
                database_path: PathBuf::new(),
            },
        }
    }

    pub(crate) fn load(&self) -> Result<WorkspaceSnapshot, String> {
        let service = self.service()?;
        let mut workspaces = service.list_workspaces()?;
        if workspaces.is_empty() {
            service.register_workspace("PolyGlid Projects", &self.default_root)?;
            workspaces = service.list_workspaces()?;
        }
        let active = match workspaces.iter().find(|item| item.is_active).cloned() {
            Some(workspace) => workspace,
            None => {
                let first = workspaces
                    .first()
                    .ok_or_else(|| "no workspace is available".to_string())?;
                service.activate(&first.id)?;
                service
                    .active_workspace()?
                    .ok_or_else(|| "active workspace was not persisted".to_string())?
            }
        };
        let projects = service.discover(&active.id)?;
        let workspaces = service.list_workspaces()?;
        let active = workspaces
            .iter()
            .find(|item| item.is_active)
            .cloned()
            .ok_or_else(|| "active workspace was not found after discovery".to_string())?;
        Ok(WorkspaceSnapshot {
            workspaces,
            active,
            projects,
            shell: self.load_shell_preferences()?,
            plugins: self.list_plugins()?,
        })
    }

    pub(crate) fn activate(&self, workspace_id: &str) -> Result<(), String> {
        self.service()?.activate(workspace_id)
    }

    pub(crate) fn create_project(&self, workspace_id: &str, name: &str) -> Result<(), String> {
        self.service()?
            .create_project(workspace_id, name)
            .map(|_| ())
    }

    pub(crate) fn rename_project(&self, project_id: &str, name: &str) -> Result<(), String> {
        self.service()?.rename_project(project_id, name).map(|_| ())
    }

    pub(crate) fn remove_project(
        &self,
        project_id: &str,
        delete_files: bool,
    ) -> Result<(), String> {
        self.service()?.remove_project(project_id, delete_files)
    }

    pub(crate) fn list_plugins(
        &self,
    ) -> Result<Vec<polyglid_config::plugin_registry::PluginRegistryEntry>, String> {
        Ok(self.plugin_manager()?.get_plugins())
    }

    pub(crate) fn toggle_plugin(&self, id: &str, enabled: bool) -> Result<(), String> {
        let id = PluginId::new(id).map_err(|error| error.to_string())?;
        self.plugin_manager()?.toggle_plugin_enabled(&id, enabled)
    }

    pub(crate) fn validate_plugin(
        &self,
        path: &str,
    ) -> Result<(PluginManifest, polyglid_plugin_api::ApiPluginMetadata), String> {
        self.plugin_manager()?.validate_plugin(path.as_ref())
    }

    pub(crate) fn install_plugin(
        &self,
        path: &str,
    ) -> Result<polyglid_config::plugin_registry::PluginRegistryEntry, String> {
        self.plugin_manager()?
            .install_plugin(path.as_ref(), PluginSource::LocalPath(PathBuf::from(path)))
    }

    pub(crate) fn uninstall_plugin(&self, id: &str) -> Result<(), String> {
        let id = PluginId::new(id).map_err(|error| error.to_string())?;
        self.plugin_manager()?.uninstall_plugin(&id)
    }

    pub(crate) fn run_plugin(
        &self,
        id: &str,
        target: &str,
        fuel_limit: u64,
    ) -> Result<PluginReport, String> {
        let id = PluginId::new(id).map_err(|error| error.to_string())?;
        let entry = self
            .plugin_manager()?
            .get_plugin(&id)
            .ok_or_else(|| format!("plugin '{}' is not installed", id.as_str()))?;
        if entry.status != PluginStatus::Enabled {
            return Err(format!("plugin '{}' is not enabled", id.as_str()));
        }
        let manager = self.execution_manager()?;
        let mut events = manager.subscribe();
        let job_id = manager.submit_job(
            entry.path.to_string_lossy().into_owned(),
            target.to_string(),
            ExecutionConfig {
                fuel_limit,
                timeout: Duration::from_secs(30),
                memory_limit: None,
                allowed_capabilities: entry.capabilities,
            },
        );
        loop {
            match events.blocking_recv() {
                Ok(ExecutionEvent::JobFinished {
                    job_id: finished,
                    report,
                    ..
                }) if finished == job_id => return Ok(report),
                Ok(ExecutionEvent::JobFailed {
                    job_id: failed,
                    error,
                    ..
                }) if failed == job_id => return Err(error),
                Ok(ExecutionEvent::JobStateChanged {
                    job_id: changed,
                    state,
                    ..
                }) if changed == job_id
                    && matches!(
                        state,
                        polyglid_core::execution::JobState::Cancelled
                            | polyglid_core::execution::JobState::TimedOut
                    ) =>
                {
                    return Err(format!("plugin execution ended with state {state:?}"));
                }
                Ok(_) => {}
                Err(error) => return Err(format!("execution event stream closed: {error}")),
            }
        }
    }

    fn service(&self) -> Result<&WorkspaceCatalogService, String> {
        self.service.as_ref().ok_or_else(|| {
            self.startup_error
                .clone()
                .unwrap_or_else(|| "desktop services are unavailable".to_string())
        })
    }

    fn plugin_manager(&self) -> Result<&Arc<PluginManager<WasmRuntime>>, String> {
        self.plugins
            .as_ref()
            .ok_or_else(|| self.unavailable_message())
    }

    fn execution_manager(&self) -> Result<&Arc<ExecutionManager<WasmRuntime>>, String> {
        self.executions
            .as_ref()
            .ok_or_else(|| self.unavailable_message())
    }

    fn unavailable_message(&self) -> String {
        self.startup_error
            .clone()
            .unwrap_or_else(|| "desktop services are unavailable".to_string())
    }
}

type DesktopServices = (
    WorkspaceCatalogService,
    Arc<PluginManager<WasmRuntime>>,
    Arc<ExecutionManager<WasmRuntime>>,
);

fn open_services(database_path: &Path) -> Result<DesktopServices, String> {
    let data_dir = database_path
        .parent()
        .ok_or_else(|| "database path has no parent".to_string())?;
    fs::create_dir_all(data_dir).map_err(|err| {
        format!(
            "failed to create PolyGlid data directory '{}': {err}",
            data_dir.display()
        )
    })?;
    let catalog = WorkspaceCatalogService::open(database_path)?;
    let store = WorkspaceStore::new(database_path)?;
    let config = AppConfig {
        plugin_dir: data_dir.join("plugins"),
        reports_dir: data_dir.join("reports"),
        ..AppConfig::development()
    };
    let runtime = Arc::new(WasmRuntime::new());
    let plugins = Arc::new(PluginManager::new(runtime, &config, store.clone())?);
    plugins.sync_directory()?;
    let executions = Arc::new(ExecutionManager::new(WasmRuntime::new(), Some(store)));
    Ok((catalog, plugins, executions))
}

fn data_directory() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("POLYGLID_DATA_DIR") {
        return Ok(PathBuf::from(path));
    }
    home_directory().map(|home| home.join(".polyglid"))
}

fn default_workspace_root() -> PathBuf {
    if let Some(path) = std::env::var_os("POLYGLID_WORKSPACE_ROOT") {
        return PathBuf::from(path);
    }
    if let Ok(current) = std::env::current_dir() {
        let projects = current.join("projects");
        if projects.is_dir() {
            return projects;
        }
    }
    home_directory()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("polyglid-projects")
}

fn home_directory() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
        .ok_or_else(|| {
            "HOME is not set; configure POLYGLID_DATA_DIR and POLYGLID_WORKSPACE_ROOT".to_string()
        })
}
