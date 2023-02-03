use crate::ReceptionConfig;
use proto::{ClaimMode, Context, DeliveryStatus, EventProcessor, StreamConsumer, TaskError};
use proto::{JobEvent, JobEventStatus};
use std::{error::Error, time::Duration};

#[derive(Default)]
pub struct JobEventConsumer {}

#[async_trait::async_trait]
impl StreamConsumer<JobEvent> for JobEventConsumer {
    const XREAD_BLOCK_TIME: Duration = Duration::from_secs(5);
    const BATCH_SIZE: usize = 5;
    const CONCURRENCY: usize = 5;

    type Entry = JobEvent;
    type Error = anyhow::Error;
    type Data = JobEvent;

    async fn process_event(
        &self,
        _ctx: &Context,
        id: &str,
        event: &JobEvent,
        _status: &DeliveryStatus,
    ) -> Result<(), TaskError<Self::Error>> {
        let JobEvent { status, room, data } = event;
        match status {
            JobEventStatus::Requested => {
                log::info!(target: id, "Skipped");
            }
            JobEventStatus::Completed => {
                log::info!(target: &id, "✅ Handled {data:?} for room {room:?}",);
            }
        }
        Ok(())
    }
}

pub async fn run(config: &ReceptionConfig) -> Result<(), Box<dyn Error>> {
    let ctx = Context::new(proto::JOB_TOPIC, "manager", config.name.clone());
    let cmode = ClaimMode::NewOnly;
    let mut reactor = <EventProcessor<JobEventConsumer, JobEvent>>::new(ctx);

    reactor.start(cmode).await?;
    Ok(())
}
