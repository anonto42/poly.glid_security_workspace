//! In-memory and serialized representation of the workspace plugin registry.

use polyglid_plugin_api::{Capability, PluginId};
use semver::Version;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PluginStatus {
    Enabled,
    Disabled,
    Invalid,
    UpdateAvailable,
}

impl std::fmt::Display for PluginStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Self::Enabled => "Enabled",
            Self::Disabled => "Disabled",
            Self::Invalid => "Invalid",
            Self::UpdateAvailable => "UpdateAvailable",
        };
        f.write_str(val)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PluginSource {
    LocalPath(PathBuf),
    Marketplace(String),
    Url(String),
}

impl std::fmt::Display for PluginSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LocalPath(p) => write!(f, "local:{}", p.display()),
            Self::Marketplace(s) => write!(f, "marketplace:{s}"),
            Self::Url(u) => write!(f, "url:{u}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PluginRegistryEntry {
    pub id: PluginId,
    pub name: String,
    pub version: Version,
    pub author: String,
    pub description: String,
    pub capabilities: Vec<Capability>,
    pub checksum: String,
    pub status: PluginStatus,
    pub source: PluginSource,
    pub file_size: u64,
    pub installed_at: u64,
    pub last_updated: u64,
    pub path: PathBuf,
}

pub trait RegistryStorage {
    fn load(&self, path: &Path) -> Result<HashMap<PluginId, PluginRegistryEntry>, String>;
    fn save(
        &self,
        path: &Path,
        registry: &HashMap<PluginId, PluginRegistryEntry>,
    ) -> Result<(), String>;
}

#[derive(Debug, Clone, Copy)]
pub struct JsonRegistryStorage;

impl RegistryStorage for JsonRegistryStorage {
    fn load(&self, path: &Path) -> Result<HashMap<PluginId, PluginRegistryEntry>, String> {
        if !path.exists() {
            return Ok(HashMap::new());
        }
        let data = std::fs::read_to_string(path)
            .map_err(|err| format!("failed to read registry file: {err}"))?;
        let registry: HashMap<PluginId, PluginRegistryEntry> = serde_json::from_str(&data)
            .map_err(|err| format!("failed to parse registry JSON: {err}"))?;
        Ok(registry)
    }

    fn save(
        &self,
        path: &Path,
        registry: &HashMap<PluginId, PluginRegistryEntry>,
    ) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create registry directory: {err}"))?;
        }
        let data = serde_json::to_string_pretty(registry)
            .map_err(|err| format!("failed to serialize registry to JSON: {err}"))?;
        std::fs::write(path, data)
            .map_err(|err| format!("failed to write registry file: {err}"))?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PluginRegistry {
    pub entries: HashMap<PluginId, PluginRegistryEntry>,
}

impl PluginRegistry {
    pub fn new(entries: HashMap<PluginId, PluginRegistryEntry>) -> Self {
        Self { entries }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polyglid_plugin_api::{Capability, PluginId};
    use semver::Version;

    #[test]
    fn test_serialization_deserialization() {
        let entry = PluginRegistryEntry {
            id: PluginId::new("test.plugin").unwrap(),
            name: "Test Plugin".to_string(),
            version: Version::parse("1.2.3").unwrap(),
            author: "Author".to_string(),
            description: "Description".to_string(),
            capabilities: vec![Capability::NetworkConnect],
            checksum: "abcdef123456".to_string(),
            status: PluginStatus::Enabled,
            source: PluginSource::LocalPath(PathBuf::from("/path/to/plugin.wasm")),
            file_size: 1024,
            installed_at: 1000,
            last_updated: 2000,
            path: PathBuf::from("/workspace/plugins/test.plugin.wasm"),
        };

        let serialized = serde_json::to_string(&entry).unwrap();
        let deserialized: PluginRegistryEntry = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.id.as_str(), "test.plugin");
        assert_eq!(deserialized.name, "Test Plugin");
        assert_eq!(deserialized.version, Version::new(1, 2, 3));
        assert_eq!(deserialized.status, PluginStatus::Enabled);
        assert_eq!(deserialized.capabilities, vec![Capability::NetworkConnect]);
    }

    #[test]
    fn test_json_registry_storage() {
        let temp_dir = std::env::temp_dir();
        let registry_path = temp_dir.join(format!(
            "polyglid_test_registry_{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        let storage = JsonRegistryStorage;

        // Load non-existent registry should be empty
        let initial = storage.load(&registry_path).unwrap();
        assert!(initial.is_empty());

        // Save entry
        let id = PluginId::new("my.plugin").unwrap();
        let entry = PluginRegistryEntry {
            id: id.clone(),
            name: "My Plugin".to_string(),
            version: Version::new(0, 1, 0),
            author: "Tester".to_string(),
            description: "desc".to_string(),
            capabilities: vec![],
            checksum: "hash123".to_string(),
            status: PluginStatus::Disabled,
            source: PluginSource::Url("https://example.com/plugin.wasm".to_string()),
            file_size: 500,
            installed_at: 12345,
            last_updated: 12345,
            path: PathBuf::from("/plugins/my.plugin.wasm"),
        };

        let mut map = HashMap::new();
        map.insert(id.clone(), entry.clone());

        storage.save(&registry_path, &map).unwrap();

        // Load again
        let loaded = storage.load(&registry_path).unwrap();
        assert_eq!(loaded.len(), 1);
        let loaded_entry = loaded.get(&id).unwrap();
        assert_eq!(loaded_entry.name, "My Plugin");
        assert_eq!(loaded_entry.status, PluginStatus::Disabled);
        assert_eq!(
            loaded_entry.source,
            PluginSource::Url("https://example.com/plugin.wasm".to_string())
        );

        // Clean up
        let _ = std::fs::remove_file(&registry_path);
    }
}
