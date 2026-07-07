use std::sync::{Arc, Mutex};
use rusqlite::{params, Connection};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginSignatureRecord {
    pub plugin_id: String,
    pub algorithm: String,
    pub key_id: String,
    pub signature: String,
    pub fingerprint: String,
    pub verified_at: u64,
    pub status: String,
}

pub struct PluginSignatureStore {
    conn: Arc<Mutex<Connection>>,
}

impl PluginSignatureStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn insert(&self, record: &PluginSignatureRecord) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO plugin_signatures (plugin_id, algorithm, key_id, signature, fingerprint, verified_at, status)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                record.plugin_id,
                record.algorithm,
                record.key_id,
                record.signature,
                record.fingerprint,
                record.verified_at,
                record.status
            ],
        )
        .map_err(|err| format!("failed to insert signature: {err}"))?;
        Ok(())
    }

    pub fn insert_with_conn(&self, conn: &Connection, record: &PluginSignatureRecord) -> Result<(), String> {
        conn.execute(
            "INSERT OR REPLACE INTO plugin_signatures (plugin_id, algorithm, key_id, signature, fingerprint, verified_at, status)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                record.plugin_id,
                record.algorithm,
                record.key_id,
                record.signature,
                record.fingerprint,
                record.verified_at,
                record.status
            ],
        )
        .map_err(|err| format!("failed to insert signature: {err}"))?;
        Ok(())
    }

    pub fn get(&self, plugin_id: &str) -> Result<Option<PluginSignatureRecord>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT plugin_id, algorithm, key_id, signature, fingerprint, verified_at, status FROM plugin_signatures WHERE plugin_id = ?")
            .map_err(|err| format!("failed to prepare statement: {err}"))?;

        let mut rows = stmt
            .query(params![plugin_id])
            .map_err(|err| format!("query failed: {err}"))?;

        if let Some(row) = rows.next().map_err(|err| format!("row retrieve failed: {err}"))? {
            Ok(Some(PluginSignatureRecord {
                plugin_id: row.get(0).unwrap(),
                algorithm: row.get(1).unwrap(),
                key_id: row.get(2).unwrap(),
                signature: row.get(3).unwrap(),
                fingerprint: row.get(4).unwrap(),
                verified_at: row.get(5).unwrap(),
                status: row.get(6).unwrap(),
            }))
        } else {
            Ok(None)
        }
    }
}
