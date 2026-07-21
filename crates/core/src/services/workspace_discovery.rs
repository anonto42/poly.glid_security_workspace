use std::fs;
use std::path::Path;

use crate::store::DbProject;

pub(super) fn discover_direct_children(
    workspace_id: &str,
    root: &Path,
    now: i64,
) -> Result<Vec<DbProject>, String> {
    if !root.is_dir() {
        return Err(format!(
            "workspace path '{}' is missing or not a directory",
            root.display()
        ));
    }
    let entries = fs::read_dir(root)
        .map_err(|err| format!("failed to read workspace '{}': {err}", root.display()))?;
    let mut projects = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| format!("failed to read workspace entry: {err}"))?;
        let file_type = entry
            .file_type()
            .map_err(|err| format!("failed to inspect workspace entry: {err}"))?;
        if !file_type.is_dir() || file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        projects.push(DbProject {
            id: uuid::Uuid::new_v4().to_string(),
            workspace_id: workspace_id.to_string(),
            name: entry.file_name().to_string_lossy().to_string(),
            path: path.to_string_lossy().to_string(),
            kind: project_kind(&path),
            archived: false,
            created_at: now,
            updated_at: now,
        });
    }
    projects.sort_by_key(|project| project.name.to_lowercase());
    Ok(projects)
}

fn project_kind(path: &Path) -> String {
    if path.join("Cargo.toml").is_file() {
        "Rust"
    } else if path.join("package.json").is_file() {
        "Node"
    } else if path.join("pyproject.toml").is_file() {
        "Python"
    } else {
        "General"
    }
    .to_string()
}
