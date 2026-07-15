use std::fs;
use std::path::{Path, PathBuf};

use polyglid_core::services::WorkspaceCatalogService;
use polyglid_core::store::{DbProject, DbWorkspace};

mod shell_preferences;
pub(crate) use shell_preferences::ShellPreferences;

#[derive(Clone, Debug)]
pub(crate) struct WorkspaceSnapshot {
    pub(crate) workspaces: Vec<DbWorkspace>,
    pub(crate) active: DbWorkspace,
    pub(crate) projects: Vec<DbProject>,
    pub(crate) shell: ShellPreferences,
}

#[derive(Clone)]
pub(crate) struct DesktopBackend {
    service: Option<WorkspaceCatalogService>,
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
                open_service(&database_path).map(|service| (service, database_path))
            });
        match opened {
            Ok((service, database_path)) => Self {
                service: Some(service),
                startup_error: None,
                default_root,
                database_path,
            },
            Err(error) => Self {
                service: None,
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

    fn service(&self) -> Result<&WorkspaceCatalogService, String> {
        self.service.as_ref().ok_or_else(|| {
            self.startup_error
                .clone()
                .unwrap_or_else(|| "desktop services are unavailable".to_string())
        })
    }
}

fn open_service(database_path: &Path) -> Result<WorkspaceCatalogService, String> {
    let data_dir = database_path
        .parent()
        .ok_or_else(|| "database path has no parent".to_string())?;
    fs::create_dir_all(data_dir).map_err(|err| {
        format!(
            "failed to create PolyGlid data directory '{}': {err}",
            data_dir.display()
        )
    })?;
    WorkspaceCatalogService::open(database_path)
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
