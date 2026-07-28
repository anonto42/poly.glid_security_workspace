use polyglid_client::client::ShellPreferences;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceView {
    Projects,
    Scanner,
    Executions,
    Reports,
    Plugins,
}

impl WorkspaceView {
    pub const ALL: [Self; 5] = [
        Self::Projects,
        Self::Scanner,
        Self::Executions,
        Self::Reports,
        Self::Plugins,
    ];

    pub const fn title(self) -> &'static str {
        match self {
            Self::Projects => "Projects",
            Self::Scanner => "Scanner",
            Self::Executions => "Executions",
            Self::Reports => "Reports",
            Self::Plugins => "Plugins",
        }
    }

    pub const fn glyph(self) -> &'static str {
        match self {
            Self::Projects => "P",
            Self::Scanner => "S",
            Self::Executions => "E",
            Self::Reports => "R",
            Self::Plugins => "X",
        }
    }
}

#[derive(Clone, Debug)]
pub struct WorkbenchState {
    pub active_view: WorkspaceView,
    pub open_views: Vec<WorkspaceView>,
    pub sidebar_visible: bool,
    pub bottom_panel_visible: bool,
    pub command_palette_visible: bool,
    pub status: String,
}

impl WorkbenchState {
    pub fn new(shell: &ShellPreferences) -> Self {
        Self {
            active_view: WorkspaceView::Projects,
            open_views: vec![WorkspaceView::Projects],
            sidebar_visible: shell.sidebar_visible,
            bottom_panel_visible: false,
            command_palette_visible: false,
            status: "Ready".to_owned(),
        }
    }

    pub fn activate(&mut self, view: WorkspaceView) {
        if !self.open_views.contains(&view) {
            self.open_views.push(view);
        }
        self.active_view = view;
        self.command_palette_visible = false;
    }

    pub fn close_active(&mut self) {
        if self.open_views.len() == 1 {
            return;
        }
        if let Some(index) = self
            .open_views
            .iter()
            .position(|view| *view == self.active_view)
        {
            self.open_views.remove(index);
            self.active_view = self.open_views[index.saturating_sub(1)];
        }
    }

    pub fn next_tab(&mut self) {
        let current = self
            .open_views
            .iter()
            .position(|view| *view == self.active_view)
            .unwrap_or(0);
        self.active_view = self.open_views[(current + 1) % self.open_views.len()];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> WorkbenchState {
        WorkbenchState::new(&ShellPreferences::default())
    }

    #[test]
    fn activating_a_view_opens_and_selects_it() {
        let mut state = state();
        state.activate(WorkspaceView::Reports);
        assert_eq!(state.active_view, WorkspaceView::Reports);
        assert_eq!(
            state.open_views,
            vec![WorkspaceView::Projects, WorkspaceView::Reports]
        );
    }

    #[test]
    fn the_last_tab_stays_open() {
        let mut state = state();
        state.close_active();
        assert_eq!(state.open_views, vec![WorkspaceView::Projects]);
    }

    #[test]
    fn next_tab_wraps_around() {
        let mut state = state();
        state.activate(WorkspaceView::Scanner);
        state.next_tab();
        assert_eq!(state.active_view, WorkspaceView::Projects);
    }
}
