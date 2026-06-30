//! Typed host events shared by CLI, runtime, and future UI surfaces.

use polyglid_plugin_api::{PluginId, PluginReport};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolyGlidEvent {
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
    CapabilityDenied {
        plugin_id: PluginId,
        capability: String,
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
