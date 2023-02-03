use super::*;
use serde::{Deserialize, Serialize};
use serde_with::{json::JsonString, serde_as, DisplayFromStr, PickFirst};

pub const JOB_TOPIC: &str = "jobs";

#[serde_as]
#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct JobEvent {
    pub status: JobEventStatus,
    #[serde_as(as = "DisplayFromStr")]
    pub room: u32,
    #[serde_as(as = "PickFirst<(_, JsonString)>")]
    pub data: JobEventData,
}
