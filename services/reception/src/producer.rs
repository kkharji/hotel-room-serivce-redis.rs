use super::ReceptionConfig;
use crate::gen;
use proto::{JobEvent, StreamEntry};
use rand::seq::SliceRandom;
use redis::AsyncCommands;
use std::error::Error;
use std::time::Duration;

pub(crate) async fn run(_config: &ReceptionConfig) -> Result<(), Box<dyn Error>> {
    let _config: ReceptionConfig = argh::from_env();
    let mut rng = rand::thread_rng();
    let mut con = redis_swapplex::get_connection();
    let jobs = gen::generate_job_events();

    loop {
        let mut job = jobs
            .to_vec()
            .choose(&mut rng)
            .map(|v| v.to_owned())
            .unwrap();

        job.room = rand::Rng::gen_range(&mut rng, 100..500);

        let id: String = con.xadd_map(proto::JOB_TOPIC, "*", job.xadd_map()?).await?;
        let JobEvent { room, data, .. } = job;

        log::info!(target: &id, "📤 Created {data:?} for room {room:?}",);

        tokio::time::sleep(Duration::new(rand::Rng::gen_range(&mut rng, 2..6), 0)).await;
    }
}
