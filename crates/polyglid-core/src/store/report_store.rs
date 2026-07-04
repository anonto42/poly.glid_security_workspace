use std::sync::{Arc, Mutex};
use rusqlite::{params, Connection, OptionalExtension};
use polyglid_plugin_api::{Issue, PluginId};
use uuid::Uuid;

pub struct DbReportRecord {
    pub id: String,
    pub job_id: Uuid,
    pub plugin_id: String,
    pub target: String,
    pub summary: String,
    pub issues: Vec<Issue>,
    pub filepath: String,
    pub created_at: u64,
}

pub struct ReportStore {
    conn: Arc<Mutex<Connection>>,
}

impl ReportStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn insert(
        &self,
        report_id: &str,
        job_id: &Uuid,
        plugin_id: &PluginId,
        target: &str,
        summary: &str,
        issues: &[Issue],
        filepath: &str,
    ) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let issues_json = serde_json::to_string(issues)
            .map_err(|err| format!("failed to serialize issues: {err}"))?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        conn.execute(
            "INSERT OR REPLACE INTO reports (id, job_id, plugin_id, target, summary, issues, filepath, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                report_id,
                job_id.to_string(),
                plugin_id.as_str(),
                target,
                summary,
                issues_json,
                filepath,
                now
            ],
        )
        .map_err(|err| format!("failed to insert report reference: {err}"))?;

        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<Option<DbReportRecord>, String> {
        let conn = self.conn.lock().unwrap();
        let row = conn.query_row(
            "SELECT id, job_id, plugin_id, target, summary, issues, filepath, created_at 
             FROM reports WHERE id = ?",
            [id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, i64>(7)?,
                ))
            },
        )
        .optional()
        .map_err(|err| format!("failed to query report reference: {err}"))?;

        match row {
            Some((id, job_id_str, plugin_id_str, target, summary, issues_json, filepath, created_at)) => {
                let job_id = Uuid::parse_str(&job_id_str)
                    .map_err(|err| format!("invalid job UUID in DB: {err}"))?;
                let issues: Vec<Issue> = serde_json::from_str(&issues_json)
                    .map_err(|err| format!("invalid issues JSON in DB: {err}"))?;

                Ok(Some(DbReportRecord {
                    id,
                    job_id,
                    plugin_id: plugin_id_str,
                    target,
                    summary,
                    issues,
                    filepath,
                    created_at: created_at as u64,
                }))
            }
            None => Ok(None),
        }
    }

    pub fn list(&self) -> Result<Vec<DbReportRecord>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, job_id, plugin_id, target, summary, issues, filepath, created_at FROM reports ORDER BY created_at DESC")
            .map_err(|err| format!("failed to prepare statement: {err}"))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, i64>(7)?,
                ))
            })
            .map_err(|err| format!("failed to query reports: {err}"))?;

        let mut list = Vec::new();
        for r in rows {
            let (id, job_id_str, plugin_id_str, target, summary, issues_json, filepath, created_at) =
                r.map_err(|err| format!("failed to read row: {err}"))?;
            let job_id = Uuid::parse_str(&job_id_str)
                .map_err(|err| format!("invalid job UUID in DB: {err}"))?;
            let issues: Vec<Issue> = serde_json::from_str(&issues_json)
                .map_err(|err| format!("invalid issues JSON in DB: {err}"))?;

            list.push(DbReportRecord {
                id,
                job_id,
                plugin_id: plugin_id_str,
                target,
                summary,
                issues,
                filepath,
                created_at: created_at as u64,
            });
        }
        Ok(list)
    }
}
