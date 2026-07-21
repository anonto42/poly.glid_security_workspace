//! TUI rendering — the view layer.
//!
//! All `draw_*` functions build ratatui widgets from `App` state.
//! Includes `render_panel_layout()` — the generic renderer that maps
//! `PanelLayout` widget types to ratatui widgets.

use polyglid_plugin_api::{PanelLayout, PanelWidget, WidgetKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation,
    ScrollbarState, Table, Wrap,
};
use ratatui::Frame;

use super::app::{App, Focus, LogLevel, Mode};

/// Palette constants for a sleek dark theme.
const BG: Color = Color::Rgb(15, 15, 25);
const SURFACE: Color = Color::Rgb(25, 25, 40);
const ACCENT: Color = Color::Rgb(100, 180, 255);
const ACCENT_DIM: Color = Color::Rgb(60, 110, 170);
const GREEN: Color = Color::Rgb(80, 220, 130);
const YELLOW: Color = Color::Rgb(240, 200, 80);
const RED: Color = Color::Rgb(240, 90, 90);
const TEXT: Color = Color::Rgb(200, 200, 220);
const TEXT_DIM: Color = Color::Rgb(100, 100, 130);

/// Main draw entry point — lays out all regions.
pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title bar
            Constraint::Min(8),    // Plugin/Scan table
            Constraint::Length(8), // Log pane
            Constraint::Length(3), // Command bar
            Constraint::Length(1), // Status bar
        ])
        .split(f.area());

    draw_title(f, chunks[0], app);
    draw_scan_table(f, chunks[1], app);
    draw_logs(f, chunks[2], app);
    draw_command_bar(f, chunks[3], app);
    draw_status(f, chunks[4], app);

    if app.show_help {
        draw_help(f, f.area());
    }

    if app.show_panel {
        if let Some(job) = app.selected_job() {
            if let Some(panel) = app.panels.get(&job.id) {
                let area = centered_rect(80, 80, f.area());
                f.render_widget(Clear, area);
                render_panel_layout(f, area, panel);
            }
        }
    }
}

