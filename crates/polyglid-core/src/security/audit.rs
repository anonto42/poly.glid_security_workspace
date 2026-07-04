use std::sync::{Arc, Mutex};
use rusqlite::{params, Connection};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub struct AuditLogger {
    conn: Arc<Mutex<Connection>>,
}

impl AuditLogger {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn log(
        &self,
        event_type: &str,
        plugin_id: Option<&str>,
        details_json: serde_json::Value
    ) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let log_id = Uuid::new_v4().to_string();
        let details_str = serde_json::to_string(&details_json)
            .unwrap_or_else(|_| "{}".to_string());

        conn.execute(
            "INSERT INTO audit_logs (id, event_type, plugin_id, details, timestamp) VALUES (?, ?, ?, ?, ?)",
            params![log_id, event_type, plugin_id, details_str, now],
        )
        .map_err(|err| format!("failed to insert audit log: {err}"))?;

        Ok(())
    }
}
