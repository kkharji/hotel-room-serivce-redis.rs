use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub enum JobEventStatus {
    #[default]
    Requested,
    Completed,
}

impl JobEventStatus {
    /// Returns `true` if the job event status is [`Requested`].
    ///
    /// [`Requested`]: JobEventStatus::Requested
    #[must_use]
    pub fn is_requested(&self) -> bool {
        matches!(self, Self::Requested)
    }

    /// Returns `true` if the job event status is [`Completed`].
    ///
    /// [`Completed`]: JobEventStatus::Completed
    #[must_use]
    pub fn is_completed(&self) -> bool {
        matches!(self, Self::Completed)
    }
}
