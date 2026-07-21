use crate::PermissionDecision;
use polyglid_plugin_api::{Capability, PluginId};
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct PermissionEngine {
    conn: Arc<Mutex<Connection>>,
}

impl PermissionEngine {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn record_decision(
        &self,
        plugin_id: &PluginId,
        capability: &Capability,
        scope: &str,
        workspace: &str,
        decision: PermissionDecision,
        expiration_seconds: Option<u64>,
    ) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let expiration = expiration_seconds.map(|secs| now + secs);
        let id = format!(
            "{}_{}_{}_{}",
            plugin_id.as_str(),
            capability,
            scope,
            workspace
        );

        let decision_str = match decision {
            PermissionDecision::Allow => "Allow",
            PermissionDecision::Deny { .. } => "Deny",
        };

        conn.execute(
            "INSERT OR REPLACE INTO permissions (id, plugin_id, capability, scope, workspace, decision, timestamp, expiration)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![id, plugin_id.as_str(), capability.to_string(), scope, workspace, decision_str, now, expiration],
        )
        .map_err(|err| format!("failed to save permission: {err}"))?;

        Ok(())
    }

    pub fn evaluate(
        &self,
        plugin_id: &PluginId,
        capability: &Capability,
        _scope: &str,
        workspace: &str,
    ) -> Result<Option<PermissionDecision>, String> {
        let conn = self.conn.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut stmt = conn
            .prepare("SELECT decision, expiration FROM permissions WHERE plugin_id = ? AND capability = ? AND workspace = ?")
            .map_err(|err| format!("failed to prepare query: {err}"))?;

        let mut rows = stmt
            .query(params![
                plugin_id.as_str(),
                capability.to_string(),
                workspace
            ])
            .map_err(|err| format!("query failed: {err}"))?;

        if let Some(row) = rows
            .next()
            .map_err(|err| format!("row retrieve failed: {err}"))?
        {
            let decision_str: String = row.get(0).unwrap();
            let expiration: Option<u64> = row.get(1).unwrap();

            // Check if permission decision is expired
            if let Some(exp) = expiration {
                if exp < now {
                    // Expired, delete the row and return None
                    drop(rows);
                    drop(stmt);
                    let _ = conn.execute(
                        "DELETE FROM permissions WHERE plugin_id = ? AND capability = ? AND workspace = ?",
                        params![plugin_id.as_str(), capability.to_string(), workspace]
                    );
                    return Ok(None);
                }
            }

            let decision = match decision_str.as_str() {
                "Allow" => PermissionDecision::Allow,
                "Deny" => PermissionDecision::Deny {
                    reason: "Explicitly denied in permission engine".to_string(),
                },
                _ => PermissionDecision::Deny {
                    reason: "Unknown permission state".to_string(),
                },
            };
            Ok(Some(decision))
        } else {
            Ok(None)
        }
    }
}
