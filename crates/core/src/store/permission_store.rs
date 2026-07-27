use crate::{CoreError, PermissionDecision, PermissionStore};
use polyglid_plugin_api::{Capability, CapabilityRequest, CapabilityScope, PluginId};
use rusqlite::{params, Connection, OptionalExtension};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalDuration {
    Once,
    Session,
    Workspace,
}

impl ApprovalDuration {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Once => "once",
            Self::Session => "session",
            Self::Workspace => "workspace",
        }
    }

    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "once" => Ok(Self::Once),
            "session" => Ok(Self::Session),
            "workspace" => Ok(Self::Workspace),
            _ => Err(format!("invalid approval duration '{value}'")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalBinding {
    pub workspace_id: String,
    pub project_id: Option<String>,
    pub plugin_version: String,
    pub plugin_checksum: String,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbPermissionRecord {
    pub id: String,
    pub plugin_id: PluginId,
    pub request: CapabilityRequest,
    pub binding: ApprovalBinding,
    pub decision: PermissionDecision,
    pub duration: ApprovalDuration,
    pub session_id: Option<String>,
    pub timestamp: u64,
    pub expiration: Option<u64>,
    pub revoked_at: Option<u64>,
}

pub struct DbPermissionStore {
    conn: Arc<Mutex<Connection>>,
}

impl DbPermissionStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Persist one exact decision together with every binding used at execution.
    #[allow(clippy::too_many_arguments)]
    pub fn record_decision(
        &self,
        plugin_id: &PluginId,
        request: &CapabilityRequest,
        binding: &ApprovalBinding,
        decision: PermissionDecision,
        duration: ApprovalDuration,
        session_id: Option<&str>,
        expiration: Option<u64>,
    ) -> Result<DbPermissionRecord, String> {
        let id = Uuid::new_v4().to_string();
        let timestamp = now_secs();
        let scope = serde_json::to_string(&request.scope)
            .map_err(|err| format!("failed to serialize approval scope: {err}"))?;
        let decision_value = match &decision {
            PermissionDecision::Allow => "Allow",
            PermissionDecision::Deny { .. } => "Deny",
        };
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO permissions (
                id, plugin_id, capability, scope, workspace, decision, timestamp, expiration,
                project_id, plugin_version, plugin_checksum, target, duration, session_id, revoked_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, NULL)",
            params![
                id,
                plugin_id.as_str(),
                request.capability.as_str(),
                scope,
                binding.workspace_id,
                decision_value,
                timestamp as i64,
                expiration.map(|value| value as i64),
                binding.project_id,
                binding.plugin_version,
                binding.plugin_checksum,
                binding.target,
                duration.as_str(),
                session_id,
            ],
        )
        .map_err(|err| format!("failed to save permission decision: {err}"))?;
        drop(conn);

        Ok(DbPermissionRecord {
            id,
            plugin_id: plugin_id.clone(),
            request: request.clone(),
            binding: binding.clone(),
            decision,
            duration,
            session_id: session_id.map(str::to_string),
            timestamp,
            expiration,
            revoked_at: None,
        })
    }

    pub fn revoke(&self, approval_id: &str) -> Result<bool, String> {
        let conn = self.conn.lock().unwrap();
        let changed = conn
            .execute(
                "UPDATE permissions SET revoked_at = ?1
                 WHERE id = ?2 AND revoked_at IS NULL",
                params![now_secs() as i64, approval_id],
            )
            .map_err(|err| format!("failed to revoke permission decision: {err}"))?;
        Ok(changed > 0)
    }

    pub fn list(&self) -> Result<Vec<DbPermissionRecord>, String> {
        let conn = self.conn.lock().unwrap();
        let mut statement = conn
            .prepare(
                "SELECT id, plugin_id, capability, scope, workspace, decision, timestamp,
                        expiration, project_id, plugin_version, plugin_checksum, target,
                        duration, session_id, revoked_at
                 FROM permissions ORDER BY timestamp DESC, id",
            )
            .map_err(|err| format!("failed to prepare permission query: {err}"))?;
        let rows = statement
            .query_map([], read_record)
            .map_err(|err| format!("failed to query permission decisions: {err}"))?;
        rows.map(|row| {
            row.map_err(|err| format!("failed to read permission decision: {err}"))
                .and_then(parse_record)
        })
        .collect()
    }

    /// Validate exact context-bound approvals and atomically consume one-time
    /// decisions. The returned requests are safe to pass to the runtime host.
    pub fn validate_and_consume(
        &self,
        approval_ids: &[String],
        plugin_id: &PluginId,
        binding: &ApprovalBinding,
        requested: &[CapabilityRequest],
        current_session_id: &str,
    ) -> Result<Vec<CapabilityRequest>, String> {
        let mut conn = self.conn.lock().unwrap();
        let transaction = conn
            .transaction()
            .map_err(|err| format!("failed to start approval validation: {err}"))?;
        let mut approvals = Vec::with_capacity(approval_ids.len());

        for approval_id in approval_ids {
            let raw = transaction
                .query_row(
                    "SELECT id, plugin_id, capability, scope, workspace, decision, timestamp,
                            expiration, project_id, plugin_version, plugin_checksum, target,
                            duration, session_id, revoked_at
                     FROM permissions WHERE id = ?1",
                    [approval_id],
                    read_record,
                )
                .optional()
                .map_err(|err| format!("failed to load approval '{approval_id}': {err}"))?
                .ok_or_else(|| format!("approval '{approval_id}' was not found"))?;
            approvals.push(parse_record(raw)?);
        }

        let now = now_secs();
        for approval in &approvals {
            if approval.plugin_id != *plugin_id {
                return Err(format!(
                    "approval '{}' belongs to a different plugin",
                    approval.id
                ));
            }
            if approval.binding != *binding {
                return Err(format!(
                    "approval '{}' does not match this plugin version, checksum, target, workspace, or project",
                    approval.id
                ));
            }
            if approval.revoked_at.is_some() {
                return Err(format!("approval '{}' was revoked", approval.id));
            }
            if approval
                .expiration
                .is_some_and(|expiration| expiration <= now)
            {
                return Err(format!("approval '{}' has expired", approval.id));
            }
            if !matches!(approval.decision, PermissionDecision::Allow) {
                return Err(format!("approval '{}' records a denial", approval.id));
            }
            if approval.duration == ApprovalDuration::Session
                && approval.session_id.as_deref() != Some(current_session_id)
            {
                return Err(format!(
                    "approval '{}' belongs to a different application session",
                    approval.id
                ));
            }
            if !requested.contains(&approval.request) {
                return Err(format!(
                    "approval '{}' does not match a requested capability scope",
                    approval.id
                ));
            }
        }

        for request in requested {
            let count = approvals
                .iter()
                .filter(|approval| approval.request == *request)
                .count();
            if count != 1 {
                return Err(format!(
                    "capability request '{request}' requires exactly one valid approval"
                ));
            }
        }
        if approvals.len() != requested.len() {
            return Err("the approval set contains duplicates or unexpected decisions".to_string());
        }

        for approval in approvals
            .iter()
            .filter(|approval| approval.duration == ApprovalDuration::Once)
        {
            transaction
                .execute(
                    "UPDATE permissions SET revoked_at = ?1 WHERE id = ?2",
                    params![now as i64, approval.id],
                )
                .map_err(|err| format!("failed to consume approval '{}': {err}", approval.id))?;
        }
        transaction
            .commit()
            .map_err(|err| format!("failed to commit approval validation: {err}"))?;

        Ok(requested.to_vec())
    }
}

