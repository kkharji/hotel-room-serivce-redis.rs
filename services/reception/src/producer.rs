use super::ReceptionConfig;
use crate::gen;
use proto::JobEvent;
use rand::seq::SliceRandom;
use std::error::Error;
use std::time::Duration;

pub(crate) async fn run(
    client: &rsc::RSClient,
    _config: &ReceptionConfig,
) -> Result<(), Box<dyn Error>> {
    let _config: ReceptionConfig = argh::from_env();
    let mut rng = rand::thread_rng();
    let jobs = gen::generate_job_events();

    loop {
        let mut job = jobs
            .to_vec()
            .choose(&mut rng)
            .map(|v| v.to_owned())
            .unwrap();

        job.room = rand::Rng::gen_range(&mut rng, 100..500);

        let id = client.publish(proto::JOB_TOPIC, &job).await?;
        let id = id.to_string();
        let JobEvent { room, data, .. } = job;

        log::info!(target: &id, "📤 Created {data:?} for room {room:?}");

        tokio::time::sleep(Duration::new(rand::Rng::gen_range(&mut rng, 2..6), 0)).await;
    }
}
