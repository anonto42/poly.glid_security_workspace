use crate::store::permission_store::{ApprovalBinding, ApprovalDuration, DbPermissionStore};
use crate::PermissionDecision;
use polyglid_plugin_api::{Capability, CapabilityRequest, CapabilityScope, PluginId};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Compatibility facade for the frozen CLI.
///
/// Product clients should use `DbPermissionStore` with explicit approval IDs
/// and complete bindings. This facade still records schema-compatible,
/// workspace-scoped decisions instead of maintaining a second SQL format.
pub struct PermissionEngine {
    store: DbPermissionStore,
}

impl PermissionEngine {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self {
            store: DbPermissionStore::new(conn),
        }
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
        let request = CapabilityRequest::new(*capability, parse_legacy_scope(scope)?);
        let binding = ApprovalBinding {
            workspace_id: workspace.to_string(),
            project_id: None,
            plugin_version: String::new(),
            plugin_checksum: String::new(),
            target: String::new(),
        };
        let expiration = expiration_seconds.map(|seconds| now_secs() + seconds);
        self.store
            .record_decision(
                plugin_id,
                &request,
                &binding,
                decision,
                ApprovalDuration::Workspace,
                None,
                expiration,
            )
            .map(|_| ())
    }

    pub fn evaluate(
        &self,
        plugin_id: &PluginId,
        capability: &Capability,
        scope: &str,
        workspace: &str,
    ) -> Result<Option<PermissionDecision>, String> {
        let request = CapabilityRequest::new(*capability, parse_legacy_scope(scope)?);
        let now = now_secs();
        Ok(self.store.list()?.into_iter().find_map(|record| {
            (record.plugin_id == *plugin_id
                && record.request == request
                && record.binding.workspace_id == workspace
                && record.binding.project_id.is_none()
                && record.revoked_at.is_none()
                && record.expiration.is_none_or(|expiration| expiration > now))
            .then_some(record.decision)
        }))
    }
}

fn parse_legacy_scope(scope: &str) -> Result<CapabilityScope, String> {
    if scope.trim().is_empty() {
        return Ok(CapabilityScope::Any);
    }
    serde_json::from_str(scope).map_err(|err| format!("invalid capability scope: {err}"))
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
