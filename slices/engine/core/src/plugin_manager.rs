//! Workspace plugin lifecyle manager, validator, and repository abstraction.

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{PluginRef, PluginRuntime};
use polyglid_config::plugin_registry::{
    JsonRegistryStorage, PluginRegistryEntry, PluginSource, PluginStatus, RegistryStorage,
};
use polyglid_config::AppConfig;
use polyglid_plugin_api::{PluginId, PluginManifest};
use semver::Version;

/// Checks structure, headers, magic bytes, and export footprints of WASM components.
pub struct PluginValidator;

impl PluginValidator {
    /// Inspect and validate a WASM plugin component.
    pub fn validate<R: PluginRuntime>(
        runtime: &R,
        path: &Path,
    ) -> Result<(PluginManifest, polyglid_plugin_api::ApiPluginMetadata), String> {
        // 1. Basic file check
        if !path.exists() {
            return Err("file does not exist".to_string());
        }
        let metadata =
            fs::metadata(path).map_err(|err| format!("failed to read file metadata: {err}"))?;
        if metadata.len() == 0 {
            return Err("file is empty".to_string());
        }

        // 2. WASM magic bytes check
        let mut file = fs::File::open(path)
            .map_err(|err| format!("failed to open file for validation: {err}"))?;
        let mut header = [0; 4];
        use std::io::Read;
        file.read_exact(&mut header)
            .map_err(|err| format!("failed to read file headers: {err}"))?;
        if header != [0x00, 0x61, 0x73, 0x6d] {
            return Err("invalid WebAssembly file header".to_string());
        }

        // 3. Inspect component structure and WIT declarations
        let plugin_ref = PluginRef::from_path(path);
        let manifest = runtime
            .inspect(&plugin_ref)
            .map_err(|err| format!("WASM component validation failed: {err}"))?;

        // 4. Query metadata export to ensure required exports exist
        // Note: WasmRuntime in polyglid-runtime has call_metadata method, but since PluginRuntime trait
        // does not declare call_metadata (it is WasmRuntime specific), we can try to downcast or
        // since we are generic over R: PluginRuntime, we can call a custom inspection method or let the manager
        // handle metadata queries. To keep it clean, let's query metadata if R implements or exposes it.
        // Wait, how can a generic PluginRuntime call WasmRuntime specific methods?
        // We can inspect the WASM component's metadata by reading its component bindings.
        // Actually, in our current workspace, polyglid-runtime's WasmRuntime implements call_metadata.
        // We can define a trait method on PluginRuntime, or since PluginRuntime is owned by polyglid-core,
        // we can just add `inspect_metadata` to the `PluginRuntime` trait!
        // Yes! Adding a default method `inspect_metadata` to the `PluginRuntime` trait is extremely clean
        // and preserves trait decoupling!
        // Let's check: WasmRuntime has `call_metadata` which does this. We can map `inspect_metadata` in the trait
        // to call it!
        // Let's add:
        // `fn inspect_metadata(&self, plugin: &PluginRef) -> Result<polyglid_plugin_api::ApiPluginMetadata, CoreError>`
        // to the `PluginRuntime` trait!
        let api_metadata = runtime
            .inspect_metadata(&plugin_ref)
            .map_err(|err| format!("failed to query plugin metadata export: {err}"))?;

        // Ensure plugin ID matches metadata name
        let metadata_id = PluginId::new(&api_metadata.name)
            .map_err(|err| format!("invalid metadata name: {err}"))?;
        if metadata_id != manifest.id {
            return Err(format!(
                "plugin ID mismatch: manifest expects '{}', metadata returns '{}'",
                manifest.id.as_str(),
                metadata_id.as_str()
            ));
        }

        Ok((manifest, api_metadata))
    }
}

/// Handles copying, deleting, and discovering files inside workspace/plugins/
pub struct PluginRepository;

impl PluginRepository {
    pub fn install(
        &self,
        id: &PluginId,
        src_path: &Path,
        plugin_dir: &Path,
    ) -> Result<PathBuf, String> {
        fs::create_dir_all(plugin_dir)
            .map_err(|err| format!("failed to create plugin directory: {err}"))?;

        let dest_path = plugin_dir.join(format!("{}.wasm", id.as_str()));
        fs::copy(src_path, &dest_path)
            .map_err(|err| format!("failed to copy plugin to workspace: {err}"))?;

        Ok(dest_path)
    }

