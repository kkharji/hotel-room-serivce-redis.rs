use proto::{JobEvent, JobEventData};

pub(crate) fn generate_job_events() -> Vec<JobEvent> {
    vec![
        JobEvent {
            data: JobEventData::RoomClean {},
            ..Default::default()
        },
        JobEvent {
            data: JobEventData::TaxiOrder {},
            ..Default::default()
        },
        JobEvent {
            data: JobEventData::ExtraTowels {},
            ..Default::default()
        },
        JobEvent {
            data: JobEventData::ExtraPillows {},
            ..Default::default()
        },
        JobEvent {
            data: JobEventData::FoodOrder {},
            ..Default::default()
        },
    ]
}
