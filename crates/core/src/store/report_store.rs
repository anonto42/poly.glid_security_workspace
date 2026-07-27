use polyglid_plugin_api::{Issue, PluginId, PluginReport};
use rusqlite::{params, Connection, OptionalExtension};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DbReportRecord {
    pub id: String,
    pub job_id: Uuid,
    pub plugin_id: String,
    pub plugin_name: String,
    pub project_id: Option<String>,
    pub plugin_version: String,
    pub plugin_checksum: String,
    pub target: String,
    pub summary: String,
    pub issues: Vec<Issue>,
    pub filepath: String,
    pub duration_ms: u64,
    pub fuel_consumed: Option<u64>,
    pub memory_used: Option<u64>,
    pub security_profile: String,
    pub report_format_version: String,
    pub created_at: u64,
}

pub struct ReportStore {
    conn: Arc<Mutex<Connection>>,
}

impl ReportStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Persist a report and the complete execution metadata required for export.
    #[allow(clippy::too_many_arguments)]
    pub fn insert(
        &self,
        report_id: &str,
        job_id: &Uuid,
        plugin_id: &PluginId,
        target: &str,
        report: &PluginReport,
        filepath: &str,
        project_id: Option<&str>,
        plugin_version: &str,
        plugin_checksum: &str,
        duration_ms: u64,
        fuel_consumed: Option<u64>,
        memory_used: Option<u64>,
        security_profile: &str,
    ) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let issues_json = serde_json::to_string(&report.issues)
            .map_err(|err| format!("failed to serialize issues: {err}"))?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        conn.execute(
            "INSERT OR REPLACE INTO reports (
                id, job_id, plugin_id, plugin_name, target, summary, issues, filepath, created_at,
                project_id, plugin_version, plugin_checksum, duration_ms, fuel_consumed,
                memory_used, security_profile, report_format_version
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, '1.0')",
            params![
                report_id,
                job_id.to_string(),
                plugin_id.as_str(),
                report.plugin_name,
                target,
                report.summary,
                issues_json,
                filepath,
                now,
                project_id,
                plugin_version,
                plugin_checksum,
                duration_ms as i64,
                fuel_consumed.map(|value| value as i64),
                memory_used.map(|value| value as i64),
                security_profile,
            ],
        )
        .map_err(|err| format!("failed to insert report reference: {err}"))?;

        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<Option<DbReportRecord>, String> {
        let conn = self.conn.lock().unwrap();
        let row = conn
            .query_row(
                "SELECT id, job_id, plugin_id, plugin_name, target, summary, issues, filepath, created_at,
                        project_id, plugin_version, plugin_checksum, duration_ms, fuel_consumed,
                        memory_used, security_profile, report_format_version
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
                        row.get::<_, String>(7)?,
                        row.get::<_, i64>(8)?,
                        row.get::<_, Option<String>>(9)?,
                        row.get::<_, String>(10)?,
                        row.get::<_, String>(11)?,
                        row.get::<_, i64>(12)?,
                        row.get::<_, Option<i64>>(13)?,
                        row.get::<_, Option<i64>>(14)?,
                        row.get::<_, String>(15)?,
                        row.get::<_, String>(16)?,
                    ))
                },
            )
            .optional()
            .map_err(|err| format!("failed to query report reference: {err}"))?;

        match row {
            Some((
                id,
                job_id_str,
                plugin_id_str,
                plugin_name,
                target,
                summary,
                issues_json,
                filepath,
                created_at,
                project_id,
                plugin_version,
                plugin_checksum,
                duration_ms,
                fuel_consumed,
                memory_used,
                security_profile,
                report_format_version,
            )) => {
                let job_id = Uuid::parse_str(&job_id_str)
                    .map_err(|err| format!("invalid job UUID in DB: {err}"))?;
                let issues: Vec<Issue> = serde_json::from_str(&issues_json)
                    .map_err(|err| format!("invalid issues JSON in DB: {err}"))?;

                Ok(Some(DbReportRecord {
                    id,
                    job_id,
                    plugin_id: plugin_id_str,
                    plugin_name,
                    project_id,
                    plugin_version,
                    plugin_checksum,
                    target,
                    summary,
                    issues,
                    filepath,
                    duration_ms: duration_ms.max(0) as u64,
                    fuel_consumed: fuel_consumed.map(|value| value.max(0) as u64),
                    memory_used: memory_used.map(|value| value.max(0) as u64),
                    security_profile,
                    report_format_version,
                    created_at: created_at as u64,
                }))
            }
            None => Ok(None),
        }
    }

    pub fn list(&self) -> Result<Vec<DbReportRecord>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, job_id, plugin_id, plugin_name, target, summary, issues, filepath, created_at,
                        project_id, plugin_version, plugin_checksum, duration_ms, fuel_consumed,
                        memory_used, security_profile, report_format_version
                 FROM reports ORDER BY created_at DESC",
            )
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
                    row.get::<_, String>(7)?,
                    row.get::<_, i64>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    row.get::<_, String>(10)?,
                    row.get::<_, String>(11)?,
                    row.get::<_, i64>(12)?,
                    row.get::<_, Option<i64>>(13)?,
                    row.get::<_, Option<i64>>(14)?,
                    row.get::<_, String>(15)?,
                    row.get::<_, String>(16)?,
                ))
            })
            .map_err(|err| format!("failed to query reports: {err}"))?;

        let mut list = Vec::new();
        for r in rows {
            let (
                id,
                job_id_str,
                plugin_id_str,
                plugin_name,
                target,
                summary,
                issues_json,
                filepath,
                created_at,
                project_id,
                plugin_version,
                plugin_checksum,
                duration_ms,
                fuel_consumed,
                memory_used,
                security_profile,
                report_format_version,
            ) = r.map_err(|err| format!("failed to read row: {err}"))?;
            let job_id = Uuid::parse_str(&job_id_str)
                .map_err(|err| format!("invalid job UUID in DB: {err}"))?;
            let issues: Vec<Issue> = serde_json::from_str(&issues_json)
                .map_err(|err| format!("invalid issues JSON in DB: {err}"))?;

            list.push(DbReportRecord {
                id,
                job_id,
                plugin_id: plugin_id_str,
                plugin_name,
                project_id,
                plugin_version,
                plugin_checksum,
                target,
                summary,
                issues,
                filepath,
                duration_ms: duration_ms.max(0) as u64,
                fuel_consumed: fuel_consumed.map(|value| value.max(0) as u64),
                memory_used: memory_used.map(|value| value.max(0) as u64),
                security_profile,
                report_format_version,
                created_at: created_at as u64,
            });
        }
        Ok(list)
    }
}
