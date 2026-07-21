use std::fs;
use std::path::{Path, PathBuf};

use crate::store::DbProject;

use super::workspace_catalog::{now_secs, validate_name, WorkspaceCatalogService};

impl WorkspaceCatalogService {
    pub fn create_project(&self, workspace_id: &str, name: &str) -> Result<DbProject, String> {
        validate_name(name, "project")?;
        let workspace = self
            .store
            .workspace_catalog()
            .get(workspace_id)?
            .ok_or_else(|| format!("workspace '{workspace_id}' was not found"))?;
        let path = Path::new(&workspace.root_path).join(name.trim());
        if path.exists() {
            return Err(format!("project '{}' already exists", name.trim()));
        }
        fs::create_dir(&path)
            .map_err(|err| format!("failed to create project '{}': {err}", path.display()))?;
        self.discover(workspace_id)?
            .into_iter()
            .find(|project| Path::new(&project.path) == path)
            .ok_or_else(|| "created project was not discovered".to_string())
    }

    pub fn rename_project(&self, project_id: &str, new_name: &str) -> Result<DbProject, String> {
        validate_name(new_name, "project")?;
        let projects = self.store.project_catalog();
        let project = projects
            .get(project_id)?
            .ok_or_else(|| format!("project '{project_id}' was not found"))?;
        let old_path = PathBuf::from(&project.path);
        let parent = old_path
            .parent()
            .ok_or_else(|| "project path has no parent".to_string())?;
        let new_path = parent.join(new_name.trim());
        if new_path.exists() {
            return Err(format!("project '{}' already exists", new_name.trim()));
        }
        fs::rename(&old_path, &new_path)
            .map_err(|err| format!("failed to rename project: {err}"))?;
        projects.update_path(
            project_id,
            new_name.trim(),
            &new_path.to_string_lossy(),
            now_secs(),
        )?;
        projects
            .get(project_id)?
            .ok_or_else(|| "renamed project could not be reloaded".to_string())
    }

    pub fn remove_project(&self, project_id: &str, delete_files: bool) -> Result<(), String> {
        let projects = self.store.project_catalog();
        let project = projects
            .get(project_id)?
            .ok_or_else(|| format!("project '{project_id}' was not found"))?;
        if delete_files {
            let workspace = self
                .store
                .workspace_catalog()
                .get(&project.workspace_id)?
                .ok_or_else(|| "project workspace was not found".to_string())?;
            validate_direct_child(Path::new(&workspace.root_path), Path::new(&project.path))?;
            fs::remove_dir_all(&project.path)
                .map_err(|err| format!("failed to delete project files: {err}"))?;
        }
        projects.archive(project_id, now_secs())
    }
}

fn validate_direct_child(root: &Path, project: &Path) -> Result<(), String> {
    let root = root
        .canonicalize()
        .map_err(|err| format!("failed to resolve workspace root: {err}"))?;
    let project = project
        .canonicalize()
        .map_err(|err| format!("failed to resolve project path: {err}"))?;
    let is_symlink = project
        .symlink_metadata()
        .map_err(|err| format!("failed to inspect project path: {err}"))?
        .file_type()
        .is_symlink();
    if project.parent() != Some(root.as_path()) || is_symlink {
        return Err("refusing to delete a path outside the direct workspace root".to_string());
    }
    Ok(())
}
