use thiserror::Error;

use super::{CapabilityKind, JobId};

pub type ClientResult<T> = Result<T, ClientError>;

#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum ClientError {
    #[error("desktop services are unavailable: {0}")]
    Unavailable(String),

    #[error("invalid {field}: {message}")]
    InvalidInput {
        field: &'static str,
        message: String,
    },

    #[error("{resource} '{id}' was not found")]
    NotFound { resource: &'static str, id: String },

    #[error("{0}")]
    Conflict(String),

    #[error("plugin '{plugin_id}' still needs approval for {missing:?}")]
    CapabilityApprovalRequired {
        plugin_id: String,
        missing: Vec<CapabilityKind>,
    },

    #[error("plugin '{plugin_id}' did not request {capabilities:?}")]
    UnexpectedCapabilityApproval {
        plugin_id: String,
        capabilities: Vec<CapabilityKind>,
    },

    #[error("{operation} failed: {message}")]
    Operation {
        operation: &'static str,
        message: String,
    },

    #[error("execution event stream closed")]
    EventStreamClosed,

    #[error("execution event stream dropped {0} event(s)")]
    EventStreamLagged(u64),

    #[error("timed out waiting for execution {job_id}")]
    WaitTimedOut { job_id: JobId },
}

impl ClientError {
    pub(crate) fn operation(operation: &'static str, message: impl Into<String>) -> Self {
        Self::Operation {
            operation,
            message: message.into(),
        }
    }
}
