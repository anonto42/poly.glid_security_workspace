use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use anyhow::Result;
use tracing::{info, error};
use tokio::time::{interval, Duration};

#[derive(Clone)]
pub struct AutoSuggester {
    workspace: PathBuf,
    running: Arc<AtomicBool>,
}

impl AutoSuggester {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace, running: Arc::new(AtomicBool::new(false)) }
    }

    pub async fn run(&mut self) -> Result<()> {
        self.running.store(true, Ordering::SeqCst);
        info!("Auto-suggester started");

        let mut check_interval = interval(Duration::from_secs(300)); // every 5 min

        while self.running.load(Ordering::SeqCst) {
            check_interval.tick().await;
            if let Err(e) = self.suggest().await {
                error!("Suggestion error: {}", e);
            }
        }

        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    async fn suggest(&self) -> Result<()> {
        let predictions_dir = self.workspace.join(".workspace").join("ai").join("predictions");
        if !predictions_dir.exists() {
            return Ok(());
        }

        let mut rd = tokio::fs::read_dir(&predictions_dir).await?;
        let mut pending_count = 0usize;

        while let Ok(Some(entry)) = rd.next_entry().await {
            if entry.path().extension().map(|e| e == "json").unwrap_or(false) {
                let content = tokio::fs::read_to_string(entry.path()).await?;
                if content.contains("\"Pending\"") || content.contains("\"status\": \"Pending\"") {
                    pending_count += 1;
                }
            }
        }

        if pending_count > 0 {
            info!("{} pending predictions awaiting feedback", pending_count);
        }

        Ok(())
    }
}
