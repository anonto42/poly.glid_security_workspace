use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use anyhow::{Result, anyhow};
use notify::{Watcher, RecursiveMode, Event, EventKind};
use tracing::{info, warn, error};
use tokio::sync::mpsc;
use std::sync::Arc;

#[derive(Clone)]
pub struct FileWatcher {
    watch_dir: PathBuf,
    running: Arc<AtomicBool>,
}

impl FileWatcher {
    pub fn new(watch_dir: PathBuf) -> Self {
        Self { watch_dir, running: Arc::new(AtomicBool::new(false)) }
    }

    pub async fn watch(&mut self) -> Result<()> {
        self.running.store(true, Ordering::SeqCst);

        let (tx, mut rx) = mpsc::channel::<Event>(256);
        let running = self.running.clone();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.blocking_send(event);
            }
        })?;

        watcher.watch(&self.watch_dir, RecursiveMode::Recursive)?;
        info!("Watching {} for file changes", self.watch_dir.display());

        while self.running.load(Ordering::SeqCst) {
            tokio::select! {
                Some(event) = rx.recv() => {
                    self.handle_event(&event).await;
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                    // heartbeat check
                }
            }
        }

        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    async fn handle_event(&self, event: &Event) {
        let paths: Vec<_> = event.paths.iter()
            .filter(|p| p.extension().map(|e| matches!(e.to_str(), Some("rs" | "toml" | "py" | "js" | "ts"))).unwrap_or(false))
            .collect();

        if paths.is_empty() {
            return;
        }

        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {
                for p in &paths {
                    info!("File changed: {}", p.display());
                }
            }
            EventKind::Remove(_) => {
                for p in &paths {
                    warn!("File removed: {}", p.display());
                }
            }
            _ => {}
        }
    }
}
