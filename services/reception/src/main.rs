use futures::{TryFutureExt, TryStreamExt};
use proto::{JobEvent, JobEventStatus};
use rand::{prelude::*, rngs::*};
use rsc::{RSClient, RSConfig};
use std::time::Duration;

mod gen;

/// Reception server configuration
#[derive(argh::FromArgs)]
pub struct ReceptionConfig {
    /// manager name
    #[argh(option)]
    name: Option<String>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::init("reception");

    let mut tasks = tokio::task::JoinSet::new();
    let ReceptionConfig { mut name } = argh::from_env();

    name.get_or_insert("Sara".into());

    let client = RSClient::new(RSConfig {
        stream_group_name: "Reception".into(),
        stream_consumer_name: name,
        stream_autoclaim_block_interval: 2000,
        stream_autoclaim_min_idle_time: 3000,
        ..RSConfig::default()
    })
    .and_then(|client| async {
        client.ensure_events([proto::JOB_TOPIC].iter()).await?;
        Ok(client)
    })
    .await?;

    // Generate Random events and push them to jobs
    let publisher = client.clone();
    tasks.spawn(async move {
        log::info!("Started Event Publisher");
        let mut rng = StdRng::from_seed(OsRng.gen());
        let jobs = gen::generate_job_events();
        loop {
            let mut job = jobs.to_vec().choose(&mut rng).map(Clone::clone).unwrap();

            job.room = rng.gen_range(100..500);

            let id = publisher.publish(proto::JOB_TOPIC, &job).await?;

            log::info! { target: &id, "📤 Created {:?} for room {:?}", job.data, job.room };

            tokio::time::sleep(Duration::new(rng.gen_range(2..6), 0)).await;
        }
    });

    // Consume events, only react when job event status is completed
    let consumer = client.clone();
    tasks.spawn(async move {
        log::info!("Started Event Consumer");
        let mut stream = consumer.consume(proto::JOB_TOPIC);
        while let Some(message) = stream.try_next().await? {
            match message.data() {
                Ok(JobEvent { status, room, data }) => {
                    match status {
                        JobEventStatus::Completed => {
                            log::info!(target: &message.id, "✅ Handled {data:?} for room {room:?}");
                        }
                        JobEventStatus::Requested => {}
                    }
                    message.ack().await?;
                }
                Err(err) => log::error!("Failed to handle a message: {err:?}"),
            }
        }

        Ok::<_, rsc::RSError>(())
    });

    while let Some(res) = tasks.join_next().await {
        if let Err(err) = res {
            log::error!("Local async task exist with err: {err:?}")
        }
    }

    Ok(())
}
