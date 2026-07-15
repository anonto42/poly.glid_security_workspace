mod agents;
mod automation;
mod plugins;
mod projects;
mod scanner;
mod tracks;

pub(crate) use agents::AgentsDashboard;
pub(crate) use automation::AutomationDashboard;
pub(crate) use plugins::PluginDashboard;
pub(crate) use projects::ProjectsDashboard;
pub(crate) use scanner::{ResultDashboard, ScannerDashboard, SourceDashboard};
pub(crate) use tracks::TracksDashboard;
