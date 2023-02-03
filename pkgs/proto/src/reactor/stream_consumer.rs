use super::error::TaskError;
use async_trait::async_trait;
use futures::{stream, StreamExt, TryStreamExt};
use redis::{streams::StreamId, AsyncCommands, RedisError};
use redis_swapplex::get_connection;
use std::{fmt::Debug, time::Duration};

use super::{Context, DeliveryStatus, StreamEntry};

#[async_trait]
pub trait StreamConsumer<T: StreamEntry>: Default + Send + Sync {
    const XREAD_BLOCK_TIME: Duration;
    const BATCH_SIZE: usize;
    const CONCURRENCY: usize;

    type Entry: StreamEntry;
    type Error: Send + Debug;
    type Data: Send + Sync;

    async fn process_event(
        &self,
        ctx: &Context,
        id: &str,
        event: &Self::Entry,
        status: &DeliveryStatus,
    ) -> Result<(), TaskError<Self::Error>>;

    async fn process_event_stream(
        &self,
        ctx: &Context,
        ids: Vec<StreamId>,
        status: &DeliveryStatus,
    ) -> Result<(), RedisError> {
        stream::iter(ids.into_iter())
            .map(Ok)
            .try_for_each_concurrent(Self::CONCURRENCY, |entry| async move {
                self.process_stream_entry(ctx, entry, status).await
            })
            .await?;

        Ok(())
    }

    async fn process_stream_entry(
        &self,
        ctx: &Context,
        entry: StreamId,
        status: &DeliveryStatus,
    ) -> Result<(), RedisError> {
        let event = <Self::Entry as StreamEntry>::from_stream_id(&entry)?;

        match self.process_event(ctx, &entry.id, &event, status).await {
            Ok(()) => {
                let mut conn = get_connection();
                let _: i64 = conn
                    .xack(ctx.stream_key(), ctx.group_name, &[&entry.id])
                    .await?;
            }
            Err(err) => match err {
                TaskError::SkipAcknowledgement => (),
                TaskError::Delete => {
                    let mut conn = get_connection();
                    let _: i64 = conn.xdel(ctx.stream_key(), &[&entry.id]).await?;
                }
                TaskError::Error(err) => {
                    log::error!("Error processing stream event: {:?}", err);
                }
            },
        }

        Ok(())
    }
}
