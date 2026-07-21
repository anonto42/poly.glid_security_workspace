use rusqlite::{params, Connection, OptionalExtension};
use std::sync::{Arc, Mutex};

pub struct SettingsStore {
    conn: Arc<Mutex<Connection>>,
}

impl SettingsStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn set(&self, key: &str, value: &str, scope: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value, scope, created_at, updated_at) 
             VALUES (?1, ?2, ?3, COALESCE((SELECT created_at FROM settings WHERE key = ?1), ?4), ?4)",
            params![key, value, scope, now],
        )
        .map_err(|err| format!("failed to set setting: {err}"))?;

        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<String>, String> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT value FROM settings WHERE key = ?", [key], |row| {
            row.get::<_, String>(0)
        })
        .optional()
        .map_err(|err| format!("failed to get setting: {err}"))
    }

    pub fn list(&self) -> Result<Vec<(String, String, String)>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT key, value, scope FROM settings ORDER BY key")
            .map_err(|err| format!("failed to prepare settings query: {err}"))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|err| format!("failed to map settings query: {err}"))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|err| format!("failed to read settings row: {err}"))?);
        }
        Ok(list)
    }
}
