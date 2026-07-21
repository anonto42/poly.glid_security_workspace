#[cfg(test)]
mod benches {
    use crate::execution::reports::{ExportedReport, ReportMetadata};
    use crate::store::WorkspaceStore;
    use polyglid_config::plugin_registry::{PluginRegistryEntry, PluginSource, PluginStatus};
    use polyglid_plugin_api::{Capability, Issue, PluginId, PluginReport, Severity};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{Instant, SystemTime, UNIX_EPOCH};

    #[test]
    fn run_real_workload_benchmarks() {
        println!("\n=== PolyGlid Real Workload Performance Benchmarks ===");
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("polyglid_benchmark.db");
        if db_path.exists() {
            let _ = fs::remove_file(&db_path);
        }

        // 1. Benchmark workspace startup & migrations
        let start_time = Instant::now();
        let store = WorkspaceStore::new(&db_path).unwrap();
        let startup_duration = start_time.elapsed();
        println!("- Workspace startup and migrations: {:?}", startup_duration);
        assert!(
            startup_duration.as_secs_f64() < 1.0,
            "Startup target < 1.0s failed"
        );

        // 2. Load workspace with 500 mock execution logs
        let start_write = Instant::now();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Insert a single mock plugin to satisfy the foreign key reference constraint
        let pid_ref = PluginId::new("plugin.mock.0").unwrap();
        let mock_plugin = PluginRegistryEntry {
            id: pid_ref,
            name: "Mock Reference Plugin".to_string(),
            version: semver::Version::new(1, 0, 0),
            author: "Author".to_string(),
            description: "Description".to_string(),
            capabilities: vec![],
            checksum: "checksum".to_string(),
            status: PluginStatus::Enabled,
            source: PluginSource::LocalPath(PathBuf::from("/tmp")),
            file_size: 1024,
            installed_at: now,
            last_updated: now,
            path: PathBuf::from("/tmp"),
        };
        store.plugins().insert(&mock_plugin).unwrap();

        store.transaction(|tx| {
            for _ in 0..500 {
                let job_id = uuid::Uuid::new_v4();
                tx.execute(
                    "INSERT INTO execution_history (job_id, plugin_id, target, state, started_at, duration_ms, error_message, fuel_consumed, created_at)
                     VALUES (?, 'plugin.mock.0', ?, 'Completed', ?, 120, NULL, 50000, ?)",
                    rusqlite::params![
                        job_id.to_string(),
                        "example.com",
                        now,
                        now
                    ]
                ).unwrap();
            }
            Ok(())
        }).unwrap();
        let write_duration = start_write.elapsed();
        let per_query_latency = write_duration / 500;
        println!("- 500 mock execution logs written: {:?}", write_duration);
        println!("- Average SQLite query latency: {:?}", per_query_latency);
        assert!(
            per_query_latency.as_millis() < 10,
            "SQLite query latency target < 10ms failed"
        );

        // 3. Install 10 mock plugins concurrently
        let start_install = Instant::now();
        for i in 0..10 {
            let pid = PluginId::new(format!("plugin.mock.{i}")).unwrap();
            let entry = PluginRegistryEntry {
                id: pid,
                name: format!("Mock Plugin {i}"),
                version: semver::Version::new(1, 0, 0),
                author: "Author".to_string(),
                description: "Description".to_string(),
                capabilities: vec![Capability::DnsResolve],
                checksum: "checksum".to_string(),
                status: PluginStatus::Enabled,
                source: PluginSource::LocalPath(PathBuf::from("/tmp")),
                file_size: 1024,
                installed_at: now,
                last_updated: now,
                path: PathBuf::from("/tmp"),
            };
            store.plugins().insert(&entry).unwrap();
        }
        let install_duration = start_install.elapsed();
        println!(
            "- 10 mock plugins discovery/installation: {:?}",
            install_duration
        );
        assert!(
            install_duration.as_millis() < 100,
            "Plugin discovery target < 100ms failed"
        );

        // 4. Export a large report and measure serialization latency
        let mut issues = Vec::new();
        for i in 0..1000 {
            issues.push(Issue {
                title: format!("Observation {i}"),
                severity: Severity::High,
                description: "This is a detailed observation description for testing report export size performance.".to_string(),
                recommendation: "Review configuration and ensure permissions are configured safely.".to_string(),
            });
        }
        let report = ExportedReport {
            metadata: ReportMetadata {
                polyglid_version: "0.9.0".to_string(),
                plugin_id: "plugin.mock.0".to_string(),
                plugin_version: "1.0.0".to_string(),
                target: "example.com".to_string(),
                timestamp: now,
                security_profile: "Balanced".to_string(),
                execution_duration_ms: 120,
                report_format_version: "1.0".to_string(),
            },
            report: PluginReport {
                plugin_name: "Mock Scanner".to_string(),
                target_tested: "example.com".to_string(),
                issues,
                summary: "Benchmark execution completed with 1000 issues.".to_string(),
            },
        };

        let start_export = Instant::now();
        let json_report = crate::execution::reports::json::export(&report).unwrap();
        let export_duration = start_export.elapsed();
        println!("- Large JSON export (1000 issues): {:?}", export_duration);
        assert!(!json_report.is_empty());

        let start_sarif = Instant::now();
        let sarif_report = crate::execution::reports::sarif::export(&report).unwrap();
        let sarif_duration = start_sarif.elapsed();
        println!("- Large SARIF export (1000 issues): {:?}", sarif_duration);
        assert!(!sarif_report.is_empty());

        let _ = fs::remove_file(&db_path);
    }
}
