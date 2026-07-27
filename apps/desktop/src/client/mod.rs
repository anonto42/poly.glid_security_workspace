//! Stable, UI-safe boundary for PolyGlid desktop clients.
//!
//! Views consume the DTOs and [`ClientGateway`] in this module instead of
//! depending on database records or runtime internals.

mod error;
mod gateway;
mod local;
mod models;

pub use error::{ClientError, ClientResult};
pub use gateway::{ClientGateway, ExecutionSubscription};
pub use local::LocalClient;
pub use models::{
    Approval, ApprovalDecision, ApprovalDuration, BootstrapSnapshot, CapabilityKind,
    CapabilityRequest, CapabilityScope, Execution, ExecutionEvent, ExecutionMetrics,
    ExecutionPreferences, ExecutionReport, ExecutionState, Issue, JobId, PermissionDecisionRequest,
    Plugin, PluginInspection, PluginSource, PluginStatus, Project, Report, ReportFormat,
    SavedTarget, Severity, ShellPreferences, StartExecutionRequest, Workspace,
};