    pub fn remove(&self, id: &PluginId, plugin_dir: &Path) -> Result<(), String> {
        let path = plugin_dir.join(format!("{}.wasm", id.as_str()));
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|err| format!("failed to delete plugin file from workspace: {err}"))?;
        }
        Ok(())
    }

    pub fn discover(&self, plugin_dir: &Path) -> Result<Vec<PathBuf>, String> {
        if !plugin_dir.exists() {
            return Ok(Vec::new());
        }
        let mut files = Vec::new();
        let entries = fs::read_dir(plugin_dir)
            .map_err(|err| format!("failed to read plugin directory: {err}"))?;
        for entry in entries {
            let entry = entry.map_err(|err| format!("failed to read entry: {err}"))?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "wasm") {
                files.push(path);
            }
        }
        Ok(files)
    }
}

/// Orchestrator for validations, installations, and active registry configurations.
pub struct PluginManager<R> {
    pub runtime: Arc<R>,
    repository: PluginRepository,
    pub store: crate::store::WorkspaceStore,
    plugin_dir: PathBuf,
}

impl<R> PluginManager<R>
where
    R: PluginRuntime,
{
    pub fn new(
        runtime: Arc<R>,
        config: &AppConfig,
        store: crate::store::WorkspaceStore,
    ) -> Result<Self, String> {
        let pm = Self {
            runtime,
            repository: PluginRepository,
            store,
            plugin_dir: config.plugin_dir.clone(),
        };

        // Automatic migration path: registry.json -> SQLite
        let registry_json_path = config.registry_path();
        if registry_json_path.exists() {
            let json_storage = JsonRegistryStorage;
            if let Ok(entries) = json_storage.load(&registry_json_path) {
                // Bulk insert inside a transaction
                pm.store.transaction(|tx| {
                    let plugin_store = pm.store.plugins();
                    for entry in entries.values() {
                        plugin_store.insert_with_conn(tx, entry)?;
                    }
                    Ok(())
                })?;
                // Rename registry.json to registry.json.bak
                let backup_path = registry_json_path.with_extension("json.bak");
                let _ = fs::rename(&registry_json_path, &backup_path);
            }
        }

        Ok(pm)
    }

    pub fn get_plugins(&self) -> Vec<PluginRegistryEntry> {
        self.store.plugins().list().unwrap_or_default()
    }

    pub fn get_plugin(&self, id: &PluginId) -> Option<PluginRegistryEntry> {
        self.store.plugins().get(id).unwrap_or(None)
    }

    pub fn is_enabled(&self, id: &PluginId) -> bool {
        if let Some(entry) = self.get_plugin(id) {
            entry.status == PluginStatus::Enabled
        } else {
            false
        }
    }

    pub fn verify_plugin_signature(
        &self,
        src_path: &std::path::Path,
        plugin_id: &PluginId,
    ) -> Result<(crate::security::SignatureStatus, Option<crate::store::signature_store::PluginSignatureRecord>), String> {
        use crate::security::SignatureStatus;
        use std::fs;

        let sig_path = src_path.with_extension("sig");
        let sig_data_opt = if sig_path.exists() {
            let data = fs::read_to_string(&sig_path)
                .map_err(|err| format!("failed to read signature file: {err}"))?;
            let sig_json: serde_json::Value = serde_json::from_str(&data)
                .map_err(|err| format!("invalid signature JSON: {err}"))?;
            
            let algorithm = sig_json["algorithm"].as_str().unwrap_or("Ed25519").to_string();
            let key_id = sig_json["key_id"].as_str().unwrap_or("").to_string();
            let signature = sig_json["signature"].as_str().unwrap_or("").to_string();
            
            Some((algorithm, key_id, signature))
        } else {
            None
        };

        let (algorithm, key_id, signature) = match sig_data_opt {
            Some(data) => data,
            None => return Ok((SignatureStatus::Missing, None)),
        };

        let fingerprint = crate::security::publisher::PublisherManager::compute_fingerprint(&key_id)?;

        let sig_bytes = hex::decode(&signature)
            .map_err(|_| "invalid signature hex encoding".to_string())?;
        let pub_key_bytes = hex::decode(&key_id)
            .map_err(|_| "invalid key_id hex encoding".to_string())?;

        let is_valid = crate::security::verifier::PluginVerifier::verify(src_path, &sig_bytes, &pub_key_bytes).is_ok();
        if !is_valid {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            return Ok((SignatureStatus::Invalid, Some(crate::store::signature_store::PluginSignatureRecord {
                plugin_id: plugin_id.as_str().to_string(),
                algorithm,
                key_id,
                signature,
                fingerprint,
                verified_at: now,
                status: "Invalid".to_string(),
            })));
        }

        let trust_store = self.store.trust_store();
        let status = match trust_store.get_publisher_by_fingerprint(&fingerprint)? {
            Some(pub_rec) => {
                if pub_rec.revocation_status == 1 {
                    SignatureStatus::Revoked
                } else {
                    SignatureStatus::Verified
                }
            }
            None => SignatureStatus::UnknownPublisher,
        };

        let status_str = status.to_string();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok((status, Some(crate::store::signature_store::PluginSignatureRecord {
            plugin_id: plugin_id.as_str().to_string(),
            algorithm,
            key_id,
            signature,
            fingerprint,
            verified_at: now,
            status: status_str,
        })))
    }

    pub fn validate_plugin(
        &self,
        src_path: &Path,
    ) -> Result<(PluginManifest, polyglid_plugin_api::ApiPluginMetadata), String> {
        PluginValidator::validate(self.runtime.as_ref(), src_path)
    }

    pub fn install_plugin(
        &self,
        src_path: &Path,
        source: PluginSource,
    ) -> Result<PluginRegistryEntry, String> {
        let (manifest, metadata) = self.validate_plugin(src_path)?;

        let file_size = fs::metadata(src_path)
            .map_err(|err| format!("failed to read source size: {err}"))?
            .len();
        let checksum = compute_sha256(src_path)?;

        let (sig_status, sig_record_opt) = self.verify_plugin_signature(src_path, &manifest.id)?;

        let active_profile_name = self.store.settings()
            .get("security_profile")
            .unwrap_or(None)
            .unwrap_or_else(|| "Balanced".to_string());

        let profile = match active_profile_name.as_str() {
            "Strict" => crate::security::profiles::SecurityProfile::strict(),
            "Development" => crate::security::profiles::SecurityProfile::development(),
            _ => crate::security::profiles::SecurityProfile::balanced(),
        };

        if profile.require_signature && (sig_status == crate::security::SignatureStatus::Missing || sig_status == crate::security::SignatureStatus::Invalid) {
            let audit_details = serde_json::json!({
                "wasm_path": src_path.display().to_string(),
                "status": sig_status.to_string(),
                "reason": "Signature required by security profile policies."
            });
            let _ = self.store.audit_logger().log("SignatureRejected", Some(manifest.id.as_str()), audit_details);
            return Err(format!("signature check failed: plugin signature is {}", sig_status));
        }

        if profile.require_trusted_publisher && sig_status == crate::security::SignatureStatus::UnknownPublisher {
            let audit_details = serde_json::json!({
                "wasm_path": src_path.display().to_string(),
                "status": sig_status.to_string(),
                "reason": "Trusted publisher required by security profile policies."
            });
            let _ = self.store.audit_logger().log("SignatureRejected", Some(manifest.id.as_str()), audit_details);
            return Err("signature check failed: publisher is untrusted".to_string());
        }

        let new_ver = Version::parse(&metadata.version)
            .map_err(|err| format!("invalid semantic version '{}': {err}", metadata.version))?;
        if let Some(existing) = self.get_plugin(&manifest.id) {
            if existing.version >= new_ver {
                return Err(format!(
                    "version conflict: workspace already contains equal or newer version '{}' for plugin '{}'",
                    existing.version, manifest.id.as_str()
                ));
            }
        }

        let entry = self.store.transaction(|tx| {
            let plugin_store = self.store.plugins();

            let dest_path = self.repository.install(&manifest.id, src_path, &self.plugin_dir)?;

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let capabilities = manifest
                .requested_capabilities
                .iter()
                .map(|req| req.capability)
                .collect();

            let entry = PluginRegistryEntry {
                id: manifest.id.clone(),
                name: metadata.display_name.clone(),
                version: new_ver,
                author: metadata.author.clone(),
                description: metadata.description.clone(),
                capabilities,
                checksum,
                status: PluginStatus::Enabled,
                source,
                file_size,
                installed_at: now,
                last_updated: now,
                path: dest_path,
            };

            plugin_store.insert_with_conn(tx, &entry)?;

            if let Some(mut sig_rec) = sig_record_opt {
                sig_rec.plugin_id = manifest.id.as_str().to_string();
                let sig_store = self.store.signatures();
                sig_store.insert_with_conn(tx, &sig_rec)?;
            }

            Ok(entry)
        })?;
        let audit_details = serde_json::json!({
            "version": metadata.version,
            "author": metadata.author,
            "signature_status": sig_status.to_string()
        });
        let _ = self.store.audit_logger().log("PluginInstalled", Some(manifest.id.as_str()), audit_details);
        Ok(entry)
    }

    pub fn uninstall_plugin(&self, id: &PluginId) -> Result<(), String> {
        // 1. Remove file
        self.repository.remove(id, &self.plugin_dir)?;

        // 2. Remove registry entry
        self.store.plugins().remove(id)?;

        Ok(())
    }

    pub fn toggle_plugin_enabled(&self, id: &PluginId, enabled: bool) -> Result<(), String> {
        self.store.plugins().toggle_enabled(id, enabled)
    }

    pub fn sync_directory(&self) -> Result<(), String> {
        let discovered_paths = self.repository.discover(&self.plugin_dir)?;
        let mut discovered_ids = HashMap::new();

        for path in discovered_paths {
            if let Ok((manifest, metadata)) =
                PluginValidator::validate(self.runtime.as_ref(), &path)
            {
                discovered_ids.insert(manifest.id.clone(), (path, manifest, metadata));
            }
        }

        // Clean up registry entries whose files no longer exist on disk
        let entries = self.get_plugins();
        for entry in entries {
            if !entry.path.exists() {
                self.store.plugins().remove(&entry.id)?;
            }
        }

        // Register any discovered wasm files not currently in registry
        for (id, (path, manifest, metadata)) in discovered_ids {
            if self.get_plugin(&id).is_none() {
                let file_size = fs::metadata(&path).map_or(0, |m| m.len());
                let checksum = compute_sha256(&path).unwrap_or_default();
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let capabilities = manifest
                    .requested_capabilities
                    .iter()
                    .map(|req| req.capability)
                    .collect();

                let entry = PluginRegistryEntry {
                    id: id.clone(),
                    name: metadata.display_name,
                    version: Version::parse(&metadata.version)
                        .unwrap_or_else(|_| Version::new(0, 1, 0)),
                    author: metadata.author,
                    description: metadata.description,
                    capabilities,
                    checksum,
                    status: PluginStatus::Enabled,
                    source: PluginSource::LocalPath(path.clone()),
                    file_size,
                    installed_at: now,
                    last_updated: now,
                    path,
                };
                self.store.plugins().insert(&entry)?;
            }
        }

        Ok(())
    }
}

