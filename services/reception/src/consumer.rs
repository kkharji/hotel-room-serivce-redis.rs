use crate::ReceptionConfig;
use futures::TryStreamExt;
use proto::{JobEvent, JobEventStatus};
use rsc::RSClient;
use std::error::Error;

pub async fn run(client: &RSClient, _config: &ReceptionConfig) -> Result<(), Box<dyn Error>> {
    client.ensure_events([proto::JOB_TOPIC].iter()).await?;

    let mut stream = client.consume(proto::JOB_TOPIC);

    while let Some(message) = stream.try_next().await? {
        let id = &message.id.to_string();
        let event: JobEvent = message.data()?;
        let JobEvent { status, room, data } = &event;
        match status {
            JobEventStatus::Requested => {}
            JobEventStatus::Completed => {
                log::info!(target: &id, "✅ Handled {data:?} for room {room:?}",);
            }
        }
        message.ack().await?;
    }
    Ok(())
}
