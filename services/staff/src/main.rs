#![allow(unused)]
use argh::FromArgs;
use proto::JobEvent;
use proto::{ClaimMode, Context, DeliveryStatus, EventProcessor, StreamConsumer, TaskError};
use rand::{seq::SliceRandom, Rng};
use redis::streams::{StreamRangeReply, StreamReadOptions, StreamReadReply};
use redis::{from_redis_value, AsyncCommands, Client};
use std::{error::Error, time::Duration};
use tokio::time::sleep;

#[derive(FromArgs)]
/// Reach new heights.
struct StaffConfig {
    /// staff name
    #[argh(option)]
    name: Option<String>,
}

#[derive(Default)]
pub struct JobEventConsumer {}

#[async_trait::async_trait]
impl StreamConsumer<JobEvent> for JobEventConsumer {
    const XREAD_BLOCK_TIME: Duration = Duration::from_secs(5);
    const BATCH_SIZE: usize = 0;
    const CONCURRENCY: usize = 0;

    type Entry = JobEvent;
    type Error = anyhow::Error;
    type Data = JobEvent;

    async fn process_event(
        &self,
        ctx: &Context,
        id: &str,
        event: &JobEvent,
        status: &DeliveryStatus,
    ) -> Result<(), TaskError<Self::Error>> {
        let name = ctx.consumer_id();
        let JobEvent { status, room, data } = event;
        log::info!(target: id, "🚕 {name} received new request");
        tokio::time::sleep(Duration::new(1, 0)).await;
        if rnd_sleep(name, id).await == 5 {
            log::info!(target: id, "❌ {id}: {name} unable to handle request.");
            Err(TaskError::SkipAcknowledgement)
        } else {
            log::info!(target: id, "✅ {name} handled {data:?} for room {room:?}.");
            log::info!(target: id, "🚶 {name} returning to station.");
            tokio::time::sleep(Duration::new(3, 0)).await;

            Ok(())
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let StaffConfig { name } = argh::from_env();
    logger::init("staff");

    log::info!("Init {name:?}");
    let ctx = Context::new(proto::JOB_TOPIC, "staff", name);
    let cmode = ClaimMode::Autoclaim(Duration::new(1, 0));
    let mut reactor = <EventProcessor<JobEventConsumer, JobEvent>>::new(ctx);

    reactor.start(cmode).await?;

    Ok(())
}

async fn rnd_sleep(name: &str, id: &str) -> u64 {
    let sec = rand::thread_rng().gen_range(5..10);
    log::info!(target: id, "🕹️  {name} hanlding request ...");
    tokio::time::sleep(Duration::new(sec, 0)).await;
    sec
}
