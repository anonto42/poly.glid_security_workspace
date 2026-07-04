use std::path::Path;
use std::sync::Arc;
use polyglid_plugin_api::PluginId;
use polyglid_config::plugin_registry::{PluginRegistryEntry, PluginStatus, PluginSource};
use crate::store::WorkspaceStore;
use crate::execution::ExecutionManager;
use crate::plugin_manager::PluginManager;
use crate::PluginRuntime;

pub struct PluginService<R> {
    pm: Arc<PluginManager<R>>,
}

impl<R: PluginRuntime + Send + Sync + 'static> PluginService<R> {
    pub fn new(pm: Arc<PluginManager<R>>) -> Self {
        Self { pm }
    }

    pub fn list_plugins(&self) -> Result<Vec<PluginRegistryEntry>, String> {
        Ok(self.pm.get_plugins())
    }

    pub fn get_plugin(&self, id: &PluginId) -> Result<Option<PluginRegistryEntry>, String> {
        Ok(self.pm.get_plugin(id))
    }

    pub fn install_plugin(&self, src_path: &Path) -> Result<PluginRegistryEntry, String> {
        self.pm.install_plugin(src_path, PluginSource::LocalPath(src_path.to_path_buf()))
    }

    pub fn uninstall_plugin(&self, id: &PluginId) -> Result<(), String> {
        self.pm.uninstall_plugin(id)
    }

    pub fn toggle_plugin(&self, id: &PluginId, enabled: bool) -> Result<(), String> {
        self.pm.toggle_plugin_enabled(id, enabled)
    }
}

pub struct ExecutionService<R> {
    em: Arc<ExecutionManager<R>>,
    store: WorkspaceStore,
}

impl<R: PluginRuntime + Send + Sync + 'static> ExecutionService<R> {
    pub fn new(em: Arc<ExecutionManager<R>>, store: WorkspaceStore) -> Self {
        Self { em, store }
    }

    pub fn run_plugin(&self, plugin_id: &PluginId, target: &str) -> Result<String, String> {
        let pm = PluginManager::new(self.em.runtime().clone(), &polyglid_config::AppConfig::development(), self.store.clone())?;
        let entry = pm.get_plugin(plugin_id)
            .ok_or_else(|| format!("plugin '{}' not found in workspace", plugin_id.as_str()))?;

        if entry.status == PluginStatus::Disabled {
            return Err(format!("plugin '{}' is currently disabled", plugin_id.as_str()));
        }

        let config = crate::execution::ExecutionConfig {
            fuel_limit: 25_000_000,
            timeout: std::time::Duration::from_secs(30),
            memory_limit: None,
            allowed_capabilities: entry.capabilities,
        };

        let job_id = self.em.submit_job(entry.path.to_string_lossy().to_string(), target.to_string(), config);
        Ok(job_id.to_string())
    }

    pub fn list_executions(&self) -> Result<Vec<crate::store::execution_store::DbJobRecord>, String> {
        self.store.executions().list()
    }

    pub fn get_execution(&self, job_id: &str) -> Result<Option<crate::store::execution_store::DbJobRecord>, String> {
        let list = self.store.executions().list()?;
        Ok(list.into_iter().find(|r| r.job_id.to_string() == job_id))
    }
}

pub struct TargetService {
    store: WorkspaceStore,
}

impl TargetService {
    pub fn new(store: WorkspaceStore) -> Self {
        Self { store }
    }

    pub fn list_targets(&self) -> Result<Vec<String>, String> {
        let list = self.store.targets().list()?;
        Ok(list.into_iter().map(|(name, _)| name).collect())
    }

    pub fn add_target(&self, name: &str) -> Result<(), String> {
        self.store.targets().add(name, None)
    }

    pub fn remove_target(&self, name: &str) -> Result<(), String> {
        self.store.targets().remove(name)
    }
}

pub struct ReportService {
    store: WorkspaceStore,
}

impl ReportService {
    pub fn new(store: WorkspaceStore) -> Self {
        Self { store }
    }

    pub fn list_reports(&self) -> Result<Vec<crate::store::report_store::DbReportRecord>, String> {
        self.store.reports().list()
    }

    pub fn get_report(&self, id: &str) -> Result<Option<crate::store::report_store::DbReportRecord>, String> {
        let list = self.store.reports().list()?;
        Ok(list.into_iter().find(|r| r.id == id))
    }

    pub fn export_report(&self, id: &str, format: &str) -> Result<String, String> {
        let report_rec = self.get_report(id)?
            .ok_or_else(|| format!("report '{}' not found", id))?;

        let issues = report_rec.issues.clone();

        let payload = crate::execution::reports::ExportedReport {
            metadata: crate::execution::reports::ReportMetadata {
                polyglid_version: "0.9.0".to_string(),
                plugin_id: report_rec.plugin_id,
                plugin_version: "0.1.0".to_string(),
                target: report_rec.target.clone(),
                timestamp: report_rec.created_at,
                security_profile: "Balanced".to_string(),
                execution_duration_ms: 120,
                report_format_version: "1.0".to_string(),
            },
            report: polyglid_plugin_api::PluginReport {
                plugin_name: "Plugin Report".to_string(),
                target_tested: report_rec.target.clone(),
                issues,
                summary: report_rec.summary,
            }
        };

        match format.to_lowercase().as_str() {
            "json" => crate::execution::reports::json::export(&payload),
            "markdown" | "md" => crate::execution::reports::markdown::export(&payload),
            "html" => crate::execution::reports::html::export(&payload),
            "sarif" => crate::execution::reports::sarif::export(&payload),
            _ => Err(format!("unknown export format '{}'", format)),
        }
    }
}

pub struct SettingsService {
    store: WorkspaceStore,
}

impl SettingsService {
    pub fn new(store: WorkspaceStore) -> Self {
        Self { store }
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, String> {
        self.store.settings().get(key)
    }

    pub fn set_setting(&self, key: &str, val: &str) -> Result<(), String> {
        self.store.settings().set(key, val, "Workspace")
    }
}
