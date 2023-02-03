use argh::FromArgs;
use proto::StreamEntry;
use proto::{JobEvent, JobEventData};
use rand::seq::SliceRandom;
use redis::AsyncCommands;
use std::{error::Error, time::Duration};

#[derive(FromArgs)]
/// Reception server configuration
struct ReceptionConfig {}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    logger::init("reception");

    let _config: ReceptionConfig = argh::from_env();
    let mut rng = rand::thread_rng();
    let mut con = redis_swapplex::get_connection();
    let jobs = generate_job_events();

    loop {
        let mut job = jobs
            .to_vec()
            .choose(&mut rng)
            .map(|v| v.to_owned())
            .unwrap();

        job.room = rand::Rng::gen_range(&mut rng, 100..500);

        let id: String = con.xadd_map(proto::JOB_TOPIC, "*", job.xadd_map()?).await?;
        let JobEvent { room, data, .. } = job;

        log::info!(target: &id, "- Created {data:?} for room {room:?}",);

        tokio::time::sleep(Duration::new(rand::Rng::gen_range(&mut rng, 5..10), 0)).await;
    }
}

fn generate_job_events() -> Vec<JobEvent> {
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
