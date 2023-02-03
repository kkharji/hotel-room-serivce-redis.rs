use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub enum JobEventStatus {
    #[default]
    Requested,
    Pending,
    Completed,
    Canceled,
}

impl JobEventStatus {
    /// Returns `true` if the job event status is [`Requested`].
    ///
    /// [`Requested`]: JobEventStatus::Requested
    #[must_use]
    pub fn is_requested(&self) -> bool {
        matches!(self, Self::Requested)
    }

    /// Returns `true` if the job event status is [`Pending`].
    ///
    /// [`Pending`]: JobEventStatus::Pending
    #[must_use]
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending)
    }

    /// Returns `true` if the job event status is [`Completed`].
    ///
    /// [`Completed`]: JobEventStatus::Completed
    #[must_use]
    pub fn is_completed(&self) -> bool {
        matches!(self, Self::Completed)
    }

    /// Returns `true` if the job event status is [`Canceled`].
    ///
    /// [`Canceled`]: JobEventStatus::Canceled
    #[must_use]
    pub fn is_canceled(&self) -> bool {
        matches!(self, Self::Canceled)
    }
}
