use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Draft,
    Ready,
    InProgress,
    Review,
    Verified,
    Done,
    Blocked,
}

impl TaskStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Ready => "ready",
            Self::InProgress => "in progress",
            Self::Review => "review",
            Self::Verified => "verified",
            Self::Done => "done",
            Self::Blocked => "blocked",
        }
    }

    pub fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Draft, Self::Ready)
                | (Self::Ready, Self::InProgress)
                | (Self::InProgress, Self::Review | Self::Blocked)
                | (Self::Blocked, Self::InProgress)
                | (Self::Review, Self::InProgress | Self::Verified)
                | (Self::Verified, Self::Done | Self::InProgress)
        )
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkTrack {
    pub id: Uuid,
    pub sequence: u8,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub completed_tasks: u16,
    pub total_tasks: u16,
    pub tags: Vec<String>,
}

impl WorkTrack {
    pub fn new(
        sequence: u8,
        title: impl Into<String>,
        description: impl Into<String>,
        status: TaskStatus,
        completed_tasks: u16,
        total_tasks: u16,
        tags: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, WorkTrackError> {
        let title = title.into();
        if title.trim().is_empty() {
            return Err(WorkTrackError::EmptyTitle);
        }
        if completed_tasks > total_tasks {
            return Err(WorkTrackError::InvalidProgress {
                completed: completed_tasks,
                total: total_tasks,
            });
        }

        Ok(Self {
            id: Uuid::now_v7(),
            sequence,
            title,
            description: description.into(),
            status,
            completed_tasks,
            total_tasks,
            tags: tags.into_iter().map(Into::into).collect(),
        })
    }

    pub fn progress_percent(&self) -> u16 {
        if self.total_tasks == 0 {
            return 0;
        }
        self.completed_tasks.saturating_mul(100) / self.total_tasks
    }

    pub fn transition_to(&mut self, next: TaskStatus) -> Result<(), WorkTrackError> {
        if !self.status.can_transition_to(next) {
            return Err(WorkTrackError::InvalidTransition {
                from: self.status,
                to: next,
            });
        }
        self.status = next;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkspaceOverview {
    pub tracks: Vec<WorkTrack>,
}

impl WorkspaceOverview {
    pub fn total_tasks(&self) -> u16 {
        self.tracks.iter().map(|track| track.total_tasks).sum()
    }

    pub fn completed_tasks(&self) -> u16 {
        self.tracks.iter().map(|track| track.completed_tasks).sum()
    }

    pub fn active_tracks(&self) -> usize {
        self.tracks
            .iter()
            .filter(|track| matches!(track.status, TaskStatus::InProgress | TaskStatus::Review))
            .count()
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum WorkTrackError {
    #[error("work track title cannot be empty")]
    EmptyTitle,
    #[error("completed task count {completed} exceeds total {total}")]
    InvalidProgress { completed: u16, total: u16 },
    #[error("cannot move work track from {from:?} to {to:?}")]
    InvalidTransition { from: TaskStatus, to: TaskStatus },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_is_calculated_without_dividing_by_zero() {
        let empty =
            WorkTrack::new(1, "Empty", "", TaskStatus::Draft, 0, 0, ["test"]).expect("valid track");
        let partial = WorkTrack::new(2, "Partial", "", TaskStatus::Ready, 3, 4, ["test"])
            .expect("valid track");

        assert_eq!(empty.progress_percent(), 0);
        assert_eq!(partial.progress_percent(), 75);
    }

    #[test]
    fn invalid_progress_is_rejected() {
        let result = WorkTrack::new(1, "Broken", "", TaskStatus::Draft, 2, 1, ["test"]);

        assert_eq!(
            result,
            Err(WorkTrackError::InvalidProgress {
                completed: 2,
                total: 1
            })
        );
    }

    #[test]
    fn state_transition_policy_is_enforced() {
        let mut track = WorkTrack::new(1, "Foundation", "", TaskStatus::Draft, 0, 4, ["contracts"])
            .expect("valid track");

        track.transition_to(TaskStatus::Ready).expect("allowed");
        track
            .transition_to(TaskStatus::InProgress)
            .expect("allowed");
        assert_eq!(track.status, TaskStatus::InProgress);
        assert!(matches!(
            track.transition_to(TaskStatus::Done),
            Err(WorkTrackError::InvalidTransition { .. })
        ));
    }

    #[test]
    fn overview_aggregates_track_metrics() {
        let overview = WorkspaceOverview {
            tracks: vec![
                WorkTrack::new(1, "One", "", TaskStatus::InProgress, 2, 5, ["domain"])
                    .expect("valid track"),
                WorkTrack::new(2, "Two", "", TaskStatus::Ready, 1, 3, ["ui"]).expect("valid track"),
            ],
        };

        assert_eq!(overview.total_tasks(), 8);
        assert_eq!(overview.completed_tasks(), 3);
        assert_eq!(overview.active_tracks(), 1);
    }
}