fn compute_sha256(path: &Path) -> Result<String, String> {
    use std::io::Read;
    let mut file =
        fs::File::open(path).map_err(|err| format!("failed to open file for hashing: {err}"))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 4096];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|err| format!("failed to read file for hashing: {err}"))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let result = hasher.finalize();
    Ok(result.iter().map(|b| format!("{:02x}", b)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PluginManifest;
    use crate::store::WorkspaceStore;
    use polyglid_plugin_api::{ApiPluginMetadata, PluginId};

    struct TestManagerRuntime;

    impl PluginRuntime for TestManagerRuntime {
        fn inspect(&self, _plugin: &PluginRef) -> Result<PluginManifest, crate::CoreError> {
            Ok(PluginManifest {
                id: PluginId::new("manager.test").unwrap(),
                name: "Manager Test Plugin".to_string(),
                version: "0.2.0".to_string(),
                requested_capabilities: vec![],
            })
        }

        fn inspect_metadata(
            &self,
            _plugin: &PluginRef,
        ) -> Result<ApiPluginMetadata, crate::CoreError> {
            Ok(ApiPluginMetadata {
                name: "manager.test".to_string(),
                display_name: "Manager Test Plugin".to_string(),
                version: "0.2.0".to_string(),
                description: "for manager testing".to_string(),
                author: "manager tester".to_string(),
            })
        }

        fn execute(
            &self,
            _request: &crate::PluginRunRequest,
            _config: &AppConfig,
        ) -> Result<crate::PluginReport, crate::CoreError> {
            Err(crate::CoreError::Runtime("cancelled".to_string()))
        }
    }

    #[test]
    fn test_validator_invalid_magic() {
        let temp_dir = std::env::temp_dir();
        let dummy_path = temp_dir.join(format!(
            "invalid_magic_{}.wasm",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::write(&dummy_path, b"not_wasm_header_stuff").unwrap();

        let runtime = TestManagerRuntime;
        let res = PluginValidator::validate(&runtime, &dummy_path);
        assert!(res.is_err());
        assert!(res
            .err()
            .unwrap()
            .contains("invalid WebAssembly file header"));

        let _ = fs::remove_file(&dummy_path);
    }

    #[test]
    fn test_validator_valid_magic_calls_runtime() {
        let temp_dir = std::env::temp_dir();
        let dummy_path = temp_dir.join(format!(
            "valid_magic_{}.wasm",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::write(&dummy_path, b"\x00asm_more_data").unwrap();

        let runtime = TestManagerRuntime;
        let res = PluginValidator::validate(&runtime, &dummy_path);
        assert!(res.is_ok());
        let (manifest, metadata) = res.unwrap();
        assert_eq!(manifest.id.as_str(), "manager.test");
        assert_eq!(metadata.name, "manager.test");

        let _ = fs::remove_file(&dummy_path);
    }

    #[test]
    fn test_repository_lifecycle() {
        let temp_dir = std::env::temp_dir();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let src_path = temp_dir.join(format!("src_plugin_{timestamp}.wasm"));
        let dest_dir = temp_dir.join(format!("dest_dir_{timestamp}"));

        fs::write(&src_path, b"\x00asmdummy").unwrap();

        let repo = PluginRepository;
        let id = PluginId::new("repo.test").unwrap();

        // 1. Install
        let installed_path = repo.install(&id, &src_path, &dest_dir).unwrap();
        assert!(installed_path.exists());
        assert_eq!(installed_path.file_name().unwrap(), "repo.test.wasm");

        // 2. Discover
        let discovered = repo.discover(&dest_dir).unwrap();
        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0], installed_path);

        // 3. Remove
        repo.remove(&id, &dest_dir).unwrap();
        assert!(!installed_path.exists());

        // Clean up
        let _ = fs::remove_file(&src_path);
        let _ = fs::remove_dir_all(&dest_dir);
    }

    #[test]
    fn test_plugin_manager_lifecycle() {
        let temp_dir = std::env::temp_dir();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let workspace_dir = temp_dir.join(format!("workspace_{timestamp}"));
        let plugin_dir = workspace_dir.join("plugins");
        let config_dir = workspace_dir.join("config");
        fs::create_dir_all(&plugin_dir).unwrap();
        fs::create_dir_all(&config_dir).unwrap();

        let mut app_config = AppConfig::development();
        app_config.plugin_dir = plugin_dir.clone();

        // 1. Create a dummy source WASM plugin file with valid header
        let src_path = workspace_dir.join("test_src.wasm");
        fs::write(&src_path, b"\x00asm_dummy_data_goes_here").unwrap();

        let db_path = workspace_dir.join("polyglid.db");
        let store = WorkspaceStore::new(&db_path).unwrap();
        let runtime = Arc::new(TestManagerRuntime);
        let manager = PluginManager::new(runtime, &app_config, store).unwrap();

        // Check initially empty
        assert!(manager.get_plugins().is_empty());
        let test_id = PluginId::new("manager.test").unwrap();
        assert!(!manager.is_enabled(&test_id));

        // 2. Install plugin
        let entry = manager
            .install_plugin(&src_path, PluginSource::LocalPath(src_path.clone()))
            .unwrap();
        assert_eq!(entry.id, test_id);
        assert_eq!(entry.name, "Manager Test Plugin");
        assert_eq!(entry.version, semver::Version::new(0, 2, 0));
        assert!(entry.path.exists());
        assert_eq!(entry.status, PluginStatus::Enabled);

        // Check manager registry holds it
        assert_eq!(manager.get_plugins().len(), 1);
        assert!(manager.is_enabled(&test_id));
        let plugin_entry = manager.get_plugin(&test_id).unwrap();
        assert_eq!(plugin_entry.author, "manager tester");

        // 3. Toggle disable
        manager.toggle_plugin_enabled(&test_id, false).unwrap();
        assert!(!manager.is_enabled(&test_id));
        assert_eq!(
            manager.get_plugin(&test_id).unwrap().status,
            PluginStatus::Disabled
        );

        // 4. Directory Sync - remove file and sync should clean registry
        fs::remove_file(&entry.path).unwrap();
        manager.sync_directory().unwrap();
        assert!(manager.get_plugins().is_empty());

        // 5. Uninstall plugin (when file is already removed/does not exist)
        // Re-install first
        fs::write(&src_path, b"\x00asm_dummy_data_goes_here").unwrap();
        manager
            .install_plugin(&src_path, PluginSource::LocalPath(src_path.clone()))
            .unwrap();
        assert_eq!(manager.get_plugins().len(), 1);
        manager.uninstall_plugin(&test_id).unwrap();
        assert!(manager.get_plugins().is_empty());

        // Clean up
        let _ = fs::remove_dir_all(&workspace_dir);
    }
}
