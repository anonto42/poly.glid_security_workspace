use std::sync::{Arc, Mutex};
use rusqlite::{params, Connection};

pub struct TargetStore {
    conn: Arc<Mutex<Connection>>,
}

impl TargetStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn add(&self, name: &str, group_name: Option<&str>) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        conn.execute(
            "INSERT OR REPLACE INTO targets (name, group_name, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?3)",
            params![name, group_name, now],
        )
        .map_err(|err| format!("failed to add target: {err}"))?;

        Ok(())
    }

    pub fn remove(&self, name: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM targets WHERE name = ?", [name])
            .map_err(|err| format!("failed to remove target: {err}"))?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<(String, Option<String>)>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT name, group_name FROM targets ORDER BY name")
            .map_err(|err| format!("failed to prepare target query: {err}"))?;

        let rows = stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)))
            .map_err(|err| format!("failed to map target query: {err}"))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|err| format!("failed to read target row: {err}"))?);
        }
        Ok(list)
    }
}
