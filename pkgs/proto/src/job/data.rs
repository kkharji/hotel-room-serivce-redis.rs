use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum JobEventData {
    RoomClean {},
    TaxiOrder {},
    ExtraTowels {},
    ExtraPillows {},
    FoodOrder {},
}

impl Default for JobEventData {
    fn default() -> Self {
        Self::RoomClean {}
    }
}

impl TryFrom<&str> for JobEventData {
    type Error = serde_json::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(value)
    }
}

impl TryFrom<String> for JobEventData {
    type Error = serde_json::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        JobEventData::try_from(value.as_str())
    }
}
