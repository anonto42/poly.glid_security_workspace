use crate::execution::ExecutionManager;
use crate::plugin_manager::PluginManager;
use crate::store::WorkspaceStore;
use crate::PluginRuntime;
use polyglid_config::plugin_registry::{PluginRegistryEntry, PluginSource, PluginStatus};
use polyglid_plugin_api::{CapabilityRequest, PluginId};
use std::path::Path;
use std::sync::Arc;

mod project_lifecycle;
mod workspace_catalog;
mod workspace_discovery;
pub use workspace_catalog::WorkspaceCatalogService;

#[cfg(test)]
mod workspace_catalog_tests;

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
        self.pm
            .install_plugin(src_path, PluginSource::LocalPath(src_path.to_path_buf()))
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
    pm: Arc<PluginManager<R>>,
    store: WorkspaceStore,
}

impl<R: PluginRuntime + Send + Sync + 'static> ExecutionService<R> {
    pub fn new(
        em: Arc<ExecutionManager<R>>,
        pm: Arc<PluginManager<R>>,
        store: WorkspaceStore,
    ) -> Self {
        Self { em, pm, store }
    }

    pub fn run_plugin(
        &self,
        plugin_id: &PluginId,
        target: &str,
        approved_capabilities: Vec<CapabilityRequest>,
    ) -> Result<String, String> {
        let entry = self
            .pm
            .get_plugin(plugin_id)
            .ok_or_else(|| format!("plugin '{}' not found in workspace", plugin_id.as_str()))?;

        if entry.status == PluginStatus::Disabled {
            return Err(format!(
                "plugin '{}' is currently disabled",
                plugin_id.as_str()
            ));
        }

        let config = crate::execution::ExecutionConfig {
            fuel_limit: 25_000_000,
            timeout: std::time::Duration::from_secs(30),
            memory_limit: Some(64 * 1024 * 1024),
            allowed_capabilities: approved_capabilities,
            project_id: None,
            approval_ids: Vec::new(),
            plugin_version: entry.version.to_string(),
            plugin_checksum: entry.checksum,
        };

        let job_id = self.em.submit_job(
            entry.path.to_string_lossy().to_string(),
            target.to_string(),
            config,
        );
        Ok(job_id.to_string())
    }

    pub fn list_executions(
        &self,
    ) -> Result<Vec<crate::store::execution_store::DbJobRecord>, String> {
        self.store.executions().list()
    }

    pub fn get_execution(
        &self,
        job_id: &str,
    ) -> Result<Option<crate::store::execution_store::DbJobRecord>, String> {
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
        Ok(list.into_iter().map(|(name, _, _)| name).collect())
    }

    pub fn add_target(&self, name: &str) -> Result<(), String> {
        self.store.targets().add(name, None, None)
    }

    pub fn remove_target(&self, name: &str) -> Result<(), String> {
        self.store.targets().remove(name, None)
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

    pub fn get_report(
        &self,
        id: &str,
    ) -> Result<Option<crate::store::report_store::DbReportRecord>, String> {
        let list = self.store.reports().list()?;
        Ok(list.into_iter().find(|r| r.id == id))
    }

    pub fn export_report(&self, id: &str, format: &str) -> Result<String, String> {
        let report_rec = self
            .get_report(id)?
            .ok_or_else(|| format!("report '{}' not found", id))?;

        let issues = report_rec.issues.clone();

        let payload = crate::execution::reports::ExportedReport {
            metadata: crate::execution::reports::ReportMetadata {
                polyglid_version: env!("CARGO_PKG_VERSION").to_string(),
                plugin_id: report_rec.plugin_id.clone(),
                plugin_version: report_rec.plugin_version,
                target: report_rec.target.clone(),
                timestamp: report_rec.created_at,
                security_profile: report_rec.security_profile,
                execution_duration_ms: report_rec.duration_ms,
                report_format_version: report_rec.report_format_version,
            },
            report: polyglid_plugin_api::PluginReport {
                plugin_name: report_rec.plugin_name,
                target_tested: report_rec.target.clone(),
                issues,
                summary: report_rec.summary,
            },
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

// ─────────────────────────────────────────────────────────────────────────────
// Marketplace Service
// ─────────────────────────────────────────────────────────────────────────────

use crate::store::marketplace_store::{
    DbMarketplaceInstall, DbMarketplacePackage, DbMarketplaceRating, DbPublisherProfile,
};

pub struct MarketplaceService {
    store: WorkspaceStore,
}

impl MarketplaceService {
    pub fn new(store: WorkspaceStore) -> Self {
        Self { store }
    }

    // Publishers

    pub fn list_publishers(&self) -> Result<Vec<DbPublisherProfile>, String> {
        self.store.marketplace().list_publishers()
    }

    pub fn get_publisher(&self, id: &str) -> Result<Option<DbPublisherProfile>, String> {
        self.store.marketplace().get_publisher(id)
    }

    pub fn register_publisher(&self, profile: &DbPublisherProfile) -> Result<(), String> {
        self.store.marketplace().add_publisher(profile)
    }

    // Packages

    pub fn list_featured(&self) -> Result<Vec<DbMarketplacePackage>, String> {
        self.store.marketplace().list_featured()
    }

    pub fn search(
        &self,
        query: &str,
        category: Option<&str>,
    ) -> Result<Vec<DbMarketplacePackage>, String> {
        self.store.marketplace().search_packages(query, category)
    }

    pub fn get_package(&self, id: &str) -> Result<Option<DbMarketplacePackage>, String> {
        self.store.marketplace().get_package(id)
    }

    pub fn publish(&self, pkg: &DbMarketplacePackage) -> Result<(), String> {
        self.store.marketplace().publish_package(pkg)
    }

    /// Record that a marketplace package was installed (called after PluginService installs it).
    pub fn record_package_install(
        &self,
        package_id: &str,
        plugin_id: Option<String>,
    ) -> Result<(), String> {
        let pkg = self
            .store
            .marketplace()
            .get_package(package_id)?
            .ok_or_else(|| format!("marketplace package '{}' not found", package_id))?;

        let install = DbMarketplaceInstall {
            id: uuid_v4(),
            package_id: pkg.id.clone(),
            plugin_id,
            installed_at: now_secs(),
        };
        self.store.marketplace().record_install(&install)?;
        self.store
            .marketplace()
            .increment_download_count(package_id)?;
        Ok(())
    }

    /// Get the download_url of a marketplace package (for the server to pass to PluginService).
    pub fn get_package_download_url(&self, package_id: &str) -> Result<String, String> {
        self.store
            .marketplace()
            .get_package(package_id)?
            .map(|p| p.download_url)
            .ok_or_else(|| format!("marketplace package '{}' not found", package_id))
    }

    // Ratings

    pub fn add_rating(&self, rating: &DbMarketplaceRating) -> Result<(), String> {
        self.store.marketplace().add_rating(rating)
    }

    pub fn list_ratings(&self, package_id: &str) -> Result<Vec<DbMarketplaceRating>, String> {
        self.store.marketplace().list_ratings(package_id)
    }

    // Installed

    pub fn list_installed(&self) -> Result<Vec<DbMarketplaceInstall>, String> {
        self.store.marketplace().list_installed_packages()
    }
}

fn uuid_v4() -> String {
    format!("mkt-{}", uuid::Uuid::new_v4())
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

// ─────────────────────────────────────────────────────────────────────────────
// Collaboration Service
// ─────────────────────────────────────────────────────────────────────────────

use crate::store::collaboration_store::{DbTeam, DbTeamMember, DbUser, DbUserToken};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use sha2::{Digest, Sha256};

pub struct CollaborationService {
    store: WorkspaceStore,
}

impl CollaborationService {
    pub fn new(store: WorkspaceStore) -> Self {
        Self { store }
    }

    fn hash_password(&self, password: &str) -> Result<(String, String), String> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|error| format!("failed to hash password: {error}"))?
            .to_string();
        Ok((hash, salt.to_string()))
    }

    pub fn count_users(&self) -> Result<i64, String> {
        self.store.collaboration().count_users()
    }

    pub fn register_user(
        &self,
        username: &str,
        password: &str,
        role: &str,
    ) -> Result<DbUser, String> {
        if username.trim().is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        if password.len() < 8 {
            return Err("Password must be at least 8 characters long".to_string());
        }
        if role != "Owner" && role != "Editor" && role != "Viewer" {
            return Err("Invalid role, must be Owner, Editor, or Viewer".to_string());
        }

        let store = self.store.collaboration();
        if store.get_user_by_username(username)?.is_some() {
            return Err(format!("User '{}' already exists", username));
        }

        let (password_hash, salt) = self.hash_password(password)?;
        let now = now_secs();
        let user = DbUser {
            id: format!("usr-{}", uuid::Uuid::new_v4()),
            username: username.to_string(),
            password_hash,
            salt,
            role: role.to_string(),
            created_at: now,
            updated_at: now,
        };

        store.create_user(&user)?;
        Ok(user)
    }

    pub fn login_user(&self, username: &str, password: &str) -> Result<(DbUser, String), String> {
        let store = self.store.collaboration();
        let user = store
            .get_user_by_username(username)?
            .ok_or_else(|| "Invalid username or password".to_string())?;

        let argon2_valid = PasswordHash::new(&user.password_hash)
            .ok()
            .is_some_and(|hash| {
                Argon2::default()
                    .verify_password(password.as_bytes(), &hash)
                    .is_ok()
            });
        let legacy_valid = !user.password_hash.starts_with("$argon2")
            && legacy_password_hash(password, &user.salt) == user.password_hash;
        if !argon2_valid && !legacy_valid {
            return Err("Invalid username or password".to_string());
        }
        if legacy_valid {
            let (password_hash, salt) = self.hash_password(password)?;
            store.update_password(&user.id, &password_hash, &salt, now_secs())?;
        }

        let token_str = format!(
            "tok-{}{}",
            uuid::Uuid::new_v4().simple(),
            uuid::Uuid::new_v4().simple()
        );
        let now = now_secs();
        let token = DbUserToken {
            token: token_str.clone(),
            user_id: user.id.clone(),
            created_at: now,
            expires_at: now + 86400, // 24 hours
        };

        store.create_token(&token)?;
        Ok((user, token_str))
    }

    pub fn validate_token(&self, token: &str) -> Result<Option<DbUser>, String> {
        self.store.collaboration().validate_token(token, now_secs())
    }

    pub fn logout_user(&self, token: &str) -> Result<(), String> {
        self.store.collaboration().revoke_token(token)
    }

    pub fn list_users(&self) -> Result<Vec<DbUser>, String> {
        self.store.collaboration().list_users()
    }

    // Teams

    pub fn create_team(&self, name: &str) -> Result<DbTeam, String> {
        if name.trim().is_empty() {
            return Err("Team name cannot be empty".to_string());
        }
        let now = now_secs();
        let team = DbTeam {
            id: format!("team-{}", uuid::Uuid::new_v4()),
            name: name.to_string(),
            created_at: now,
            updated_at: now,
        };
        self.store.collaboration().create_team(&team)?;
        Ok(team)
    }

    pub fn list_teams(&self) -> Result<Vec<DbTeam>, String> {
        self.store.collaboration().list_teams()
    }

    pub fn add_team_member(&self, team_id: &str, user_id: &str, role: &str) -> Result<(), String> {
        if role != "Owner" && role != "Editor" && role != "Viewer" {
            return Err("Invalid role".to_string());
        }
        let member = DbTeamMember {
            team_id: team_id.to_string(),
            user_id: user_id.to_string(),
            role: role.to_string(),
            created_at: now_secs(),
        };
        self.store.collaboration().add_team_member(&member)
    }

    pub fn remove_team_member(&self, team_id: &str, user_id: &str) -> Result<(), String> {
        self.store
            .collaboration()
            .remove_team_member(team_id, user_id)
    }

    pub fn list_team_members(&self, team_id: &str) -> Result<Vec<(DbUser, String)>, String> {
        self.store.collaboration().list_team_members(team_id)
    }
}

fn legacy_password_hash(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod collaboration_service_tests {
    use super::*;

    #[test]
    fn legacy_password_is_upgraded_to_argon2_after_login() {
        let store = WorkspaceStore::new(std::path::Path::new(":memory:")).expect("workspace");
        let salt = "legacy-salt";
        store
            .collaboration()
            .create_user(&DbUser {
                id: "legacy-user".to_string(),
                username: "legacy".to_string(),
                password_hash: legacy_password_hash("password123", salt),
                salt: salt.to_string(),
                role: "Owner".to_string(),
                created_at: 1,
                updated_at: 1,
            })
            .expect("legacy user");

        CollaborationService::new(store.clone())
            .login_user("legacy", "password123")
            .expect("legacy login");
        let upgraded = store
            .collaboration()
            .get_user_by_username("legacy")
            .expect("user query")
            .expect("user");
        assert!(upgraded.password_hash.starts_with("$argon2"));
        assert_ne!(upgraded.salt, salt);
    }
}
