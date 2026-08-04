use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// OS-aware locations used by the local PolyGlid application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePaths {
    pub data: PathBuf,
    pub config: PathBuf,
    pub cache: PathBuf,
    pub logs: PathBuf,
    pub plugins: PathBuf,
    pub reports: PathBuf,
    pub workspace: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeInitialization {
    pub created_directories: Vec<PathBuf>,
}

impl RuntimePaths {
    pub fn discover() -> Result<Self, String> {
        let data = std::env::var_os("POLYGLID_DATA_DIR")
            .filter(|path| !path.is_empty())
            .map(PathBuf::from)
            .unwrap_or(default_data_directory()?);
        let workspace = std::env::var_os("POLYGLID_WORKSPACE_ROOT")
            .filter(|path| !path.is_empty())
            .map(PathBuf::from)
            .unwrap_or(default_workspace_directory()?);
        Ok(Self::from_roots(data, workspace))
    }

    pub fn from_roots(data: impl Into<PathBuf>, workspace: impl Into<PathBuf>) -> Self {
        let data = data.into();
        Self {
            config: data.join("config"),
            cache: data.join("cache"),
            logs: data.join("logs"),
            plugins: data.join("plugins"),
            reports: data.join("reports"),
            data,
            workspace: workspace.into(),
        }
    }

    /// Create all application-owned directories. Re-running this is safe.
    pub fn initialize(&self) -> io::Result<RuntimeInitialization> {
        let mut created_directories = Vec::new();
        for directory in [
            &self.data,
            &self.config,
            &self.cache,
            &self.logs,
            &self.plugins,
            &self.reports,
            &self.workspace,
        ] {
            let existed = directory.is_dir();
            fs::create_dir_all(directory)?;
            if !existed {
                created_directories.push(directory.clone());
            }
            restrict_directory_permissions(directory)?;
        }
        Ok(RuntimeInitialization {
            created_directories,
        })
    }

    pub fn database(&self) -> PathBuf {
        self.data.join("polyglid.db")
    }
}

fn default_data_directory() -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    if let Some(path) = non_empty_env("LOCALAPPDATA") {
        return Ok(path.join("PolyGlid"));
    }

    #[cfg(target_os = "macos")]
    if let Some(path) = home_directory()?.map(|home| home.join("Library/Application Support")) {
        return Ok(path.join("PolyGlid"));
    }

    home_directory()?
        .map(|home| home.join(".polyglid"))
        .ok_or_else(|| "set POLYGLID_DATA_DIR when no home directory is available".to_string())
}

fn default_workspace_directory() -> Result<PathBuf, String> {
    home_directory()?
        .map(|home| home.join("polyglid-projects"))
        .ok_or_else(|| {
            "set POLYGLID_WORKSPACE_ROOT when no home directory is available".to_string()
        })
}

fn non_empty_env(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
}

fn home_directory() -> Result<Option<PathBuf>, String> {
    Ok(non_empty_env("HOME").or_else(|| non_empty_env("USERPROFILE")))
}

#[cfg(unix)]
fn restrict_directory_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
fn restrict_directory_permissions(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_all_runtime_directories_from_two_roots() {
        let paths = RuntimePaths::from_roots("/tmp/polyglid-data", "/tmp/polyglid-projects");
        assert_eq!(paths.config, PathBuf::from("/tmp/polyglid-data/config"));
        assert_eq!(paths.cache, PathBuf::from("/tmp/polyglid-data/cache"));
        assert_eq!(paths.logs, PathBuf::from("/tmp/polyglid-data/logs"));
        assert_eq!(paths.plugins, PathBuf::from("/tmp/polyglid-data/plugins"));
        assert_eq!(paths.reports, PathBuf::from("/tmp/polyglid-data/reports"));
        assert_eq!(paths.workspace, PathBuf::from("/tmp/polyglid-projects"));
        assert_eq!(
            paths.database(),
            PathBuf::from("/tmp/polyglid-data/polyglid.db")
        );
    }
}
