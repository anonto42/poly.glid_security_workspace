use std::path::PathBuf;
use crate::pipelines::watcher::FileWatcher;
use crate::pipelines::scheduler::ScheduledTask;
use crate::pipelines::autosuggest::AutoSuggester;
use anyhow::Result;
use tracing::{info, error};

pub struct PipelineManager {
    pub watcher: FileWatcher,
    pub scheduler: ScheduledTask,
    pub suggester: AutoSuggester,
    running: bool,
}

impl PipelineManager {
    pub fn new(workspace: PathBuf) -> Self {
        Self {
            watcher: FileWatcher::new(workspace.join("projects")),
            scheduler: ScheduledTask::new(workspace.clone()),
            suggester: AutoSuggester::new(workspace),
            running: false,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        self.running = true;
        info!("Pipeline manager started");

        let watcher_handle = tokio::spawn({
            let mut w = self.watcher.clone();
            async move {
                if let Err(e) = w.watch().await {
                    error!("File watcher error: {}", e);
                }
            }
        });

        let scheduler_handle = tokio::spawn({
            let mut s = self.scheduler.clone();
            async move {
                if let Err(e) = s.run().await {
                    error!("Scheduler error: {}", e);
                }
            }
        });

        let suggest_handle = tokio::spawn({
            let mut s = self.suggester.clone();
            async move {
                if let Err(e) = s.run().await {
                    error!("Auto-suggester error: {}", e);
                }
            }
        });

        tokio::select! {
            _ = watcher_handle => {},
            _ = scheduler_handle => {},
            _ = suggest_handle => {},
        }

        self.running = false;
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.running
    }
}