/// Title bar with version and uptime.
fn draw_title(f: &mut Frame, area: Rect, app: &App) {
    let uptime = app.uptime_secs();
    let title = Line::from(vec![
        Span::styled(
            " ◆ PolyGlid ",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::styled("Security Workspace", Style::default().fg(TEXT)),
        Span::styled(
            concat!("  v", env!("CARGO_PKG_VERSION")),
            Style::default().fg(TEXT_DIM),
        ),
        Span::styled(
            format!("  │  uptime: {}m {}s ", uptime / 60, uptime % 60),
            Style::default().fg(TEXT_DIM),
        ),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(ACCENT_DIM))
        .style(Style::default().bg(SURFACE));

    let paragraph = Paragraph::new(title).block(block);
    f.render_widget(paragraph, area);
}

/// Plugin/Scan results table.
fn draw_scan_table(f: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == Focus::PluginTable;
    let border_color = if focused { ACCENT } else { ACCENT_DIM };

    let header = Row::new(vec![
        Cell::from("Plugin").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Target").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Status").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Cell::from("Issues").style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
    ])
    .height(1);

    let jobs = app.execution_manager.get_jobs();
    let rows: Vec<Row> = if jobs.is_empty() {
        vec![Row::new(vec![
            Cell::from("No scans yet").style(Style::default().fg(TEXT_DIM)),
            Cell::from("").style(Style::default().fg(TEXT_DIM)),
            Cell::from("").style(Style::default().fg(TEXT_DIM)),
            Cell::from("Press : to run").style(Style::default().fg(TEXT_DIM)),
        ])]
    } else {
        jobs.iter()
            .enumerate()
            .map(|(i, job)| {
                let selected = i == app.selected_scan;
                let style = if selected {
                    Style::default().fg(TEXT).bg(Color::Rgb(40, 40, 65))
                } else {
                    Style::default().fg(TEXT)
                };
                let status_style = match &job.state {
                    polyglid_core::execution::JobState::Completed => Style::default().fg(GREEN),
                    polyglid_core::execution::JobState::Running
                    | polyglid_core::execution::JobState::Starting => Style::default().fg(YELLOW),
                    polyglid_core::execution::JobState::Failed
                    | polyglid_core::execution::JobState::TimedOut => Style::default().fg(RED),
                    polyglid_core::execution::JobState::Cancelled => Style::default().fg(TEXT_DIM),
                    polyglid_core::execution::JobState::Queued => Style::default().fg(TEXT_DIM),
                };
                let status_text = match &job.state {
                    polyglid_core::execution::JobState::Completed => "✓ Done".to_string(),
                    polyglid_core::execution::JobState::Running => "Running".to_string(),
                    polyglid_core::execution::JobState::Starting => "Starting".to_string(),
                    polyglid_core::execution::JobState::Failed => "✗ Failed".to_string(),
                    polyglid_core::execution::JobState::TimedOut => "✗ Timeout".to_string(),
                    polyglid_core::execution::JobState::Cancelled => "Cancelled".to_string(),
                    polyglid_core::execution::JobState::Queued => "Queued".to_string(),
                };
                let issues_text = match &job.report {
                    Some(report) => report.issues.len().to_string(),
                    None => "-".to_string(),
                };
                Row::new(vec![
                    Cell::from(job.plugin_path.clone()).style(style),
                    Cell::from(job.target.clone()).style(style),
                    Cell::from(status_text).style(status_style),
                    Cell::from(issues_text).style(style),
                ])
            })
            .collect()
    };

    let widths = [
        Constraint::Percentage(30),
        Constraint::Percentage(30),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(Span::styled(
                    " Scans ",
                    Style::default()
                        .fg(border_color)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color))
                .style(Style::default().bg(BG)),
        )
        .row_highlight_style(Style::default().bg(Color::Rgb(40, 40, 65)));

    f.render_widget(table, area);
}

/// Scrollable log pane.
fn draw_logs(f: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == Focus::Logs;
    let border_color = if focused { ACCENT } else { ACCENT_DIM };

    let log_lines: Vec<Line> = app
        .logs
        .iter()
        .skip(app.log_scroll)
        .map(|entry| {
            let (level_color, level_str) = match entry.level {
                LogLevel::Info => (ACCENT_DIM, "INFO"),
                LogLevel::Warn => (YELLOW, "WARN"),
                LogLevel::Error => (RED, " ERR"),
            };
            Line::from(vec![
                Span::styled(format!("[{level_str}] "), Style::default().fg(level_color)),
                Span::styled(&entry.message, Style::default().fg(TEXT_DIM)),
            ])
        })
        .collect();

    let block = Block::default()
        .title(Span::styled(
            format!(" Logs ({}) ", app.logs.len()),
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(BG));

    let paragraph = Paragraph::new(log_lines)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);

    // Scrollbar
    if app.logs.len() > area.height as usize {
        let mut scrollbar_state = ScrollbarState::new(app.logs.len()).position(app.log_scroll);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .style(Style::default().fg(ACCENT_DIM)),
            area,
            &mut scrollbar_state,
        );
    }
}

