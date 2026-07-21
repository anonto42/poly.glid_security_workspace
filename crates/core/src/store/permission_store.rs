use crate::{CoreError, PermissionDecision, PermissionStore};
use polyglid_plugin_api::{Capability, CapabilityRequest, CapabilityScope, PluginId};
use rusqlite::{params, Connection};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

pub struct DbPermissionStore {
    conn: Arc<Mutex<Connection>>,
}

impl DbPermissionStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn grant(
        &self,
        plugin_id: Option<&PluginId>,
        request: &CapabilityRequest,
    ) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let pid_str = plugin_id.map(|id| id.as_str());
        let cap_str = request.capability.as_str();
        let scope_json = serde_json::to_string(&request.scope)
            .map_err(|err| format!("failed to serialize scope: {err}"))?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        conn.execute(
            "INSERT INTO permissions (plugin_id, capability, scope, created_at) 
             VALUES (?1, ?2, ?3, ?4)",
            params![pid_str, cap_str, scope_json, now],
        )
        .map_err(|err| format!("failed to insert permission grant: {err}"))?;

        Ok(())
    }

    pub fn revoke(
        &self,
        plugin_id: Option<&PluginId>,
        capability: Capability,
    ) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let pid_str = plugin_id.map(|id| id.as_str());
        let cap_str = capability.as_str();

        if let Some(pid) = pid_str {
            conn.execute(
                "DELETE FROM permissions WHERE plugin_id = ?1 AND capability = ?2",
                params![pid, cap_str],
            )
            .map_err(|err| format!("failed to revoke permission: {err}"))?;
        } else {
            conn.execute(
                "DELETE FROM permissions WHERE plugin_id IS NULL AND capability = ?1",
                params![cap_str],
            )
            .map_err(|err| format!("failed to revoke permission: {err}"))?;
        }

        Ok(())
    }

    pub fn list(&self) -> Result<Vec<(Option<PluginId>, CapabilityRequest)>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT plugin_id, capability, scope FROM permissions ORDER BY plugin_id, capability")
            .map_err(|err| format!("failed to prepare query: {err}"))?;

        let rows = stmt
            .query_map([], |row| {
                let pid_str: Option<String> = row.get(0)?;
                let cap_str: String = row.get(1)?;
                let scope_json: Option<String> = row.get(2)?;
                Ok((pid_str, cap_str, scope_json))
            })
            .map_err(|err| format!("failed to map query: {err}"))?;

        let mut list = Vec::new();
        for r in rows {
            let (pid_str, cap_str, scope_json) =
                r.map_err(|err| format!("failed to read row: {err}"))?;
            let plugin_id = pid_str.and_then(|s| PluginId::new(&s).ok());
            let capability = Capability::from_str(&cap_str)
                .map_err(|err| format!("invalid capability in DB: {err}"))?;
            let scope = if let Some(json) = scope_json {
                serde_json::from_str(&json).map_err(|err| format!("invalid scope JSON: {err}"))?
            } else {
                CapabilityScope::Any
            };

            list.push((plugin_id, CapabilityRequest { capability, scope }));
        }

        Ok(list)
    }
}

impl PermissionStore for DbPermissionStore {
    fn decide(
        &self,
        plugin_id: &PluginId,
        request: &CapabilityRequest,
    ) -> Result<PermissionDecision, CoreError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT plugin_id, scope FROM permissions 
                 WHERE (plugin_id = ?1 OR plugin_id IS NULL) AND capability = ?2",
            )
            .map_err(|err| CoreError::PermissionStore(err.to_string()))?;

        let rows = stmt
            .query_map(
                params![plugin_id.as_str(), request.capability.as_str()],
                |row| {
                    let pid_str: Option<String> = row.get(0)?;
                    let scope_json: Option<String> = row.get(1)?;
                    Ok((pid_str, scope_json))
                },
            )
            .map_err(|err| CoreError::PermissionStore(err.to_string()))?;

        for r in rows {
            let (_pid_str, scope_json) =
                r.map_err(|err| CoreError::PermissionStore(err.to_string()))?;
            let scope = if let Some(json) = scope_json {
                serde_json::from_str(&json)
                    .map_err(|err| CoreError::PermissionStore(err.to_string()))?
            } else {
                CapabilityScope::Any
            };

            let grant = CapabilityRequest {
                capability: request.capability,
                scope,
            };

            // Replicate grant_covers checks
            if grant.capability == request.capability
                && (grant.scope == CapabilityScope::Any || grant.scope == request.scope)
            {
                return Ok(PermissionDecision::Allow);
            }
        }

        Ok(PermissionDecision::Deny {
            reason: "capability request is denied by default".to_string(),
        })
    }
}
