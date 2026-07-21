use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DbJobRecord {
    pub job_id: Uuid,
    pub plugin_id: String,
    pub target: String,
    pub state: String,
    pub started_at: u64,
    pub duration_ms: u64,
    pub error_message: Option<String>,
    pub fuel_consumed: u64,
    pub created_at: u64,
}

pub struct ExecutionStore {
    conn: Arc<Mutex<Connection>>,
}

impl ExecutionStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn insert_job(
        &self,
        job_id: &Uuid,
        plugin_id: &str,
        target: &str,
        state: &str,
        started_at: u64,
    ) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        conn.execute(
            "INSERT OR REPLACE INTO execution_history (
                job_id, plugin_id, target, state, started_at, duration_ms, error_message, fuel_consumed, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                job_id.to_string(),
                plugin_id,
                target,
                state,
                started_at as i64,
                0i64,
                None::<String>,
                0i64,
                now
            ],
        )
        .map_err(|err| format!("failed to insert execution job: {err}"))?;

        Ok(())
    }

    pub fn update_job(
        &self,
        job_id: &Uuid,
        state: &str,
        duration_ms: u64,
        fuel_consumed: u64,
        error_message: Option<&str>,
    ) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE execution_history SET state = ?1, duration_ms = ?2, fuel_consumed = ?3, error_message = ?4 WHERE job_id = ?5",
            params![
                state,
                duration_ms as i64,
                fuel_consumed as i64,
                error_message,
                job_id.to_string()
            ],
        )
        .map_err(|err| format!("failed to update execution job: {err}"))?;

        Ok(())
    }

    pub fn list(&self) -> Result<Vec<DbJobRecord>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT job_id, plugin_id, target, state, started_at, duration_ms, error_message, fuel_consumed, created_at 
                 FROM execution_history ORDER BY started_at DESC"
            )
            .map_err(|err| format!("failed to prepare statement: {err}"))?;

        let rows = stmt
            .query_map([], |row| {
                let job_id_str: String = row.get(0)?;
                let job_id = Uuid::parse_str(&job_id_str)
                    .map_err(|_| rusqlite::Error::ExecuteReturnedResults)?;
                Ok(DbJobRecord {
                    job_id,
                    plugin_id: row.get(1)?,
                    target: row.get(2)?,
                    state: row.get(3)?,
                    started_at: row.get::<_, i64>(4)? as u64,
                    duration_ms: row.get::<_, i64>(5)? as u64,
                    error_message: row.get(6)?,
                    fuel_consumed: row.get::<_, i64>(7)? as u64,
                    created_at: row.get::<_, i64>(8)? as u64,
                })
            })
            .map_err(|err| format!("failed to query map jobs: {err}"))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|err| format!("failed to read execution row: {err}"))?);
        }
        Ok(list)
    }
}
