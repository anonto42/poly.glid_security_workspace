#[cfg(test)]
mod tests {
    use crate::security::verifier::PluginVerifier;
    use crate::security::publisher::PublisherManager;
    use crate::store::WorkspaceStore;
    use polyglid_plugin_api::{Capability, PluginId};
    use ed25519_dalek::{SigningKey, Signer};
    use std::fs;

    #[test]
    fn test_signature_verification_flow() {
        let temp_dir = std::env::temp_dir();
        let wasm_path = temp_dir.join("test_sig_component.wasm");
        fs::write(&wasm_path, b"\x00asm_dummy_component_bytes").unwrap();

        let mut key_bytes = [0u8; 32];
        key_bytes[0] = 1;
        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();

        let sig = signing_key.sign(b"\x00asm_dummy_component_bytes");

        let pub_key_bytes = verifying_key.to_bytes();
        let sig_bytes = sig.to_bytes();

        let verification = PluginVerifier::verify(&wasm_path, &sig_bytes, &pub_key_bytes);
        assert!(verification.is_ok());

        fs::write(&wasm_path, b"\x00asm_manipulated_component_bytes").unwrap();
        let verification_fail = PluginVerifier::verify(&wasm_path, &sig_bytes, &pub_key_bytes);
        assert!(verification_fail.is_err());

        let _ = fs::remove_file(wasm_path);
    }

    #[test]
    fn test_publisher_fingerprint_generation() {
        let key_hex = "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20";
        let fingerprint = PublisherManager::compute_fingerprint(key_hex).unwrap();
        assert_eq!(fingerprint.len(), 64);
    }

    #[test]
    fn test_permission_engine_expiration() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_permission_engine.db");
        let store = WorkspaceStore::new(&db_path).unwrap();

        let pid = PluginId::new("test.plugin").unwrap();
        let cap = Capability::DnsResolve;

        let engine = store.permission_engine();

        store.plugins().insert(&polyglid_config::plugin_registry::PluginRegistryEntry {
            id: pid.clone(),
            name: "Test Plugin".to_string(),
            version: semver::Version::new(1, 0, 0),
            author: "Author".to_string(),
            description: "Desc".to_string(),
            capabilities: vec![],
            checksum: "sum".to_string(),
            status: polyglid_config::plugin_registry::PluginStatus::Enabled,
            source: polyglid_config::plugin_registry::PluginSource::LocalPath(std::path::PathBuf::from("/tmp")),
            file_size: 100,
            installed_at: 0,
            last_updated: 0,
            path: std::path::PathBuf::from("/tmp"),
        }).unwrap();

        engine.record_decision(
            &pid,
            &cap,
            "",
            "Workspace",
            crate::PermissionDecision::Allow,
            Some(10)
        ).unwrap();

        let decision = engine.evaluate(&pid, &cap, "", "Workspace").unwrap();
        assert!(matches!(decision, Some(crate::PermissionDecision::Allow)));

        let _ = store.conn.lock().unwrap().execute("DELETE FROM permissions", []);
        engine.record_decision(
            &pid,
            &cap,
            "",
            "Workspace",
            crate::PermissionDecision::Allow,
            Some(0)
        ).unwrap();
        
        store.conn.lock().unwrap().execute(
            "UPDATE permissions SET expiration = 100", []
        ).unwrap();

        let expired_decision = engine.evaluate(&pid, &cap, "", "Workspace").unwrap();
        assert!(expired_decision.is_none());

        let _ = fs::remove_file(db_path);
    }

    #[test]
    fn test_structured_audit_logs() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_audit.db");
        let store = WorkspaceStore::new(&db_path).unwrap();

        let logger = store.audit_logger();
        logger.log(
            "PluginInstalled",
            Some("test.plugin"),
            serde_json::json!({ "version": "1.0.0" })
        ).unwrap();

        let conn = store.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT count(*) FROM audit_logs", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 1);

        let _ = fs::remove_file(db_path);
    }
}
