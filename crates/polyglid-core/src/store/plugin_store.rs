use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use rusqlite::{params, Connection, OptionalExtension};
use polyglid_config::plugin_registry::{PluginRegistryEntry, PluginStatus, PluginSource};
use polyglid_plugin_api::{PluginId, Capability};
use semver::Version;

pub struct PluginStore {
    conn: Arc<Mutex<Connection>>,
}

impl PluginStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn insert(&self, entry: &PluginRegistryEntry) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        self.insert_with_conn(&conn, entry)
    }

    pub fn insert_with_conn(&self, conn: &Connection, entry: &PluginRegistryEntry) -> Result<(), String> {
        let capabilities_json = serde_json::to_string(&entry.capabilities)
            .map_err(|err| format!("failed to serialize capabilities: {err}"))?;
        let source_json = serde_json::to_string(&entry.source)
            .map_err(|err| format!("failed to serialize source: {err}"))?;
        let version_str = entry.version.to_string();
        let status_str = entry.status.to_string();
        let path_str = entry.path.to_string_lossy().to_string();

        conn.execute(
            "INSERT OR REPLACE INTO plugins (
                id, name, version, author, description, capabilities, checksum, status, source, file_size, path, created_at, updated_at, last_used_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                entry.id.as_str(),
                entry.name,
                version_str,
                entry.author,
                entry.description,
                capabilities_json,
                entry.checksum,
                status_str,
                source_json,
                entry.file_size as i64,
                path_str,
                entry.installed_at as i64,
                entry.last_updated as i64,
                None::<i64>
            ],
        ).map_err(|err| format!("failed to insert plugin: {err}"))?;

        Ok(())
    }

    pub fn get(&self, id: &PluginId) -> Result<Option<PluginRegistryEntry>, String> {
        let conn = self.conn.lock().unwrap();
        let row = conn.query_row(
            "SELECT name, version, author, description, capabilities, checksum, status, source, file_size, path, created_at, updated_at FROM plugins WHERE id = ?",
            [id.as_str()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, i64>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, i64>(10)?,
                    row.get::<_, i64>(11)?,
                ))
            },
        ).optional().map_err(|err| format!("failed to query plugin: {err}"))?;

        match row {
            Some((name, version_str, author, description, caps_json, checksum, status_str, source_json, file_size, path_str, created_at, updated_at)) => {
                let version = Version::parse(&version_str)
                    .map_err(|err| format!("invalid semver in DB: {err}"))?;
                let capabilities: Vec<Capability> = serde_json::from_str(&caps_json)
                    .map_err(|err| format!("invalid capabilities JSON in DB: {err}"))?;
                let status = match status_str.as_str() {
                    "Enabled" => PluginStatus::Enabled,
                    "Disabled" => PluginStatus::Disabled,
                    "Invalid" => PluginStatus::Invalid,
                    "UpdateAvailable" => PluginStatus::UpdateAvailable,
                    _ => PluginStatus::Enabled,
                };
                let source: PluginSource = serde_json::from_str(&source_json)
                    .map_err(|err| format!("invalid source JSON in DB: {err}"))?;

                Ok(Some(PluginRegistryEntry {
                    id: id.clone(),
                    name,
                    version,
                    author,
                    description,
                    capabilities,
                    checksum,
                    status,
                    source,
                    file_size: file_size as u64,
                    installed_at: created_at as u64,
                    last_updated: updated_at as u64,
                    path: PathBuf::from(path_str),
                }))
            }
            None => Ok(None)
        }
    }

    pub fn list(&self) -> Result<Vec<PluginRegistryEntry>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, version, author, description, capabilities, checksum, status, source, file_size, path, created_at, updated_at FROM plugins ORDER BY id"
        ).map_err(|err| format!("failed to prepare query: {err}"))?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, i64>(9)?,
                row.get::<_, String>(10)?,
                row.get::<_, i64>(11)?,
                row.get::<_, i64>(12)?,
            ))
        }).map_err(|err| format!("failed to map query: {err}"))?;

        let mut list = Vec::new();
        for row in rows {
            let (id_str, name, version_str, author, description, caps_json, checksum, status_str, source_json, file_size, path_str, created_at, updated_at) = row.map_err(|err| format!("failed to read row: {err}"))?;
            let id = PluginId::new(&id_str).map_err(|err| format!("invalid plugin id in DB: {err}"))?;
            let version = Version::parse(&version_str)
                .map_err(|err| format!("invalid semver in DB: {err}"))?;
            let capabilities: Vec<Capability> = serde_json::from_str(&caps_json)
                .map_err(|err| format!("invalid capabilities JSON in DB: {err}"))?;
            let status = match status_str.as_str() {
                "Enabled" => PluginStatus::Enabled,
                "Disabled" => PluginStatus::Disabled,
                "Invalid" => PluginStatus::Invalid,
                "UpdateAvailable" => PluginStatus::UpdateAvailable,
                _ => PluginStatus::Enabled,
            };
            let source: PluginSource = serde_json::from_str(&source_json)
                .map_err(|err| format!("invalid source JSON in DB: {err}"))?;

            list.push(PluginRegistryEntry {
                id,
                name,
                version,
                author,
                description,
                capabilities,
                checksum,
                status,
                source,
                file_size: file_size as u64,
                installed_at: created_at as u64,
                last_updated: updated_at as u64,
                path: PathBuf::from(path_str),
            });
        }

        Ok(list)
    }

    pub fn remove(&self, id: &PluginId) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM plugins WHERE id = ?", [id.as_str()])
            .map_err(|err| format!("failed to delete plugin from DB: {err}"))?;
        Ok(())
    }

    pub fn toggle_enabled(&self, id: &PluginId, enabled: bool) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let status = if enabled { "Enabled" } else { "Disabled" };
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
        conn.execute(
            "UPDATE plugins SET status = ?, updated_at = ? WHERE id = ?",
            params![status, now, id.as_str()],
        ).map_err(|err| format!("failed to update plugin status: {err}"))?;
        Ok(())
    }
}
