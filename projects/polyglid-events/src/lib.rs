//! Typed host events shared by CLI, runtime, and future UI surfaces.

use polyglid_plugin_api::{PluginId, PluginReport};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolyGlidEvent {
    WorkspaceDiscoveryStarted {
        workspace_id: String,
        root_path: String,
    },
    WorkspaceDiscoveryCompleted {
        workspace_id: String,
        project_count: usize,
    },
    WorkspaceDiscoveryFailed {
        workspace_id: String,
        message: String,
    },
    WorkspaceActivated {
        workspace_id: String,
    },
    ProjectCreated {
        workspace_id: String,
        project_id: String,
        path: String,
    },
    ProjectRenamed {
        project_id: String,
        name: String,
        path: String,
    },
    ProjectRemoved {
        project_id: String,
        files_deleted: bool,
    },
    PluginInspectStarted {
        path: String,
    },
    PluginRunStarted {
        plugin_id: PluginId,
        target: String,
    },
    PluginRunCompleted {
        plugin_id: PluginId,
        report: PluginReport,
    },
    PluginRunFailed {
        plugin_id: PluginId,
        message: String,
    },
    CapabilityAllowed {
        plugin_id: PluginId,
        capability: String,
    },
    CapabilityDenied {
        plugin_id: PluginId,
        capability: String,
        reason: String,
    },
    CapabilityCheckFailed {
        plugin_id: PluginId,
        capability: String,
        message: String,
    },
}

pub trait EventSink {
    fn emit(&mut self, event: PolyGlidEvent);
}

#[derive(Debug, Default)]
pub struct VecEventSink {
    events: Vec<PolyGlidEvent>,
}

impl VecEventSink {
    pub fn events(&self) -> &[PolyGlidEvent] {
        &self.events
    }
}

impl EventSink for VecEventSink {
    fn emit(&mut self, event: PolyGlidEvent) {
        self.events.push(event);
    }
}
