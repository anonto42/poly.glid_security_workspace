use std::fs;
use std::path::{Path, PathBuf};

use super::WorkspaceCatalogService;

fn test_paths(label: &str) -> (PathBuf, PathBuf, PathBuf) {
    let base = std::env::temp_dir().join(format!("polyglid-{label}-{}", uuid::Uuid::new_v4()));
    let root = base.join("workspace");
    let database = base.join("polyglid.db");
    fs::create_dir_all(&root).expect("create test workspace");
    (base, root, database)
}

fn cleanup(path: &Path) {
    fs::remove_dir_all(path).expect("remove test workspace");
}

#[test]
fn workspace_discovery_and_active_selection_survive_reopen() {
    let (base, root, database) = test_paths("workspace-discovery");
    let rust_project = root.join("alpha");
    let general_project = root.join("notes");
    fs::create_dir_all(&rust_project).unwrap();
    fs::create_dir_all(&general_project).unwrap();
    fs::write(rust_project.join("Cargo.toml"), "[package]\nname='alpha'\n").unwrap();

    let service = WorkspaceCatalogService::open(&database).unwrap();
    let workspace = service.register_workspace("Primary", &root).unwrap();
    assert!(workspace.is_active);
    assert_eq!(workspace.discovery_state, "ready");
    let projects = service.list_projects(&workspace.id).unwrap();
    assert_eq!(projects.len(), 2);
    assert_eq!(projects[0].name, "alpha");
    assert_eq!(projects[0].kind, "Rust");
    assert_eq!(projects[1].kind, "General");
    drop(service);

    let reopened = WorkspaceCatalogService::open(&database).unwrap();
    let active = reopened.active_workspace().unwrap().unwrap();
    assert_eq!(active.id, workspace.id);
    assert_eq!(reopened.list_projects(&active.id).unwrap().len(), 2);
    drop(reopened);
    cleanup(&base);
}

#[test]
fn project_create_rename_archive_and_confirmed_delete_are_safe() {
    let (base, root, database) = test_paths("project-lifecycle");
    let service = WorkspaceCatalogService::open(&database).unwrap();
    let workspace = service.register_workspace("Primary", &root).unwrap();

    let created = service.create_project(&workspace.id, "first").unwrap();
    assert!(Path::new(&created.path).is_dir());
    let renamed = service.rename_project(&created.id, "renamed").unwrap();
    assert!(renamed.path.ends_with("renamed"));
    assert!(!root.join("first").exists());

    service.remove_project(&renamed.id, false).unwrap();
    assert!(root.join("renamed").exists());
    assert!(service.list_projects(&workspace.id).unwrap().is_empty());
    assert!(service.discover(&workspace.id).unwrap().is_empty());

    let disposable = service.create_project(&workspace.id, "disposable").unwrap();
    service.remove_project(&disposable.id, true).unwrap();
    assert!(!root.join("disposable").exists());
    drop(service);
    cleanup(&base);
}

#[test]
fn project_names_cannot_escape_the_workspace() {
    let (base, root, database) = test_paths("project-path-safety");
    let service = WorkspaceCatalogService::open(&database).unwrap();
    let workspace = service.register_workspace("Primary", &root).unwrap();
    let error = service
        .create_project(&workspace.id, "../escape")
        .unwrap_err();
    assert!(error.contains("safe path segment"));
    assert!(!base.join("escape").exists());
    drop(service);
    cleanup(&base);
}
