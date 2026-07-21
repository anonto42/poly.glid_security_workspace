use polyglid_desktop::client::{
    CapabilityKind, CapabilityRequest, ExecutionState, PluginInspection,
};

/// Product areas exposed by the desktop client.
///
/// Every entry maps to a working local-client capability. Preview-only areas stay
/// out of this enum so they cannot accidentally appear in navigation or commands.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorkspaceView {
    Projects,
    Scanner,
    Executions,
    Reports,
    Plugins,
}

impl WorkspaceView {
    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Projects => "Projects",
            Self::Scanner => "New scan",
            Self::Executions => "Executions",
            Self::Reports => "Reports",
            Self::Plugins => "Plugins",
        }
    }

    pub(crate) fn icon(self) -> &'static str {
        match self {
            Self::Projects => "▦",
            Self::Scanner => "⌕",
            Self::Executions => "▷",
            Self::Reports => "▥",
            Self::Plugins => "◇",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ResizeAxis {
    Sidebar,
    BottomPanel,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum LoadState {
    Loading,
    Empty,
    Ready,
    Error(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BottomTab {
    Findings,
    Activity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SettingsTab {
    Overview,
    Execution,
    Plugins,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Overlay {
    Settings,
    Commands,
    PluginInstall(PendingPluginInstall),
    PermissionReview(PermissionReview),
    Error(DialogError),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PendingPluginInstall {
    pub(crate) path: String,
    pub(crate) plugin: PluginInspection,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PermissionReview {
    pub(crate) plugin_id: String,
    pub(crate) plugin_name: String,
    pub(crate) target: String,
    pub(crate) requested: Vec<CapabilityRequest>,
    pub(crate) approved: Vec<CapabilityKind>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DialogError {
    pub(crate) title: String,
    pub(crate) message: String,
}

pub(crate) fn capability_explanation(capability: CapabilityKind) -> &'static str {
    match capability {
        CapabilityKind::NetworkConnect => "Open outbound network connections.",
        CapabilityKind::NetworkListen => "Listen for inbound network connections.",
        CapabilityKind::FilesystemRead => "Read files available to the sandbox.",
        CapabilityKind::FilesystemWrite => "Write files available to the sandbox.",
        CapabilityKind::ConfigRead => "Read approved PolyGlid configuration.",
        CapabilityKind::ReportWrite => "Create a persisted analysis report.",
        CapabilityKind::Crypto => "Use cryptographic operations.",
        CapabilityKind::DnsResolve => "Resolve domain names for the selected target.",
        CapabilityKind::ProcessSpawn => "Start an operating-system process.",
        CapabilityKind::EnvironmentRead => "Read approved environment variables.",
    }
}

pub(crate) fn capability_risk(capability: CapabilityKind) -> &'static str {
    match capability {
        CapabilityKind::ProcessSpawn
        | CapabilityKind::FilesystemWrite
        | CapabilityKind::NetworkListen => "Elevated",
        CapabilityKind::NetworkConnect
        | CapabilityKind::FilesystemRead
        | CapabilityKind::EnvironmentRead => "Sensitive",
        CapabilityKind::ConfigRead
        | CapabilityKind::ReportWrite
        | CapabilityKind::Crypto
        | CapabilityKind::DnsResolve => "Scoped",
    }
}

pub(crate) fn execution_state_class(state: ExecutionState) -> &'static str {
    match state {
        ExecutionState::Queued | ExecutionState::Starting => "info",
        ExecutionState::Running => "running",
        ExecutionState::Completed => "completed",
        ExecutionState::Failed | ExecutionState::TimedOut => "failed",
        ExecutionState::Cancelled => "cancelled",
    }
}
