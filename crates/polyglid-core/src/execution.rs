//! Asynchronous execution engine and job manager for PolyGlid.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;
use uuid::Uuid;

use polyglid_config::AppConfig;
use polyglid_events::VecEventSink;
use polyglid_plugin_api::{Capability, PluginId, PluginReport};

use crate::{
    CoreEngine, InMemoryPermissionStore, PluginRef, PluginRunRequest, PluginRuntime, Target,
};
use crate::store::WorkspaceStore;

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
    pub allowed_capabilities: Vec<Capability>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JobMetrics {
    pub duration: Duration,
    pub fuel_consumed: Option<u64>,
    pub memory_used: Option<u64>,
    pub timestamp: u64,
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
}

impl<R> ExecutionManager<R>
where
    R: PluginRuntime + Send + Sync + 'static,
{
    pub fn runtime(&self) -> &Arc<R> {
        &self.runtime
    }

    pub fn new(runtime: R, store: Option<WorkspaceStore>) -> Self {
        let (tx, _) = broadcast::channel(100);
        let mut jobs_list = Vec::new();
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
                    };
                    let mut report = None;
                    if state == JobState::Completed {
                        if let Ok(Some(rep_rec)) = s.reports().get(&r.job_id.to_string()) {
                            report = Some(PluginReport {
                                plugin_name: rep_rec.plugin_id,
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
                            fuel_limit: r.fuel_consumed,
                            timeout: Duration::from_secs(30),
                            memory_limit: None,
                            allowed_capabilities: vec![],
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
            let _ = store.executions().insert_job(
                &job_id,
                &plugin_id,
                &target,
                "Queued",
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
            );
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
                let _ = store.executions().update_job(&job_id, "Starting", 0, 0, None);
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
                let _ = store.executions().update_job(&job_id, "Running", 0, 0, None);
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
            for cap in &config.allowed_capabilities {
                permissions.grant_for_all(*cap);
            }

            let mut app_config = AppConfig::development();
            app_config.max_wasm_fuel = config.fuel_limit;

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
                fuel_consumed: Some(config.fuel_limit), 
                memory_used: None,
                timestamp,
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
                    {
                        let mut jobs = jobs_clone.lock().unwrap();
                        if let Some(j) = jobs.iter_mut().find(|j| j.id == job_id) {
                            j.state = JobState::Completed;
                            j.metrics = Some(metrics.clone());
                            j.report = Some(report.clone());
                        }
                    }
                    if let Some(ref store) = store_clone {
                        let _ = store.executions().update_job(
                            &job_id,
                            "Completed",
                            duration.as_millis() as u64,
                            config.fuel_limit,
                            None,
                        );

                        let plugin_id_obj = PluginId::new(&plugin_id)
                            .unwrap_or_else(|_| PluginId::new("unknown").unwrap());
                        let filepath = format!("reports/report_{}.json", job_id);
                        let _ = store.reports().insert(
                            &job_id.to_string(),
                            &job_id,
                            &plugin_id_obj,
                            &target_clone,
                            &report.summary,
                            &report.issues,
                            &filepath,
                        );
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
        let timeout = config.timeout;

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
        let _ = s.executions().update_job(
            &job_id,
            "Failed",
            duration,
            0,
            Some(&error),
        );
    }

    let _ = tx.send(ExecutionEvent::JobFailed {
        job_id,
        error,
        metrics: Some(metrics),
    });
}

use std::cell::Cell;

thread_local! {
    pub static CURRENT_JOB_ID: Cell<Option<Uuid>> = Cell::new(None);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CoreError, PluginManifest};
    use polyglid_plugin_api::PluginId;

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

    #[test]
    fn test_successful_job_execution() {
        let manager = ExecutionManager::new(MockRuntime {
            delay: Duration::from_millis(10),
        }, None);
        let mut rx = manager.subscribe();

        let config = ExecutionConfig {
            fuel_limit: 1000,
            timeout: Duration::from_secs(1),
            memory_limit: None,
            allowed_capabilities: vec![],
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
    fn test_job_execution_timeout() {
        let manager = ExecutionManager::new(MockRuntime {
            delay: Duration::from_millis(100),
        }, None);
        let mut rx = manager.subscribe();

        let config = ExecutionConfig {
            fuel_limit: 1000,
            timeout: Duration::from_millis(20),
            memory_limit: None,
            allowed_capabilities: vec![],
        };

        let job_id =
            manager.submit_job("plugin.wasm".to_string(), "example.com".to_string(), config);

        let mut timed_out = false;
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(2) {
            if let Ok(event) = rx.try_recv() {
                if let ExecutionEvent::JobStateChanged { job_id: id, state } = event {
                    if id == job_id && state == JobState::TimedOut {
                        timed_out = true;
                        break;
                    }
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
        let manager = ExecutionManager::new(MockRuntime {
            delay: Duration::from_millis(200),
        }, None);
        let mut rx = manager.subscribe();

        let config = ExecutionConfig {
            fuel_limit: 1000,
            timeout: Duration::from_secs(2),
            memory_limit: None,
            allowed_capabilities: vec![],
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
            if let Ok(event) = rx.try_recv() {
                if let ExecutionEvent::JobStateChanged { job_id: id, state } = event {
                    if id == job_id && state == JobState::Cancelled {
                        cancelled = true;
                        break;
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(5));
        }

        assert!(cancelled);
        let jobs = manager.get_jobs();
        let job = jobs.iter().find(|j| j.id == job_id).unwrap();
        assert_eq!(job.state, JobState::Cancelled);
    }
}
