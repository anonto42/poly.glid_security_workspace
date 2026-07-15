use polyglid_desktop::TaskStatus;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorkspaceView {
    Projects,
    Explorer,
    Plugins,
    Tracks,
    Automation,
    Agents,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TopBarAction {
    pub(crate) id: &'static str,
    pub(crate) icon: &'static str,
    pub(crate) label: &'static str,
    pub(crate) source: &'static str,
    pub(crate) destination: WorkspaceView,
}

impl WorkspaceView {
    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Projects => "My Projects",
            Self::Explorer => "Explorer",
            Self::Plugins => "Plugins",
            Self::Tracks => "Work tracks",
            Self::Automation => "Automation",
            Self::Agents => "AI agents",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorkspaceLoadState {
    Loading,
    Empty,
    Ready,
    Error(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EditorTab {
    Scanner,
    Result,
    Source,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BottomTab {
    Problems,
    Output,
    Terminal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SettingsTab {
    Overview,
    Engine,
    Plugins,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TrackFilter {
    All,
    Active,
    Planned,
    Complete,
}

impl TrackFilter {
    pub(crate) const ALL: [Self; 4] = [Self::All, Self::Active, Self::Planned, Self::Complete];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Active => "active",
            Self::Planned => "planned",
            Self::Complete => "complete",
        }
    }

    pub(crate) fn matches(self, status: TaskStatus) -> bool {
        match self {
            Self::All => true,
            Self::Active => matches!(status, TaskStatus::InProgress | TaskStatus::Review),
            Self::Planned => matches!(status, TaskStatus::Draft | TaskStatus::Ready),
            Self::Complete => matches!(status, TaskStatus::Verified | TaskStatus::Done),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PluginCard {
    pub(crate) id: &'static str,
    pub(crate) name: &'static str,
    pub(crate) version: &'static str,
    pub(crate) description: &'static str,
    pub(crate) capabilities: Vec<&'static str>,
    pub(crate) enabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Finding {
    pub(crate) severity: &'static str,
    pub(crate) title: &'static str,
    pub(crate) description: &'static str,
    pub(crate) recommendation: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ScanReport {
    pub(crate) target: String,
    pub(crate) summary: &'static str,
    pub(crate) findings: Vec<Finding>,
}
