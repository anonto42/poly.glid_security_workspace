//! Asynchronous execution engine and job manager for PolyGlid.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;
use uuid::Uuid;

use polyglid_config::AppConfig;
use polyglid_events::VecEventSink;
use polyglid_plugin_api::{CapabilityRequest, PluginId, PluginReport};

use crate::store::WorkspaceStore;
use crate::{
    CoreEngine, InMemoryPermissionStore, PluginRef, PluginRunRequest, PluginRuntime, Target,
};

pub mod reports;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum JobState {
    Queued,
    Starting,
    Running,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionConfig {
    pub fuel_limit: u64,
    pub timeout: Duration,
    pub memory_limit: Option<u64>,
    /// Exact capability requests approved for this single execution.
    pub allowed_capabilities: Vec<CapabilityRequest>,
    pub project_id: Option<String>,
    pub approval_ids: Vec<String>,
    pub plugin_version: String,
    pub plugin_checksum: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JobMetrics {
    pub duration: Duration,
    pub fuel_consumed: Option<u64>,
    pub memory_used: Option<u64>,
    pub timestamp: u64,
    pub stage: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub plugin_path: String,
    pub target: String,
    pub state: JobState,
    pub config: ExecutionConfig,
    pub metrics: Option<JobMetrics>,
    pub error: Option<String>,
    pub report: Option<PluginReport>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ExecutionEvent {
    JobStateChanged {
        job_id: Uuid,
        state: JobState,
    },
    JobFinished {
        job_id: Uuid,
        report: PluginReport,
        metrics: JobMetrics,
    },
    JobFailed {
        job_id: Uuid,
        error: String,
        metrics: Option<JobMetrics>,
    },
    JobLog {
        job_id: Uuid,
        message: String,
    },
}

pub struct ExecutionManager<R> {
    runtime: Arc<R>,
    jobs: Arc<Mutex<Vec<Job>>>,
    event_tx: broadcast::Sender<ExecutionEvent>,
    store: Option<WorkspaceStore>,
    app_config: AppConfig,
}

impl<R> ExecutionManager<R>
where
    R: PluginRuntime + Send + Sync + 'static,
{
    pub fn runtime(&self) -> &Arc<R> {
        &self.runtime
    }

    pub fn new(runtime: R, store: Option<WorkspaceStore>) -> Self {
        Self::new_with_config(runtime, store, AppConfig::development())
    }

    /// Construct an execution manager with explicit host paths and policy
    /// defaults. Desktop and server clients should use this constructor so a
    /// job never falls back to process-working-directory data.
    pub fn new_with_config(
        runtime: R,
        store: Option<WorkspaceStore>,
        app_config: AppConfig,
    ) -> Self {
        let (tx, _) = broadcast::channel(100);
        let mut jobs_list = Vec::new();
        let default_fuel_limit = app_config.max_wasm_fuel;
        let default_memory_limit = app_config.max_wasm_memory_bytes;
        if let Some(ref s) = store {
            if let Ok(records) = s.executions().list() {
                for r in records {
                    let plugin_path = format!("plugins/{}.wasm", r.plugin_id);
                    let state = match r.state.as_str() {
                        "Queued" => JobState::Queued,
                        "Starting" => JobState::Starting,
                        "Running" => JobState::Running,
                        "Completed" => JobState::Completed,
                        "Failed" => JobState::Failed,
                        "Cancelled" => JobState::Cancelled,
                        "TimedOut" => JobState::TimedOut,
                        _ => JobState::Completed,
                    };
                    let duration = Duration::from_millis(r.duration_ms);
                    let metrics = JobMetrics {
                        duration,
                        fuel_consumed: Some(r.fuel_consumed),
                        memory_used: None,
                        timestamp: r.started_at,
                        stage: None,
                    };
                    let mut report = None;
                    if state == JobState::Completed {
                        if let Ok(Some(rep_rec)) = s.reports().get(&r.job_id.to_string()) {
                            report = Some(PluginReport {
                                plugin_name: rep_rec.plugin_name,
                                target_tested: rep_rec.target,
                                issues: rep_rec.issues,
                                summary: rep_rec.summary,
                            });
                        }
                    }
                    jobs_list.push(Job {
                        id: r.job_id,
                        plugin_path,
                        target: r.target,
                        state,
                        config: ExecutionConfig {
                            fuel_limit: default_fuel_limit,
                            timeout: Duration::from_secs(30),
                            memory_limit: default_memory_limit,
                            allowed_capabilities: vec![],
                            project_id: r.project_id,
                            approval_ids: r.approval_ids,
                            plugin_version: r.plugin_version,
                            plugin_checksum: r.plugin_checksum,
                        },
                        metrics: Some(metrics),
                        error: r.error_message,
                        report,
                    });
                }
            }
        }

        jobs_list.reverse();

        Self {
            runtime: Arc::new(runtime),
            jobs: Arc::new(Mutex::new(jobs_list)),
            event_tx: tx,
            store,
            app_config,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ExecutionEvent> {
        self.event_tx.subscribe()
    }

    pub fn get_jobs(&self) -> Vec<Job> {
        self.jobs.lock().unwrap().clone()
    }

    pub fn submit_job(&self, plugin_path: String, target: String, config: ExecutionConfig) -> Uuid {
        let job_id = Uuid::new_v4();
        let job = Job {
            id: job_id,
            plugin_path: plugin_path.clone(),
            target: target.clone(),
            state: JobState::Queued,
            config: config.clone(),
            metrics: None,
            error: None,
            report: None,
        };

        {
            let mut jobs = self.jobs.lock().unwrap();
            jobs.push(job);
        }

        let plugin_id = Path::new(&plugin_path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| plugin_path.clone());

        if let Some(ref store) = self.store {
            if let Err(error) = store.executions().insert_job(
                &job_id,
                &plugin_id,
                &target,
                "Queued",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                config.project_id.as_deref(),
                &config.approval_ids,
                &config.plugin_version,
                &config.plugin_checksum,
            ) {
                let error = format!("Failed to persist queued execution: {error}");
                if let Some(job) = self
                    .jobs
                    .lock()
                    .unwrap()
                    .iter_mut()
                    .find(|job| job.id == job_id)
                {
                    job.state = JobState::Failed;
                    job.error = Some(error.clone());
                }
                let _ = self.event_tx.send(ExecutionEvent::JobFailed {
                    job_id,
                    error,
                    metrics: None,
                });
                return job_id;
            }
        }

        let _ = self.event_tx.send(ExecutionEvent::JobStateChanged {
            job_id,
            state: JobState::Queued,
        });

        // Spawn background execution
        let jobs_clone = Arc::clone(&self.jobs);
        let runtime_clone = Arc::clone(&self.runtime);
        let tx_clone = self.event_tx.clone();
        let store_clone = self.store.clone();
        let plugin_path_clone = plugin_path.clone();
        let target_clone = target.clone();
        let app_config = self.app_config.clone();
        let timeout = config.timeout;

        std::thread::spawn(move || {
            // Update to Starting
            {
                let mut jobs = jobs_clone.lock().unwrap();
                if let Some(j) = jobs.iter_mut().find(|j| j.id == job_id) {
                    if j.state != JobState::Queued {
                        return; // Already cancelled or timed out
                    }
                    j.state = JobState::Starting;
                }
            }
            if let Some(ref store) = store_clone {
                let _ = store
                    .executions()
                    .update_job(&job_id, "Starting", 0, 0, None);
            }
            let _ = tx_clone.send(ExecutionEvent::JobStateChanged {
                job_id,
                state: JobState::Starting,
            });

            // Update to Running
            {
                let mut jobs = jobs_clone.lock().unwrap();
                if let Some(j) = jobs.iter_mut().find(|j| j.id == job_id) {
                    if j.state != JobState::Starting {
                        return;
                    }
                    j.state = JobState::Running;
                }
            }
            if let Some(ref store) = store_clone {
                let _ = store
                    .executions()
                    .update_job(&job_id, "Running", 0, 0, None);
            }
            let _ = tx_clone.send(ExecutionEvent::JobStateChanged {
                job_id,
                state: JobState::Running,
            });

            let start_time = Instant::now();
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            // Configure permissions dynamically
            let mut permissions = InMemoryPermissionStore::default();
            for request in &config.allowed_capabilities {
                permissions.grant_request_for_all(request.clone());
            }

            let reports_dir = app_config.reports_dir.clone();
            let mut app_config = app_config;
            app_config.max_wasm_fuel = config.fuel_limit;
            app_config.max_wasm_memory_bytes = config.memory_limit;

            let mut engine = match CoreEngine::new(
                Arc::clone(&runtime_clone),
                permissions,
                VecEventSink::default(),
                app_config,
            ) {
                Ok(eng) => eng,
                Err(err) => {
                    let err_msg = format!("Failed to create core engine: {err}");
                    fail_job(
                        job_id,
                        &jobs_clone,
                        &tx_clone,
                        err_msg,
                        start_time,
                        timestamp,
                        &store_clone,
                    );
                    return;
                }
            };

            let parsed_target = match Target::parse(&target) {
                Ok(t) => t,
                Err(err) => {
                    let err_msg = format!("Invalid target: {err}");
                    fail_job(
                        job_id,
                        &jobs_clone,
                        &tx_clone,
                        err_msg,
                        start_time,
                        timestamp,
                        &store_clone,
                    );
                    return;
                }
            };

            let req = PluginRunRequest {
                plugin: PluginRef::from_path(PathBuf::from(&plugin_path_clone)),
                target: parsed_target,
            };

            // Execute the plugin run
            CURRENT_JOB_ID.with(|cell| cell.set(Some(job_id)));

            let result = engine.run_plugin(req);

            CURRENT_JOB_ID.with(|cell| cell.set(None));

            let duration = start_time.elapsed();
            let metrics = JobMetrics {
                duration,
                // Wasmtime does not currently expose consumed fuel through the
                // runtime port. Never present the configured budget as usage.
                fuel_consumed: None,
                memory_used: None,
                timestamp,
                stage: Some("Finished".to_string()),
            };

            // Check if job timed out or cancelled during execution
            {
                let mut jobs = jobs_clone.lock().unwrap();
                if let Some(j) = jobs.iter_mut().find(|j| j.id == job_id) {
                    if j.state == JobState::Cancelled || j.state == JobState::TimedOut {
                        return; // Execution interrupted
                    }
                }
            }

            match result {
                Ok(report) => {
                    if let Some(ref store) = store_clone {
                        let plugin_id_obj = match PluginId::new(&plugin_id) {
                            Ok(plugin_id) => plugin_id,
                            Err(error) => {
                                fail_job(
                                    job_id,
                                    &jobs_clone,
                                    &tx_clone,
                                    format!("Invalid persisted plugin identity: {error}"),
                                    start_time,
                                    timestamp,
                                    &store_clone,
                                );
                                return;
                            }
                        };
                        let security_profile = store
                            .settings()
                            .get("security_profile")
                            .ok()
                            .flatten()
                            .unwrap_or_else(|| "Balanced".to_string());
                        if let Err(error) = persist_completed_report(
                            store,
                            &reports_dir,
                            job_id,
                            &plugin_id_obj,
                            &target_clone,
                            &report,
                            &config,
                            &metrics,
                            &security_profile,
                        ) {
                            fail_job(
                                job_id,
                                &jobs_clone,
                                &tx_clone,
                                format!("Report persistence failed: {error}"),
                                start_time,
                                timestamp,
                                &store_clone,
                            );
                            return;
                        }
                    }
                    {
                        let mut jobs = jobs_clone.lock().unwrap();
                        if let Some(j) = jobs.iter_mut().find(|j| j.id == job_id) {
                            j.state = JobState::Completed;
                            j.metrics = Some(metrics.clone());
                            j.report = Some(report.clone());
                        }
                    }
                    let _ = tx_clone.send(ExecutionEvent::JobFinished {
                        job_id,
                        report,
                        metrics,
                    });
                }
                Err(err) => {
                    let err_msg = format!("Run failed: {err}");
                    fail_job(
                        job_id,
                        &jobs_clone,
                        &tx_clone,
                        err_msg,
                        start_time,
                        timestamp,
                        &store_clone,
                    );
                }
            }
        });

        // Spawn timeout thread
        let jobs_clone_to = Arc::clone(&self.jobs);
        let runtime_clone_to = Arc::clone(&self.runtime);
        let tx_clone_to = self.event_tx.clone();
        let store_clone_to = self.store.clone();
        std::thread::spawn(move || {
            std::thread::sleep(timeout);
            let mut jobs = jobs_clone_to.lock().unwrap();
            if let Some(j) = jobs.iter_mut().find(|j| j.id == job_id) {
                if j.state == JobState::Running
                    || j.state == JobState::Starting
                    || j.state == JobState::Queued
                {
                    j.state = JobState::TimedOut;
                    j.error = Some("Job execution timed out".to_string());
                    let _ = runtime_clone_to.cancel(job_id);
                    if let Some(ref store) = store_clone_to {
                        let _ = store.executions().update_job(
                            &job_id,
                            "TimedOut",
                            0,
                            0,
                            Some("Job execution timed out"),
                        );
                    }
                    let _ = tx_clone_to.send(ExecutionEvent::JobStateChanged {
                        job_id,
                        state: JobState::TimedOut,
                    });
                }
            }
        });

        job_id
    }

    pub fn cancel_job(&self, job_id: Uuid) -> Result<(), String> {
        let mut jobs = self.jobs.lock().unwrap();
        if let Some(j) = jobs.iter_mut().find(|j| j.id == job_id) {
            if j.state == JobState::Completed
                || j.state == JobState::Failed
                || j.state == JobState::Cancelled
                || j.state == JobState::TimedOut
            {
                return Err("Job has already completed/terminated".to_string());
            }

            j.state = JobState::Cancelled;
            j.error = Some("Job execution cancelled by user".to_string());
            let _ = self.runtime.cancel(job_id);
            if let Some(ref store) = self.store {
                let _ = store.executions().update_job(
                    &job_id,
                    "Cancelled",
                    0,
                    0,
                    Some("Job execution cancelled by user"),
                );
            }
            let _ = self.event_tx.send(ExecutionEvent::JobStateChanged {
                job_id,
                state: JobState::Cancelled,
            });
            Ok(())
        } else {
            Err("Job not found".to_string())
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn persist_completed_report(
    store: &WorkspaceStore,
    reports_dir: &Path,
    job_id: Uuid,
    plugin_id: &PluginId,
    target: &str,
    report: &PluginReport,
    config: &ExecutionConfig,
    metrics: &JobMetrics,
    security_profile: &str,
) -> Result<(), String> {
    std::fs::create_dir_all(reports_dir)
        .map_err(|error| format!("failed to create reports directory: {error}"))?;

    let exported = reports::ExportedReport {
        metadata: reports::ReportMetadata {
            polyglid_version: env!("CARGO_PKG_VERSION").to_string(),
            plugin_id: plugin_id.as_str().to_string(),
            plugin_version: config.plugin_version.clone(),
            target: target.to_string(),
            timestamp: metrics.timestamp,
            security_profile: security_profile.to_string(),
            execution_duration_ms: metrics.duration.as_millis() as u64,
            report_format_version: "1.0".to_string(),
        },
        report: report.clone(),
    };
    let payload = reports::json::export(&exported)?;
    let filepath = reports_dir.join(format!("report_{job_id}.json"));
    let temporary = reports_dir.join(format!(".report_{job_id}.json.tmp"));
    std::fs::write(&temporary, payload)
        .map_err(|error| format!("failed to write temporary report: {error}"))?;
    std::fs::rename(&temporary, &filepath)
        .map_err(|error| format!("failed to publish report atomically: {error}"))?;

    store.reports().insert(
        &job_id.to_string(),
        &job_id,
        plugin_id,
        target,
        report,
        &filepath.to_string_lossy(),
        config.project_id.as_deref(),
        &config.plugin_version,
        &config.plugin_checksum,
        metrics.duration.as_millis() as u64,
        metrics.fuel_consumed,
        metrics.memory_used,
        security_profile,
    )?;

    store.executions().update_job(
        &job_id,
        "Completed",
        metrics.duration.as_millis() as u64,
        metrics.fuel_consumed.unwrap_or(0),
        None,
    )
}

fn fail_job(
    job_id: Uuid,
    jobs: &Arc<Mutex<Vec<Job>>>,
    tx: &broadcast::Sender<ExecutionEvent>,
    error: String,
    start_time: Instant,
    timestamp: u64,
    store: &Option<WorkspaceStore>,
) {
    let metrics = JobMetrics {
        duration: start_time.elapsed(),
        fuel_consumed: None,
        memory_used: None,
        timestamp,
        stage: Some("Failed".to_string()),
    };

    {
        let mut jobs_lock = jobs.lock().unwrap();
        if let Some(j) = jobs_lock.iter_mut().find(|j| j.id == job_id) {
            j.state = JobState::Failed;
            j.error = Some(error.clone());
            j.metrics = Some(metrics.clone());
        }
    }

    if let Some(ref s) = store {
        let duration = start_time.elapsed().as_millis() as u64;
        let _ = s
            .executions()
            .update_job(&job_id, "Failed", duration, 0, Some(&error));
    }

    let _ = tx.send(ExecutionEvent::JobFailed {
        job_id,
        error,
        metrics: Some(metrics),
    });
}

use std::cell::Cell;

thread_local! {
    pub static CURRENT_JOB_ID: Cell<Option<Uuid>> = const { Cell::new(None) };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CoreError, PluginManifest};
    use polyglid_config::plugin_registry::{PluginRegistryEntry, PluginSource, PluginStatus};
    use polyglid_plugin_api::{Capability, PluginId};
    use semver::Version;

    struct MockRuntime {
        delay: Duration,
    }

    impl PluginRuntime for MockRuntime {
        fn inspect(&self, _plugin: &PluginRef) -> Result<PluginManifest, CoreError> {
            Ok(PluginManifest {
                id: PluginId::new("mock").unwrap(),
                name: "Mock Plugin".to_string(),
                version: "1.0.0".to_string(),
                requested_capabilities: vec![],
            })
        }

        fn inspect_metadata(
            &self,
            _plugin: &PluginRef,
        ) -> Result<polyglid_plugin_api::ApiPluginMetadata, CoreError> {
            Ok(polyglid_plugin_api::ApiPluginMetadata {
                name: "mock".to_string(),
                display_name: "Mock Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "mocked runtime".to_string(),
                author: "mock author".to_string(),
            })
        }

        fn execute(
            &self,
            request: &PluginRunRequest,
            _config: &AppConfig,
            _effective_grants: &[polyglid_plugin_api::CapabilityRequest],
        ) -> Result<PluginReport, CoreError> {
            std::thread::sleep(self.delay);
            Ok(PluginReport {
                plugin_name: "Mock Plugin".to_string(),
                target_tested: request.target.as_str().to_string(),
                issues: vec![],
                summary: "Success".to_string(),
            })
        }
    }

    struct ConfigCapturingRuntime {
        observed: Arc<Mutex<Option<AppConfig>>>,
    }

    impl PluginRuntime for ConfigCapturingRuntime {
        fn inspect(&self, _plugin: &PluginRef) -> Result<PluginManifest, CoreError> {
            Ok(PluginManifest {
                id: PluginId::new("config-capture").unwrap(),
                name: "Config Capture".to_string(),
                version: "1.0.0".to_string(),
                requested_capabilities: vec![],
            })
        }

        fn inspect_metadata(
            &self,
            _plugin: &PluginRef,
        ) -> Result<polyglid_plugin_api::ApiPluginMetadata, CoreError> {
            Ok(polyglid_plugin_api::ApiPluginMetadata {
                name: "config-capture".to_string(),
                display_name: "Config Capture".to_string(),
                version: "1.0.0".to_string(),
                description: "captures runtime configuration".to_string(),
                author: "test".to_string(),
            })
        }

        fn execute(
            &self,
            request: &PluginRunRequest,
            config: &AppConfig,
            _effective_grants: &[polyglid_plugin_api::CapabilityRequest],
        ) -> Result<PluginReport, CoreError> {
            *self.observed.lock().unwrap() = Some(config.clone());
            Ok(PluginReport::clean(
                "Config Capture",
                request.target.as_str(),
            ))
        }
    }

    #[test]
    fn test_successful_job_execution() {
        let manager = ExecutionManager::new(
            MockRuntime {
                delay: Duration::from_millis(10),
            },
            None,
        );
        let mut rx = manager.subscribe();

        let config = ExecutionConfig {
            fuel_limit: 1000,
            timeout: Duration::from_secs(1),
            memory_limit: None,
            allowed_capabilities: vec![],
            project_id: None,
            approval_ids: Vec::new(),
            plugin_version: String::new(),
            plugin_checksum: String::new(),
        };

        let job_id =
            manager.submit_job("plugin.wasm".to_string(), "example.com".to_string(), config);

        // Track events
        let mut states = Vec::new();
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(2) {
            if let Ok(event) = rx.try_recv() {
                match event {
                    ExecutionEvent::JobStateChanged { job_id: id, state } if id == job_id => {
                        states.push(state);
                    }
                    ExecutionEvent::JobFinished { job_id: id, .. } if id == job_id => {
                        states.push(JobState::Completed);
                        break;
                    }
                    _ => {}
                }
            }
            std::thread::sleep(Duration::from_millis(5));
        }

        assert!(states.contains(&JobState::Queued));
        assert!(states.contains(&JobState::Running));
        assert!(states.contains(&JobState::Completed));

        let jobs = manager.get_jobs();
        let job = jobs.iter().find(|j| j.id == job_id).unwrap();
        assert_eq!(job.state, JobState::Completed);
        assert!(job.report.is_some());
    }

    #[test]
    fn completed_job_persists_a_real_report_and_rehydrates_safe_limits() {
        let root = std::env::temp_dir().join(format!(
            "polyglid-report-roundtrip-{}",
            uuid::Uuid::new_v4()
        ));
        let reports_dir = root.join("reports");
        std::fs::create_dir_all(&root).expect("test directory");
        let store = WorkspaceStore::new(&root.join("polyglid.db")).expect("workspace store");
        store
            .plugins()
            .insert(&PluginRegistryEntry {
                id: PluginId::new("mock").expect("plugin id"),
                name: "Mock Plugin".to_string(),
                version: Version::new(1, 2, 3),
                author: "PolyGlid".to_string(),
                description: "report fixture".to_string(),
                capabilities: Vec::<Capability>::new(),
                checksum: "abc123".to_string(),
                status: PluginStatus::Enabled,
                source: PluginSource::LocalPath(PathBuf::from("mock.wasm")),
                file_size: 1,
                installed_at: 1,
                last_updated: 1,
                path: PathBuf::from("mock.wasm"),
            })
            .expect("register test plugin");
        let app_config = AppConfig {
            plugin_dir: root.join("plugins"),
            reports_dir: reports_dir.clone(),
            max_wasm_fuel: 777_000,
            max_wasm_memory_bytes: Some(32 * 1024 * 1024),
            ..AppConfig::development()
        };
        let manager = ExecutionManager::new_with_config(
            MockRuntime {
                delay: Duration::from_millis(1),
            },
            Some(store.clone()),
            app_config.clone(),
        );
        let mut events = manager.subscribe();
        let job_id = manager.submit_job(
            "mock.wasm".to_string(),
            "example.com".to_string(),
            ExecutionConfig {
                fuel_limit: 1_000,
                timeout: Duration::from_secs(1),
                memory_limit: Some(8 * 1024 * 1024),
                allowed_capabilities: vec![],
                project_id: Some("project-1".to_string()),
                approval_ids: vec!["approval-1".to_string()],
                plugin_version: "1.2.3".to_string(),
                plugin_checksum: "abc123".to_string(),
            },
        );

        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match events.try_recv() {
                Ok(ExecutionEvent::JobFinished { job_id: id, .. }) if id == job_id => break,
                Ok(ExecutionEvent::JobFailed { error, .. }) => panic!("{error}"),
                _ if Instant::now() >= deadline => panic!("job did not finish"),
                _ => std::thread::sleep(Duration::from_millis(5)),
            }
        }

        let stored = store
            .reports()
            .get(&job_id.to_string())
            .expect("report query")
            .expect("persisted report");
        assert_eq!(stored.plugin_name, "Mock Plugin");
        assert_eq!(stored.plugin_version, "1.2.3");
        assert_eq!(stored.project_id.as_deref(), Some("project-1"));
        assert!(Path::new(&stored.filepath).is_file());
        let payload = std::fs::read_to_string(&stored.filepath).expect("report file");
        let exported: reports::ExportedReport =
            serde_json::from_str(&payload).expect("valid exported report");
        assert_eq!(exported.metadata.plugin_id, "mock");

        let reopened = ExecutionManager::new_with_config(
            MockRuntime {
                delay: Duration::ZERO,
            },
            Some(store),
            app_config,
        );
        let job = reopened
            .get_jobs()
            .into_iter()
            .find(|job| job.id == job_id)
            .expect("rehydrated job");
        assert_eq!(job.config.fuel_limit, 777_000);
        assert_eq!(job.config.memory_limit, Some(32 * 1024 * 1024));

        std::fs::remove_dir_all(root).expect("remove test data");
    }

    #[test]
    fn test_job_execution_timeout() {
        let manager = ExecutionManager::new(
            MockRuntime {
                delay: Duration::from_millis(100),
            },
            None,
        );
        let mut rx = manager.subscribe();

        let config = ExecutionConfig {
            fuel_limit: 1000,
            timeout: Duration::from_millis(20),
            memory_limit: None,
            allowed_capabilities: vec![],
            project_id: None,
            approval_ids: Vec::new(),
            plugin_version: String::new(),
            plugin_checksum: String::new(),
        };

        let job_id =
            manager.submit_job("plugin.wasm".to_string(), "example.com".to_string(), config);

        let mut timed_out = false;
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(2) {
            if let Ok(ExecutionEvent::JobStateChanged { job_id: id, state }) = rx.try_recv() {
                if id == job_id && state == JobState::TimedOut {
                    timed_out = true;
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(5));
        }

        assert!(timed_out);
        let jobs = manager.get_jobs();
        let job = jobs.iter().find(|j| j.id == job_id).unwrap();
        assert_eq!(job.state, JobState::TimedOut);
        assert!(job.error.as_ref().unwrap().contains("timed out"));
    }

    #[test]
    fn test_job_execution_cancellation() {
        let manager = ExecutionManager::new(
            MockRuntime {
                delay: Duration::from_millis(200),
            },
            None,
        );
        let mut rx = manager.subscribe();

        let config = ExecutionConfig {
            fuel_limit: 1000,
            timeout: Duration::from_secs(2),
            memory_limit: None,
            allowed_capabilities: vec![],
            project_id: None,
            approval_ids: Vec::new(),
            plugin_version: String::new(),
            plugin_checksum: String::new(),
        };

        let job_id =
            manager.submit_job("plugin.wasm".to_string(), "example.com".to_string(), config);

        // Cancel immediately
        std::thread::sleep(Duration::from_millis(10));
        let cancel_res = manager.cancel_job(job_id);
        assert!(cancel_res.is_ok());

        let mut cancelled = false;
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(2) {
            if let Ok(ExecutionEvent::JobStateChanged { job_id: id, state }) = rx.try_recv() {
                if id == job_id && state == JobState::Cancelled {
                    cancelled = true;
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(5));
        }

        assert!(cancelled);
        let jobs = manager.get_jobs();
        let job = jobs.iter().find(|j| j.id == job_id).unwrap();
        assert_eq!(job.state, JobState::Cancelled);
    }

    #[test]
    fn explicit_app_config_reaches_runtime_jobs() {
        let observed = Arc::new(Mutex::new(None));
        let expected_reports = PathBuf::from("/tmp/polyglid-explicit-reports");
        let manager = ExecutionManager::new_with_config(
            ConfigCapturingRuntime {
                observed: Arc::clone(&observed),
            },
            None,
            AppConfig {
                plugin_dir: PathBuf::from("/tmp/polyglid-explicit-plugins"),
                reports_dir: expected_reports.clone(),
                ..AppConfig::development()
            },
        );
        let mut events = manager.subscribe();
        manager.submit_job(
            "plugin.wasm".to_string(),
            "example.com".to_string(),
            ExecutionConfig {
                fuel_limit: 123_456,
                timeout: Duration::from_secs(1),
                memory_limit: None,
                allowed_capabilities: vec![],
                project_id: None,
                approval_ids: Vec::new(),
                plugin_version: String::new(),
                plugin_checksum: String::new(),
            },
        );

        while !matches!(
            events.blocking_recv().expect("execution event"),
            ExecutionEvent::JobFinished { .. }
        ) {}

        let config = observed.lock().unwrap().clone().expect("observed config");
        assert_eq!(config.reports_dir, expected_reports);
        assert_eq!(config.max_wasm_fuel, 123_456);
    }
}
