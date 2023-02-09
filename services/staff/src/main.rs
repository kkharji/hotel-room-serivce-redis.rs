use argh::FromArgs;
use futures::TryStreamExt;
use proto::{JobEvent, JobEventStatus};
use rand::Rng;
use std::{error::Error, time::Duration};

#[derive(FromArgs)]
/// Reach new heights.
struct StaffConfig {
    /// staff name
    #[argh(option)]
    name: Option<String>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    logger::init("staff");

    let StaffConfig { name } = argh::from_env();

    log::info!("Init {name:?}");

    let client = rsc::RSClient::new(rsc::RSConfig {
        stream_group_name: "Staff".into(),
        stream_consumer_name: name.clone(),
        stream_autoclaim_block_interval: 5000,
        stream_autoclaim_min_idle_time: 5000,
        ..Default::default()
    })
    .await?;

    let name = name.unwrap_or_else(|| "StaffMemeber".into());

    client.ensure_events([proto::JOB_TOPIC].iter()).await?;
    let mut stream = client.consume(proto::JOB_TOPIC);

    while let Some(message) = stream.try_next().await? {
        let id = &message.id.to_string();
        let event: JobEvent = message.data()?;
        let JobEvent { status, room, data } = &event;

        if status.is_completed() {
            log::info!(target: id, "skipping completed event");
            message.ack().await?;
            continue;
        }

        log::info!(target: id, "🚕 {name} received new request");

        tokio::time::sleep(Duration::new(1, 0)).await;

        let skip = rnd_sleep(&name, id).await == 5;

        if !skip {
            log::info!(target: id, "✅ {name} handled {data:?} for room {room:?}.");
            let mut update = event.clone();
            update.status = JobEventStatus::Completed;
            message.ack().await?;

            let client = client.clone();

            tokio::spawn(async move {
                let _id = client.publish(proto::JOB_TOPIC, &update).await?;
                Ok::<_, rsc::RSError>(())
            });
        }

        log::info!(target: id, "🚶 {name} returning to station.");
        tokio::time::sleep(Duration::new(3, 0)).await;
        log::info!(target: id, "🚶 {name} Ready.");

        if !skip {
            message.ack().await?;
        }
    }

    Ok(())
}

async fn rnd_sleep(name: &str, id: &str) -> u64 {
    let sec = rand::thread_rng().gen_range(5..10);
    log::info!(target: id, "🕹️  {name} hanlding request ...");
    tokio::time::sleep(Duration::new(sec, 0)).await;
    sec
}
