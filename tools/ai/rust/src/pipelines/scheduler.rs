use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use anyhow::Result;
use tracing::{info, error};
use tokio::time::{interval, Duration};

#[derive(Clone)]
pub struct ScheduledTask {
    workspace: PathBuf,
    running: Arc<AtomicBool>,
}

impl ScheduledTask {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace, running: Arc::new(AtomicBool::new(false)) }
    }

    pub async fn run(&mut self) -> Result<()> {
        self.running.store(true, Ordering::SeqCst);
        info!("Scheduler started");

        let mut hourly = interval(Duration::from_secs(3600));
        let mut daily = interval(Duration::from_secs(86400));

        while self.running.load(Ordering::SeqCst) {
            tokio::select! {
                _ = hourly.tick() => {
                    self.run_hourly().await;
                }
                _ = daily.tick() => {
                    self.run_daily().await;
                }
            }
        }

        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    async fn run_hourly(&self) {
        info!("Running hourly tasks");
        // Dependency check
        let deps_path = self.workspace.join("projects").join("Cargo.toml");
        if deps_path.exists() {
            match tokio::fs::read_to_string(&deps_path).await {
                Ok(content) => info!("Dependency check: {} bytes", content.len()),
                Err(e) => error!("Failed to read Cargo.toml: {}", e),
            }
        }
    }

    async fn run_daily(&self) {
        info!("Running daily tasks");
        // Full workspace analysis
        let projects = self.workspace.join("projects");
        if projects.is_dir() {
            let mut count = 0usize;
            let mut rd = match tokio::fs::read_dir(&projects).await {
                Ok(r) => r,
                Err(e) => {
                    error!("Failed to read projects: {}", e);
                    return;
                }
            };
            while let Ok(Some(entry)) = rd.next_entry().await {
                if entry.path().is_dir() {
                    count += 1;
                }
            }
            info!("Daily scan: {} projects found", count);
        }
    }
}
