use gpui::{div, prelude::*, px, Context, Div, FocusHandle, IntoElement, Render, Window};
use polyglid_client::{
    client::{BootstrapSnapshot, WorkspaceEntry},
    controllers::DesktopControllers,
};

use crate::{
    actions::*,
    models::{WorkbenchState, WorkspaceView},
    theme,
};

pub struct Workbench {
    controllers: DesktopControllers,
    snapshot: BootstrapSnapshot,
    explorer_entries: Vec<WorkspaceEntry>,
    state: WorkbenchState,
    focus_handle: FocusHandle,
}

impl Workbench {
    fn workspace_label(&self) -> String {
        std::path::Path::new(&self.snapshot.active_workspace.root_path)
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .unwrap_or(&self.snapshot.active_workspace.name)
            .to_owned()
    }

    pub fn new(
        controllers: DesktopControllers,
        snapshot: BootstrapSnapshot,
        cx: &mut Context<Self>,
    ) -> Self {
        let state = WorkbenchState::new(&snapshot.shell);
        let explorer_entries = controllers
            .documents
            .list_directory(&snapshot.active_workspace.id, "")
            .unwrap_or_default();
        Self {
            controllers,
            snapshot,
            explorer_entries,
            state,
            focus_handle: cx.focus_handle(),
        }
    }

    fn activate(&mut self, view: WorkspaceView, cx: &mut Context<Self>) {
        self.state.activate(view);
        self.state.status = format!("{} selected", view.title());
        cx.notify();
    }

    fn reload(&mut self) {
        match self.controllers.application.bootstrap() {
            Ok(snapshot) => {
                match self
                    .controllers
                    .documents
                    .list_directory(&snapshot.active_workspace.id, "")
                {
                    Ok(entries) => {
                        self.explorer_entries = entries;
                        self.state.status = "Workspace refreshed".to_owned();
                    }
                    Err(error) => {
                        self.explorer_entries.clear();
                        self.state.status = format!("Explorer unavailable: {error}");
                    }
                }
                self.snapshot = snapshot;
            }
            Err(error) => self.state.status = format!("Refresh failed: {error}"),
        }
    }

    fn open_projects(&mut self, _: &OpenProjects, _: &mut Window, cx: &mut Context<Self>) {
        self.activate(WorkspaceView::Projects, cx);
    }

    fn open_scanner(&mut self, _: &OpenScanner, _: &mut Window, cx: &mut Context<Self>) {
        self.activate(WorkspaceView::Scanner, cx);
    }

    fn open_executions(&mut self, _: &OpenExecutions, _: &mut Window, cx: &mut Context<Self>) {
        self.activate(WorkspaceView::Executions, cx);
    }

    fn open_reports(&mut self, _: &OpenReports, _: &mut Window, cx: &mut Context<Self>) {
        self.activate(WorkspaceView::Reports, cx);
    }

    fn open_plugins(&mut self, _: &OpenPlugins, _: &mut Window, cx: &mut Context<Self>) {
        self.activate(WorkspaceView::Plugins, cx);
    }

    fn open_palette(&mut self, _: &OpenCommandPalette, _: &mut Window, cx: &mut Context<Self>) {
        self.state.command_palette_visible = true;
        cx.notify();
    }

    fn close_overlay(&mut self, _: &CloseOverlay, _: &mut Window, cx: &mut Context<Self>) {
        self.state.command_palette_visible = false;
        cx.notify();
    }

    fn toggle_sidebar(&mut self, _: &ToggleSidebar, _: &mut Window, cx: &mut Context<Self>) {
        self.state.sidebar_visible = !self.state.sidebar_visible;
        cx.notify();
    }

    fn toggle_bottom_panel(
        &mut self,
        _: &ToggleBottomPanel,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.state.bottom_panel_visible = !self.state.bottom_panel_visible;
        cx.notify();
    }

