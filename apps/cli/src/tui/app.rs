//! TUI application state model.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

use polyglid_core::execution::{ExecutionManager, Job};
use polyglid_plugin_api::PanelLayout;
use polyglid_runtime::WasmRuntime;

use polyglid_config::AppConfig;
use polyglid_core::plugin_manager::PluginManager;

/// Which pane currently has focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    PluginTable,
    Logs,
    CommandBar,
}

impl Focus {
    /// Cycle focus forward.
    pub fn next(self) -> Self {
        match self {
            Self::PluginTable => Self::Logs,
            Self::Logs => Self::CommandBar,
            Self::CommandBar => Self::PluginTable,
        }
    }
}

/// Current interaction mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Command,
}

/// A single log entry.
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
}

/// Log severity.
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => f.write_str("INFO"),
            Self::Warn => f.write_str("WARN"),
            Self::Error => f.write_str(" ERR"),
        }
    }
}

/// The complete application state for the TUI dashboard.
pub struct App {
    pub focus: Focus,
    pub mode: Mode,
    pub input: String,
    pub selected_scan: usize,
    pub logs: Vec<LogEntry>,
    pub log_scroll: usize,
    pub show_help: bool,
    pub show_panel: bool,
    pub status: String,
    pub quit: bool,
    pub start_time: Instant,
    pub execution_manager: Arc<ExecutionManager<Arc<WasmRuntime>>>,
    pub panels: HashMap<Uuid, PanelLayout>,
    pub plugin_manager: Arc<PluginManager<WasmRuntime>>,
}

impl App {
    pub fn new() -> Self {
        let runtime = Arc::new(WasmRuntime::new());
        let config = AppConfig::load_from_env().unwrap_or_else(|_| AppConfig::development());
        let db_path = config
            .plugin_dir
            .parent()
            .unwrap_or(&config.plugin_dir)
            .join("polyglid.db");
        let store = polyglid_core::store::WorkspaceStore::new(&db_path).unwrap();
        let pm =
            Arc::new(PluginManager::new(Arc::clone(&runtime), &config, store.clone()).unwrap());
        let _ = pm.sync_directory();

        Self {
            focus: Focus::PluginTable,
            mode: Mode::Normal,
            input: String::new(),
            selected_scan: 0,
            logs: vec![LogEntry {
                level: LogLevel::Info,
                message: "PolyGlid Security Workspace initialized.".to_string(),
            }],
            log_scroll: 0,
            show_help: false,
            show_panel: false,
            status: "Ready".to_string(),
            quit: false,
            start_time: Instant::now(),
            execution_manager: Arc::new(ExecutionManager::new(runtime, Some(store))),
            panels: HashMap::new(),
            plugin_manager: pm,
        }
    }

    /// Log an info message.
    pub fn log_info(&mut self, message: impl Into<String>) {
        self.logs.push(LogEntry {
            level: LogLevel::Info,
            message: message.into(),
        });
    }

    /// Log a warning message.
    pub fn log_warn(&mut self, message: impl Into<String>) {
        self.logs.push(LogEntry {
            level: LogLevel::Warn,
            message: message.into(),
        });
    }

    /// Log an error message.
    pub fn log_error(&mut self, message: impl Into<String>) {
        self.logs.push(LogEntry {
            level: LogLevel::Error,
            message: message.into(),
        });
    }

    /// Move scan selection up.
    pub fn select_previous(&mut self) {
        if self.selected_scan > 0 {
            self.selected_scan -= 1;
        }
    }

    /// Move scan selection down.
    pub fn select_next(&mut self) {
        let jobs = self.execution_manager.get_jobs();
        if !jobs.is_empty() && self.selected_scan < jobs.len() - 1 {
            self.selected_scan += 1;
        }
    }

    /// Scroll logs up.
    pub fn scroll_logs_up(&mut self) {
        if self.log_scroll > 0 {
            self.log_scroll -= 1;
        }
    }

    /// Scroll logs down.
    pub fn scroll_logs_down(&mut self) {
        if self.log_scroll < self.logs.len().saturating_sub(1) {
            self.log_scroll += 1;
        }
    }

    /// Enter command mode.
    pub fn enter_command_mode(&mut self) {
        self.mode = Mode::Command;
        self.focus = Focus::CommandBar;
        self.input.clear();
    }

    /// Exit command mode.
    pub fn exit_command_mode(&mut self) {
        self.mode = Mode::Normal;
        self.focus = Focus::PluginTable;
        self.input.clear();
    }

    /// Toggle help overlay.
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// Toggle the plugin panel overlay for the selected scan.
    pub fn toggle_panel(&mut self) {
        let jobs = self.execution_manager.get_jobs();
        if let Some(job) = jobs.get(self.selected_scan) {
            if self.panels.contains_key(&job.id) {
                self.show_panel = !self.show_panel;
            }
        }
    }

    /// Get the currently selected job.
    pub fn selected_job(&self) -> Option<Job> {
        let jobs = self.execution_manager.get_jobs();
        jobs.get(self.selected_scan).cloned()
    }

    /// Uptime in seconds.
    pub fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}
