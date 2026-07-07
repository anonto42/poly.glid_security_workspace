//! TUI entrypoint and event loop orchestration.

pub mod app;
pub mod keys;
pub mod ui;

use std::io::stdout;
use std::time::Duration;

use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use polyglid_core::execution::ExecutionEvent;
use polyglid_runtime::WasmRuntime;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm;
use ratatui::Terminal;

use self::app::App;

/// Run the TUI interactive dashboard.
pub fn event_loop() -> Result<(), String> {
    // Setup terminal
    enable_raw_mode().map_err(|err| format!("failed to enable raw mode: {err}"))?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)
        .map_err(|err| format!("failed to enter alternate screen: {err}"))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal =
        Terminal::new(backend).map_err(|err| format!("failed to create terminal: {err}"))?;

    let mut app = App::new();

    // Wire up events
    let mut event_rx = app.execution_manager.subscribe();
    let result = run_loop(&mut terminal, &mut app, &mut event_rx);

    // Restore terminal
    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    result
}

fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    event_rx: &mut tokio::sync::broadcast::Receiver<ExecutionEvent>,
) -> Result<(), String> {
    loop {
        // Process background execution events
        while let Ok(event) = event_rx.try_recv() {
            match event {
                ExecutionEvent::JobStateChanged { job_id, state } => {
                    app.log_info(format!("[Job] {} -> {:?}", &job_id.to_string()[..8], state));
                }
                ExecutionEvent::JobFinished {
                    job_id,
                    report,
                    metrics,
                } => {
                    app.log_info(format!(
                        "[Job] {} finished in {}ms: {}",
                        &job_id.to_string()[..8],
                        metrics.duration.as_millis(),
                        report.summary
                    ));
                    // Construct a dynamic PanelLayout for the finished job in TUI
                    let runtime = WasmRuntime::new();
                    let jobs = app.execution_manager.get_jobs();
                    if let Some(job) = jobs.iter().find(|j| j.id == job_id) {
                        let plugin_ref = polyglid_core::PluginRef::from_path(
                            std::path::PathBuf::from(&job.plugin_path),
                        );
                        if let Ok(panel) = runtime.call_cli_panel(&plugin_ref, &report) {
                            app.panels.insert(job_id, panel);
                        }
                    }
                }
                ExecutionEvent::JobFailed {
                    job_id,
                    error,
                    metrics,
                } => {
                    let dur_str = if let Some(m) = metrics {
                        format!(" in {}ms", m.duration.as_millis())
                    } else {
                        "".to_string()
                    };
                    app.log_error(format!(
                        "[Job] {} failed{}: {}",
                        &job_id.to_string()[..8],
                        dur_str,
                        error
                    ));
                }
                ExecutionEvent::JobLog { job_id, message } => {
                    app.log_info(format!("[Job {}] {}", &job_id.to_string()[..8], message));
                }
            }
        }

        // Redraw screen
        terminal
            .draw(|f| ui::draw(f, app))
            .map_err(|err| format!("failed to draw frame: {err}"))?;

        // Check for quit
        if app.quit {
            break;
        }

        // Poll for inputs, updating every 250ms for clock tick
        if event::poll(Duration::from_millis(250))
            .map_err(|err| format!("failed to poll event: {err}"))?
        {
            if let Event::Key(key) =
                event::read().map_err(|err| format!("failed to read event: {err}"))?
            {
                keys::handle_key(app, key);
            }
        }
    }

    Ok(())
}