    fn close_active_tab(&mut self, _: &CloseActiveTab, _: &mut Window, cx: &mut Context<Self>) {
        self.state.close_active();
        cx.notify();
    }

    fn next_tab(&mut self, _: &NextTab, _: &mut Window, cx: &mut Context<Self>) {
        self.state.next_tab();
        cx.notify();
    }

    fn refresh(&mut self, _: &RefreshWorkspace, _: &mut Window, cx: &mut Context<Self>) {
        self.reload();
        cx.notify();
    }

    fn top_chip(label: impl Into<String>) -> Div {
        div()
            .h(px(24.))
            .flex()
            .items_center()
            .px_2()
            .rounded_sm()
            .bg(theme::CHIP)
            .border_1()
            .border_color(theme::CHIP_BORDER)
            .text_xs()
            .text_color(theme::MUTED)
            .child(label.into())
    }

    fn top_bar(&self, cx: &mut Context<Self>) -> Div {
        let workspace_name = self.workspace_label();
        div()
            .h(px(31.))
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .bg(theme::TOP_BAR)
            .border_b_1()
            .border_color(theme::DARK_BORDER)
            .child(
                div()
                    .h_full()
                    .flex()
                    .items_center()
                    .gap_1()
                    .px_1()
                    .child(
                        div()
                            .id("menu")
                            .size(px(24.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_sm()
                            .cursor_pointer()
                            .text_sm()
                            .text_color(theme::MUTED)
                            .hover(|style| style.bg(theme::CHIP).text_color(theme::TEXT))
                            .child("☰")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.state.sidebar_visible = !this.state.sidebar_visible;
                                cx.notify();
                            })),
                    )
                    .child(Self::top_chip(workspace_name))
                    .child(Self::top_chip("↪  main"))
                    .child(Self::top_chip("⑂  main")),
            )
            .child(
                div()
                    .h_full()
                    .flex()
                    .items_center()
                    .gap_3()
                    .px_3()
                    .text_xs()
                    .text_color(theme::MUTED)
                    .child(
                        div()
                            .size(px(19.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_full()
                            .bg(theme::CHIP)
                            .child("P"),
                    )
                    .child("—")
                    .child("□")
                    .child("×"),
            )
    }

    fn entry_color(entry: &WorkspaceEntry) -> gpui::Rgba {
        if entry.is_directory {
            return if entry.name == "crates" {
                theme::GREEN
            } else if entry.name == "docs" {
                theme::ORANGE
            } else if entry.name == "apps" {
                theme::PINK
            } else {
                theme::EXPLORER_TEXT
            };
        }
        match entry.name.rsplit('.').next().unwrap_or_default() {
            "rs" => theme::ORANGE,
            "md" => theme::BLUE,
            "toml" | "lock" => theme::ORANGE,
            "json" => theme::YELLOW,
            _ => theme::MUTED,
        }
    }

    fn explorer(&self, cx: &mut Context<Self>) -> Div {
        let workspace_name = self.workspace_label();
        let is_empty = self.explorer_entries.is_empty();
        div()
            .w(px(268.))
            .h_full()
            .flex()
            .flex_col()
            .bg(theme::EXPLORER)
            .border_l_1()
            .border_color(theme::DARK_BORDER)
            .child(
                div()
                    .h(px(33.))
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .text_xs()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(theme::PINK)
                    .child(div().text_color(theme::EXPLORER_TEXT).child("▱"))
                    .child(workspace_name),
            )
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .px_2()
                    .children(self.explorer_entries.iter().cloned().enumerate().map(
                        |(index, entry)| {
                            let color = Self::entry_color(&entry);
                            let label = entry.name.clone();
                            div()
                                .id(("explorer-entry", index))
                                .h(px(26.))
                                .w_full()
                                .flex()
                                .items_center()
                                .gap_2()
                                .px_1()
                                .rounded_sm()
                                .cursor_pointer()
                                .text_xs()
                                .text_color(color)
                                .hover(|style| style.bg(theme::CHIP))
                                .child(if entry.is_directory { "▱" } else { "◇" })
                                .child(label)
                        },
                    ))
                    .when(is_empty, |explorer| {
                        explorer.child(
                            div()
                                .p_3()
                                .text_xs()
                                .text_color(theme::MUTED)
                                .child("No entries in this workspace")
                                .child(
                                    div()
                                        .id("refresh-empty-explorer")
                                        .mt_2()
                                        .cursor_pointer()
                                        .text_color(theme::BLUE)
                                        .child("Refresh")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.reload();
                                            cx.notify();
                                        })),
                                ),
                        )
                    }),
            )
    }

    fn editor_canvas(&self) -> Div {
        div()
            .flex_1()
            .h_full()
            .bg(theme::EDITOR)
            .overflow_hidden()
            .when(self.state.bottom_panel_visible, |canvas| {
                canvas.child(
                    div()
                        .absolute()
                        .bottom(px(28.))
                        .left(px(12.))
                        .right(px(12.))
                        .h(px(180.))
                        .p_3()
                        .bg(theme::EXPLORER)
                        .border_1()
                        .border_color(theme::DARK_BORDER)
                        .text_xs()
                        .text_color(theme::MUTED)
                        .child(format!(
                            "{} · {} executions · {} reports",
                            self.state.status,
                            self.snapshot.executions.len(),
                            self.snapshot.reports.len()
                        )),
                )
            })
    }

    fn utility_button(
        id: &'static str,
        glyph: &'static str,
        active: bool,
        cx: &mut Context<Self>,
        on_click: impl Fn(&mut Self, &mut Context<Self>) + 'static,
    ) -> impl IntoElement {
        div()
            .id(id)
            .h_full()
            .min_w(px(26.))
            .flex()
            .items_center()
            .justify_center()
            .px_1()
            .cursor_pointer()
            .text_xs()
            .text_color(if active { theme::BLUE } else { theme::MUTED })
            .bg(if active {
                theme::CHIP
            } else {
                theme::BOTTOM_BAR
            })
            .hover(|style| style.bg(theme::CHIP).text_color(theme::TEXT))
            .child(glyph)
            .on_click(cx.listener(move |this, _, _, cx| on_click(this, cx)))
    }

    fn bottom_bar(&self, cx: &mut Context<Self>) -> Div {
        div()
            .h(px(27.))
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .bg(theme::BOTTOM_BAR)
            .border_t_1()
            .border_color(theme::DARK_BORDER)
            .child(
                div()
                    .h_full()
                    .flex()
                    .items_center()
                    .child(Self::utility_button(
                        "toggle-explorer",
                        "▣",
                        self.state.sidebar_visible,
                        cx,
                        |this, cx| {
                            this.state.sidebar_visible = !this.state.sidebar_visible;
                            cx.notify();
                        },
                    ))
                    .child(Self::utility_button(
                        "settings",
                        "⚙",
                        false,
                        cx,
                        |this, cx| {
                            this.state.status = "Settings are mapped for the next milestone".into();
                            cx.notify();
                        },
                    ))
                    .child(Self::utility_button(
                        "search",
                        "⌕",
                        self.state.command_palette_visible,
                        cx,
                        |this, cx| {
                            this.state.command_palette_visible = true;
                            cx.notify();
                        },
                    ))
                    .child(Self::utility_button(
                        "refresh",
                        "✓",
                        false,
                        cx,
                        |this, cx| {
                            this.reload();
                            cx.notify();
                        },
                    )),
            )
            .child(
                div()
                    .h_full()
                    .flex()
                    .items_center()
                    .children(
                        WorkspaceView::ALL
                            .into_iter()
                            .enumerate()
                            .map(|(index, view)| {
                                let active = self.state.active_view == view;
                                div()
                                    .id(("product-view", index))
                                    .h_full()
                                    .min_w(px(28.))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .cursor_pointer()
                                    .text_xs()
                                    .text_color(if active { theme::BLUE } else { theme::MUTED })
                                    .hover(|style| style.bg(theme::CHIP).text_color(theme::TEXT))
                                    .child(view.glyph())
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.activate(view, cx);
                                    }))
                            }),
                    )
                    .child(Self::utility_button(
                        "toggle-panel",
                        "▤",
                        self.state.bottom_panel_visible,
                        cx,
                        |this, cx| {
                            this.state.bottom_panel_visible = !this.state.bottom_panel_visible;
                            cx.notify();
                        },
                    ))
                    .child(Self::utility_button(
                        "branch-state",
                        "⑂",
                        false,
                        cx,
                        |this, cx| {
                            this.state.status =
                                "Source control service is not connected yet".into();
                            cx.notify();
                        },
                    )),
            )
    }

    fn command_palette(&self, cx: &mut Context<Self>) -> Div {
        let commands = [
            (WorkspaceView::Projects, "Open Projects", "Ctrl 1"),
            (WorkspaceView::Scanner, "Open Scanner", "Ctrl 2"),
            (WorkspaceView::Executions, "Open Executions", "Ctrl 3"),
            (WorkspaceView::Reports, "Open Reports", "Ctrl 4"),
            (WorkspaceView::Plugins, "Open Plugins", "Ctrl 5"),
        ];
        div()
            .absolute()
            .top(px(64.))
            .left(px(320.))
            .w(px(560.))
            .p_2()
            .rounded_md()
            .bg(theme::EXPLORER)
            .border_1()
            .border_color(theme::CHIP_BORDER)
            .shadow_lg()
            .child(
                div()
                    .h(px(40.))
                    .flex()
                    .items_center()
                    .px_3()
                    .rounded_sm()
                    .bg(theme::TOP_BAR)
                    .text_sm()
                    .text_color(theme::MUTED)
                    .child("Search commands…"),
            )
            .child(
                div()
                    .mt_2()
                    .flex()
                    .flex_col()
                    .children(commands.into_iter().enumerate().map(
                        |(index, (view, label, shortcut))| {
                            div()
                                .id(("palette-command", index))
                                .flex()
                                .items_center()
                                .justify_between()
                                .px_3()
                                .py_2()
                                .rounded_sm()
                                .cursor_pointer()
                                .text_sm()
                                .text_color(theme::TEXT)
                                .hover(|style| style.bg(theme::CHIP))
                                .child(label)
                                .child(div().text_xs().text_color(theme::MUTED).child(shortcut))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.activate(view, cx);
                                }))
                        },
                    )),
            )
    }
}

impl Render for Workbench {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("polyglid-workbench")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::open_palette))
            .on_action(cx.listener(Self::close_overlay))
            .on_action(cx.listener(Self::toggle_sidebar))
            .on_action(cx.listener(Self::toggle_bottom_panel))
            .on_action(cx.listener(Self::open_projects))
            .on_action(cx.listener(Self::open_scanner))
            .on_action(cx.listener(Self::open_executions))
            .on_action(cx.listener(Self::open_reports))
            .on_action(cx.listener(Self::open_plugins))
            .on_action(cx.listener(Self::close_active_tab))
            .on_action(cx.listener(Self::next_tab))
            .on_action(cx.listener(Self::refresh))
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(theme::EDITOR)
            .text_color(theme::TEXT)
            .child(self.top_bar(cx))
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .flex()
                    .overflow_hidden()
                    .child(self.editor_canvas())
                    .when(self.state.sidebar_visible, |body| {
                        body.child(self.explorer(cx))
                    }),
            )
            .child(self.bottom_bar(cx))
            .when(self.state.command_palette_visible, |root| {
                root.child(self.command_palette(cx))
            })
    }
}