// A generic PermissionStore call has no workspace/project/version/checksum
// context, so it must never use durable approvals implicitly.
impl PermissionStore for DbPermissionStore {
    fn decide(
        &self,
        _plugin_id: &PluginId,
        _request: &CapabilityRequest,
    ) -> Result<PermissionDecision, CoreError> {
        Ok(PermissionDecision::Deny {
            reason: "a context-bound approval id is required".to_string(),
        })
    }
}

type RawPermissionRecord = (
    String,
    String,
    String,
    String,
    String,
    String,
    i64,
    Option<i64>,
    Option<String>,
    String,
    String,
    String,
    String,
    Option<String>,
    Option<i64>,
);

fn read_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<RawPermissionRecord> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        row.get(5)?,
        row.get(6)?,
        row.get(7)?,
        row.get(8)?,
        row.get(9)?,
        row.get(10)?,
        row.get(11)?,
        row.get(12)?,
        row.get(13)?,
        row.get(14)?,
    ))
}

fn parse_record(raw: RawPermissionRecord) -> Result<DbPermissionRecord, String> {
    let (
        id,
        plugin_id,
        capability,
        scope,
        workspace_id,
        decision,
        timestamp,
        expiration,
        project_id,
        plugin_version,
        plugin_checksum,
        target,
        duration,
        session_id,
        revoked_at,
    ) = raw;
    let request = CapabilityRequest {
        capability: Capability::from_str(&capability)
            .map_err(|err| format!("invalid stored capability: {err}"))?,
        scope: serde_json::from_str::<CapabilityScope>(&scope)
            .map_err(|err| format!("invalid stored capability scope: {err}"))?,
    };
    Ok(DbPermissionRecord {
        id,
        plugin_id: PluginId::new(plugin_id)
            .map_err(|err| format!("invalid stored plugin id: {err}"))?,
        request,
        binding: ApprovalBinding {
            workspace_id,
            project_id,
            plugin_version,
            plugin_checksum,
            target,
        },
        decision: match decision.as_str() {
            "Allow" => PermissionDecision::Allow,
            "Deny" => PermissionDecision::Deny {
                reason: "explicitly denied".to_string(),
            },
            _ => return Err(format!("invalid stored permission decision '{decision}'")),
        },
        duration: ApprovalDuration::parse(&duration)?,
        session_id,
        timestamp: timestamp.max(0) as u64,
        expiration: expiration.map(|value| value.max(0) as u64),
        revoked_at: revoked_at.map(|value| value.max(0) as u64),
    })
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::WorkspaceStore;
    use polyglid_config::plugin_registry::{PluginRegistryEntry, PluginSource, PluginStatus};
    use semver::Version;
    use std::path::PathBuf;

    fn test_store() -> (WorkspaceStore, PluginId, ApprovalBinding) {
        let store = WorkspaceStore::new(std::path::Path::new(":memory:")).expect("workspace");
        let plugin_id = PluginId::new("polyglid.permission_test").expect("plugin id");
        store
            .plugins()
            .insert(&PluginRegistryEntry {
                id: plugin_id.clone(),
                name: "Permission Test".to_string(),
                version: Version::new(1, 2, 3),
                author: "PolyGlid".to_string(),
                description: "permission fixture".to_string(),
                capabilities: vec![Capability::DnsResolve],
                checksum: "sha256-fixture".to_string(),
                status: PluginStatus::Enabled,
                source: PluginSource::LocalPath(PathBuf::from("fixture.wasm")),
                file_size: 1,
                installed_at: 1,
                last_updated: 1,
                path: PathBuf::from("fixture.wasm"),
            })
            .expect("insert plugin");
        let binding = ApprovalBinding {
            workspace_id: "workspace-1".to_string(),
            project_id: Some("project-1".to_string()),
            plugin_version: "1.2.3".to_string(),
            plugin_checksum: "sha256-fixture".to_string(),
            target: "example.com".to_string(),
        };
        (store, plugin_id, binding)
    }

    #[test]
    fn exact_once_approval_is_consumed_atomically() {
        let (store, plugin_id, binding) = test_store();
        assert_eq!(store.user_version().expect("schema version"), 7);
        let permissions = store.permissions();
        let request = CapabilityRequest::new(
            Capability::DnsResolve,
            CapabilityScope::Target("example.com".to_string()),
        );
        let approval = permissions
            .record_decision(
                &plugin_id,
                &request,
                &binding,
                PermissionDecision::Allow,
                ApprovalDuration::Once,
                Some("session-1"),
                Some(now_secs() + 60),
            )
            .expect("record approval");

        assert_eq!(
            permissions
                .validate_and_consume(
                    std::slice::from_ref(&approval.id),
                    &plugin_id,
                    &binding,
                    std::slice::from_ref(&request),
                    "session-1",
                )
                .expect("approval accepted"),
            vec![request.clone()]
        );
        assert!(permissions
            .validate_and_consume(
                &[approval.id],
                &plugin_id,
                &binding,
                &[request],
                "session-1",
            )
            .expect_err("approval cannot be reused")
            .contains("revoked"));
    }

    #[test]
    fn approval_rejects_context_mismatch_and_expiration() {
        let (store, plugin_id, binding) = test_store();
        let permissions = store.permissions();
        let request = CapabilityRequest::unscoped(Capability::DnsResolve);
        let approval = permissions
            .record_decision(
                &plugin_id,
                &request,
                &binding,
                PermissionDecision::Allow,
                ApprovalDuration::Workspace,
                None,
                Some(now_secs().saturating_sub(1)),
            )
            .expect("record expired approval");

        assert!(permissions
            .validate_and_consume(
                std::slice::from_ref(&approval.id),
                &plugin_id,
                &binding,
                std::slice::from_ref(&request),
                "session-1",
            )
            .expect_err("expired approval rejected")
            .contains("expired"));

        let active = permissions
            .record_decision(
                &plugin_id,
                &request,
                &binding,
                PermissionDecision::Allow,
                ApprovalDuration::Workspace,
                None,
                None,
            )
            .expect("record active approval");
        let mut wrong_binding = binding.clone();
        wrong_binding.plugin_checksum = "different".to_string();
        assert!(permissions
            .validate_and_consume(
                &[active.id],
                &plugin_id,
                &wrong_binding,
                &[request],
                "session-1",
            )
            .expect_err("binding mismatch rejected")
            .contains("does not match"));
    }
}