/// Command bar (input field).
fn draw_command_bar(f: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == Focus::CommandBar || app.mode == Mode::Command;
    let border_color = if focused { ACCENT } else { ACCENT_DIM };

    let content = if app.mode == Mode::Command {
        Line::from(vec![
            Span::styled(
                ":",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(&app.input, Style::default().fg(TEXT)),
            Span::styled("█", Style::default().fg(ACCENT)),
        ])
    } else {
        Line::from(Span::styled(
            " Press : to enter command mode  │  ? for help  │  q to quit",
            Style::default().fg(TEXT_DIM),
        ))
    };

    let block = Block::default()
        .title(Span::styled(
            " Command ",
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(SURFACE));

    let paragraph = Paragraph::new(content).block(block);
    f.render_widget(paragraph, area);
}

/// Bottom status bar.
fn draw_status(f: &mut Frame, area: Rect, app: &App) {
    let jobs = app.execution_manager.get_jobs();
    let scan_count = jobs.len();
    let done_count = jobs
        .iter()
        .filter(|s| matches!(s.state, polyglid_core::execution::JobState::Completed))
        .count();

    let line = Line::from(vec![
        Span::styled(" Engine: ", Style::default().fg(TEXT_DIM)),
        Span::styled("Wasmtime 46", Style::default().fg(ACCENT_DIM)),
        Span::styled(" │ ", Style::default().fg(TEXT_DIM)),
        Span::styled(
            format!("Scans: {done_count}/{scan_count}"),
            Style::default().fg(GREEN),
        ),
        Span::styled(" │ ", Style::default().fg(TEXT_DIM)),
        Span::styled(&app.status, Style::default().fg(TEXT)),
    ]);

    let paragraph = Paragraph::new(line).style(Style::default().bg(SURFACE));
    f.render_widget(paragraph, area);
}

/// Help overlay (modal popup).
fn draw_help(f: &mut Frame, area: Rect) {
    let popup = centered_rect(60, 60, area);
    f.render_widget(Clear, popup);

    let help_text = vec![
        Line::from(Span::styled(
            "Keyboard Shortcuts",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Tab       ", Style::default().fg(ACCENT)),
            Span::styled("Cycle focus between panes", Style::default().fg(TEXT)),
        ]),
        Line::from(vec![
            Span::styled("  ↑/↓       ", Style::default().fg(ACCENT)),
            Span::styled(
                "Navigate scan table / scroll logs",
                Style::default().fg(TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Enter     ", Style::default().fg(ACCENT)),
            Span::styled(
                "View selected scan's plugin panel",
                Style::default().fg(TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled("  :         ", Style::default().fg(ACCENT)),
            Span::styled("Enter command mode", Style::default().fg(TEXT)),
        ]),
        Line::from(vec![
            Span::styled("  ?         ", Style::default().fg(ACCENT)),
            Span::styled("Toggle this help", Style::default().fg(TEXT)),
        ]),
        Line::from(vec![
            Span::styled("  q         ", Style::default().fg(ACCENT)),
            Span::styled("Quit", Style::default().fg(TEXT)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Commands",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  :run ", Style::default().fg(GREEN)),
            Span::styled(
                "<plugin.wasm> --target <domain>",
                Style::default().fg(TEXT_DIM),
            ),
        ]),
        Line::from(vec![
            Span::styled("  :inspect ", Style::default().fg(GREEN)),
            Span::styled("<plugin.wasm>", Style::default().fg(TEXT_DIM)),
        ]),
        Line::from(vec![Span::styled("  :quit", Style::default().fg(GREEN))]),
        Line::from(""),
        Line::from(Span::styled(
            "  Press ? or Esc to close",
            Style::default().fg(TEXT_DIM),
        )),
    ];

    let block = Block::default()
        .title(Span::styled(
            " Help ",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(ACCENT))
        .style(Style::default().bg(Color::Rgb(20, 20, 35)));

    let paragraph = Paragraph::new(help_text).block(block);
    f.render_widget(paragraph, popup);
}

/// Generic panel layout renderer.
/// Maps `PanelLayout` widget types from the WASM plugin to native ratatui widgets.
pub fn render_panel_layout(f: &mut Frame, area: Rect, layout: &PanelLayout) {
    let widget_count = layout.widgets.len();
    let constraints: Vec<Constraint> = if widget_count == 0 {
        vec![Constraint::Min(1)]
    } else {
        layout.widgets.iter().map(|_| Constraint::Min(4)).collect()
    };

    let outer_block = Block::default()
        .title(Span::styled(
            format!(" {} ", layout.title),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(ACCENT))
        .style(Style::default().bg(Color::Rgb(20, 20, 35)));

    let inner = outer_block.inner(area);
    f.render_widget(outer_block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    for (i, widget) in layout.widgets.iter().enumerate() {
        if i >= chunks.len() {
            break;
        }
        render_widget(f, chunks[i], widget);
    }
}

/// Render a single `PanelWidget` based on its kind.
fn render_widget(f: &mut Frame, area: Rect, widget: &PanelWidget) {
    let block = Block::default()
        .title(Span::styled(
            format!(" {} ", widget.title),
            Style::default().fg(ACCENT_DIM).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(40, 40, 60)));

    match widget.widget_kind {
        WidgetKind::Table => {
            let (header_row, data_rows) = if widget.data.is_empty() {
                (vec![], vec![])
            } else {
                let headers: Vec<Cell> = widget.data[0]
                    .iter()
                    .map(|h| Cell::from(h.as_str()).style(Style::default().fg(ACCENT).bold()))
                    .collect();
                let rows: Vec<Row> = widget.data[1..]
                    .iter()
                    .map(|row| {
                        Row::new(
                            row.iter()
                                .map(|cell| {
                                    Cell::from(cell.as_str()).style(Style::default().fg(TEXT))
                                })
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect();
                (headers, rows)
            };

            let widths: Vec<Constraint> = if header_row.is_empty() {
                vec![Constraint::Percentage(100)]
            } else {
                header_row
                    .iter()
                    .map(|_| Constraint::Ratio(1, header_row.len() as u32))
                    .collect()
            };

            let table = Table::new(data_rows, widths)
                .header(Row::new(header_row))
                .block(block);
            f.render_widget(table, area);
        }
        WidgetKind::KeyValue => {
            let lines: Vec<Line> = widget
                .data
                .iter()
                .map(|row| {
                    if row.len() >= 2 {
                        Line::from(vec![
                            Span::styled(
                                format!("  {}: ", row[0]),
                                Style::default().fg(ACCENT_DIM),
                            ),
                            Span::styled(&row[1], Style::default().fg(TEXT)),
                        ])
                    } else if !row.is_empty() {
                        Line::from(Span::styled(&row[0], Style::default().fg(TEXT)))
                    } else {
                        Line::from("")
                    }
                })
                .collect();
            let paragraph = Paragraph::new(lines)
                .block(block)
                .wrap(Wrap { trim: false });
            f.render_widget(paragraph, area);
        }
        WidgetKind::TextBlock | WidgetKind::Log | WidgetKind::Tree => {
            let lines: Vec<Line> = widget
                .data
                .iter()
                .map(|row| {
                    let text = row.join(" ");
                    Line::from(Span::styled(
                        format!("  {text}"),
                        Style::default().fg(TEXT_DIM),
                    ))
                })
                .collect();
            let paragraph = Paragraph::new(lines)
                .block(block)
                .wrap(Wrap { trim: false });
            f.render_widget(paragraph, area);
        }
        WidgetKind::ChartBar => {
            // Simple text-based bar chart.
            let lines: Vec<Line> = widget
                .data
                .iter()
                .map(|row| {
                    if row.len() >= 2 {
                        let label = &row[0];
                        let value: usize = row[1].parse().unwrap_or(0);
                        let bar = "█".repeat(value.min(40));
                        Line::from(vec![
                            Span::styled(
                                format!("  {label:>15} "),
                                Style::default().fg(ACCENT_DIM),
                            ),
                            Span::styled(bar, Style::default().fg(ACCENT)),
                            Span::styled(format!(" {value}"), Style::default().fg(TEXT_DIM)),
                        ])
                    } else {
                        Line::from("")
                    }
                })
                .collect();
            let paragraph = Paragraph::new(lines).block(block);
            f.render_widget(paragraph, area);
        }
    }
}

/// Helper: create a centered rectangle.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
