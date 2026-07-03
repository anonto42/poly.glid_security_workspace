//! Asynchronous execution engine and job manager for PolyGlid.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;
use uuid::Uuid;

use polyglid_config::AppConfig;
use polyglid_events::VecEventSink;
use polyglid_plugin_api::{Capability, PluginReport};

use crate::{
    CoreEngine, InMemoryPermissionStore, PluginRef, PluginRunRequest, PluginRuntime,
    Target,
};

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

#[derive(Debug, Clone)]
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
}

impl<R> ExecutionManager<R>
where
    R: PluginRuntime + Send + Sync + 'static,
{
    pub fn new(runtime: R) -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            runtime: Arc::new(runtime),
            jobs: Arc::new(Mutex::new(Vec::new())),
            event_tx: tx,
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

        let _ = self.event_tx.send(ExecutionEvent::JobStateChanged {
            job_id,
            state: JobState::Queued,
        });

        // Spawn background execution
        let jobs_clone = Arc::clone(&self.jobs);
        let runtime_clone = Arc::clone(&self.runtime);
        let tx_clone = self.event_tx.clone();

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
            let _ = tx_clone.send(ExecutionEvent::JobStateChanged {
                job_id,
                state: JobState::Running,
            });

            let start_time = Instant::now();
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            // Set the thread-local CURRENT_JOB_ID for runtime registry mapping
            // (CURRENT_JOB_ID is defined in polyglid-runtime which is downstream, but
            // wait: polyglid-core cannot import CURRENT_JOB_ID directly.
            // But wait! polyglid-runtime registers the engine. Does polyglid-core need to set it?
            // Yes, polyglid-runtime's thread local CURRENT_JOB_ID is used when instantiate_plugin is called.
            // Since they run on the same spawned thread, if we set the thread-local in the runtime crate,
            // we can expose a helper function in runtime, or we can just let runtime's CURRENT_JOB_ID
            // be set. Since core does not depend on runtime, we can define a thread-local in polyglid-core
            // and let runtime import it!
            // Yes! polyglid-runtime depends on polyglid-core. So if we define CURRENT_JOB_ID in polyglid-core,
            // then runtime can read it directly from core!
            // This is perfect! Let's declare CURRENT_JOB_ID in crates/polyglid-core/src/lib.rs!)

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
                    fail_job(job_id, &jobs_clone, &tx_clone, err_msg, start_time, timestamp);
                    return;
                }
            };

            let parsed_target = match Target::parse(&target) {
                Ok(t) => t,
                Err(err) => {
                    let err_msg = format!("Invalid target: {err}");
                    fail_job(job_id, &jobs_clone, &tx_clone, err_msg, start_time, timestamp);
                    return;
                }
            };

            let req = PluginRunRequest {
                plugin: PluginRef::from_path(PathBuf::from(&plugin_path)),
                target: parsed_target,
            };

            // Execute the plugin run
            // We set the thread-local CURRENT_JOB_ID in core so runtime can read it.
            CURRENT_JOB_ID.with(|cell| cell.set(Some(job_id)));

            let result = engine.run_plugin(req);

            CURRENT_JOB_ID.with(|cell| cell.set(None));

            let duration = start_time.elapsed();
            let metrics = JobMetrics {
                duration,
                fuel_consumed: Some(config.fuel_limit), // Default/estimated for fuel monitoring
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
                    let _ = tx_clone.send(ExecutionEvent::JobFinished {
                        job_id,
                        report,
                        metrics,
                    });
                }
                Err(err) => {
                    let err_msg = format!("Run failed: {err}");
                    fail_job(job_id, &jobs_clone, &tx_clone, err_msg, start_time, timestamp);
                }
            }
        });

        // Spawn timeout thread
        let jobs_clone_to = Arc::clone(&self.jobs);
        let runtime_clone_to = Arc::clone(&self.runtime);
        let tx_clone_to = self.event_tx.clone();
        let timeout = config.timeout;

        std::thread::spawn(move || {
            std::thread::sleep(timeout);
            let mut jobs = jobs_clone_to.lock().unwrap();
            if let Some(j) = jobs.iter_mut().find(|j| j.id == job_id) {
                if j.state == JobState::Running || j.state == JobState::Starting || j.state == JobState::Queued {
                    j.state = JobState::TimedOut;
                    j.error = Some("Job execution timed out".to_string());
                    let _ = runtime_clone_to.cancel(job_id);
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

    let _ = tx.send(ExecutionEvent::JobFailed {
        job_id,
        error,
        metrics: Some(metrics),
    });
}

use std::cell::Cell;
use std::path::PathBuf;

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
        });
        let mut rx = manager.subscribe();

        let config = ExecutionConfig {
            fuel_limit: 1000,
            timeout: Duration::from_secs(1),
            memory_limit: None,
            allowed_capabilities: vec![],
        };

        let job_id = manager.submit_job("plugin.wasm".to_string(), "example.com".to_string(), config);

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
        });
        let mut rx = manager.subscribe();

        let config = ExecutionConfig {
            fuel_limit: 1000,
            timeout: Duration::from_millis(20),
            memory_limit: None,
            allowed_capabilities: vec![],
        };

        let job_id = manager.submit_job("plugin.wasm".to_string(), "example.com".to_string(), config);

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
        });
        let mut rx = manager.subscribe();

        let config = ExecutionConfig {
            fuel_limit: 1000,
            timeout: Duration::from_secs(2),
            memory_limit: None,
            allowed_capabilities: vec![],
        };

        let job_id = manager.submit_job("plugin.wasm".to_string(), "example.com".to_string(), config);

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
