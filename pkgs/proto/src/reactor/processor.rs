use super::*;
use backoff::ExponentialBackoff;
use redis::streams::{StreamId, StreamKey, StreamRangeReply, StreamReadOptions, StreamReadReply};
use redis::{AsyncCommands, RedisError};
use redis_swapplex::get_connection;
use std::time::{Duration, SystemTime};
use std::{fmt::Debug, marker::PhantomData, sync::Arc};
use tokio::{signal, time::sleep, try_join};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug)]
/// The current state of the consumer group
pub enum ConsumerGroupState {
    Uninitialized,
    NewlyCreated,
    PreviouslyCreated,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DeliveryStatus {
    /// Stream entry newly delivered to consumer group
    NewDelivery,
    /// Stream entry was claimed via XAUTOCLAIM
    MinIdleElapsed,
}

/// Redis event stream reactor
pub struct EventProcessor<'a, T, E>
where
    T: StreamConsumer<E>,
    E: StreamEntry,
{
    group_status: ConsumerGroupState,
    consumer: Arc<T>,
    ctx: Context<'a>,
    _marker: PhantomData<fn() -> E>,
}

impl<'a, T, E> EventProcessor<'a, T, E>
where
    T: StreamConsumer<E>,
    E: StreamEntry,
{
    pub fn new(ctx: Context<'a>) -> Self {
        EventProcessor {
            group_status: ConsumerGroupState::Uninitialized,
            consumer: Arc::new(T::default()),
            ctx,
            _marker: PhantomData,
        }
    }

    async fn initialize_consumer_group(&mut self) -> Result<ConsumerGroupState, RedisError> {
        match get_connection()
            .xgroup_create_mkstream(self.ctx.stream_key(), self.ctx.group_name(), "0")
            .await
            .map(|_: String| ())
        {
            // It is expected behavior that this will fail when already initalized
            // Expected error: `BUSYGROUP: Consumer Group name already exists`
            Err(err) if err.code() == Some("BUSYGROUP") => {
                self.group_status = ConsumerGroupState::PreviouslyCreated;
                Ok(ConsumerGroupState::PreviouslyCreated)
            }
            Err(err) => Err(err),
            Ok(_) => {
                self.group_status = ConsumerGroupState::NewlyCreated;
                Ok(ConsumerGroupState::NewlyCreated)
            }
        }
    }

    async fn autoclaim_batch(
        &self,
        min_idle: &Duration,
        cursor: &str,
    ) -> Result<Option<(String, StreamRangeReply)>, RedisError> {
        if self.ctx.cancel_token.is_cancelled() {
            return Ok(None);
        }

        tokio::select! {
          reply = async {
            let mut conn = get_connection();

            let mut cmd = redis::cmd("XAUTOCLAIM");

            cmd.arg(self.ctx.stream_key())
              .arg(self.ctx.group_name())
              .arg(self.ctx.consumer_id())
              .arg(min_idle.as_millis() as usize)
              .arg(cursor)
              .arg("COUNT")
              .arg(T::BATCH_SIZE);

            let reply: (String, StreamRangeReply) = cmd.query_async(&mut conn).await?;

            Ok(Some(reply))
          } => reply,
          _ = self.ctx.cancel_token.cancelled() => Ok(None),
          _ = signal::ctrl_c() => Ok(None),
        }
    }

    async fn process_idle_pending(
        &self,
        min_idle: &Duration,
        max_idle: &Option<Duration>,
    ) -> Result<(), RedisError> {
        let mut cursor = String::from("0-0");

        let start_time = SystemTime::now();

        while let Some((next_cursor, reply)) =
            self.autoclaim_batch(min_idle, cursor.as_str()).await?
        {
            cursor = next_cursor;

            let poll_time = SystemTime::now();

            if !reply.ids.is_empty() {
                self.consumer
                    .process_event_stream(&self.ctx, reply.ids, &DeliveryStatus::MinIdleElapsed)
                    .await?;
            } else if let Some(sleep_time) = min_idle.checked_div(4) {
                if let Some(max_idle) = max_idle {
                    if let Ok(call_elapsed) = poll_time.duration_since(start_time) {
                        if call_elapsed.gt(max_idle) {
                            continue;
                        }
                    }
                }

                tokio::select! {
                    _ = self.ctx.cancel_token.cancelled() => {
                      continue;
                    }
                    _ = sleep(sleep_time) => {}
                }
            }
        }

        Ok(())
    }

    async fn next_batch(&self) -> Result<Option<StreamReadReply>, RedisError> {
        if self.ctx.cancel_token.is_cancelled() {
            return Ok(None);
        }

        tokio::select! {
          reply = async {
            let mut conn = get_connection();

            let reply: StreamReadReply = conn
            .xread_options(
              &[self.ctx.stream_key()],
              &[">"],
              &StreamReadOptions::default()
                .group(self.ctx.group_name(), self.ctx.consumer_id())
                .block(T::XREAD_BLOCK_TIME.as_millis() as usize)
                .count(T::BATCH_SIZE),
            )
            .await?;

            Ok(Some(reply))
          } => reply,
          _ = self.ctx.cancel_token.cancelled() => Ok(None),
          _ = signal::ctrl_c() => Ok(None)
        }
    }

    async fn process_stream(&self) -> Result<(), RedisError> {
        while let Some(reply) = self.next_batch().await? {
            if !reply.keys.is_empty() {
                let ids: Vec<StreamId> = reply
                    .keys
                    .into_iter()
                    .flat_map(|key| {
                        let StreamKey { ids, .. } = key;
                        ids.into_iter()
                    })
                    .collect();

                self.consumer
                    .process_event_stream(&self.ctx, ids, &DeliveryStatus::NewDelivery)
                    .await?;
            }
        }

        Ok(())
    }

    /// Process idle-pending backlog without claiming new entries until none remain and max_idle has elapsed (relative to start_time)
    pub async fn clear_idle_backlog(
        &mut self,
        min_idle: &Duration,
        max_idle: &Option<Duration>,
    ) -> Result<(), RedisError> {
        if matches!(self.group_status, ConsumerGroupState::Uninitialized) {
            self.initialize_consumer_group().await?;
        }

        if matches!(self.group_status, ConsumerGroupState::PreviouslyCreated) {
            self.process_idle_pending(
                min_idle,
                &Some(max_idle.unwrap_or_else(|| Duration::new(0, 0))),
            )
            .await?;
        }

        Ok(())
    }

    /// Process redis stream entries until shutdown signal received
    pub async fn start(&mut self, claim_mode: ClaimMode) -> Result<(), RedisError> {
        if matches!(self.group_status, ConsumerGroupState::Uninitialized) {
            self.initialize_consumer_group().await?;
        }

        match claim_mode {
            ClaimMode::ClaimAllPending => {
                if matches!(self.group_status, ConsumerGroupState::PreviouslyCreated) {
                    try_join!(
                        backoff::future::retry(ExponentialBackoff::default(), || async {
                            self.process_idle_pending(
                                &Duration::new(0, 0),
                                &Some(Duration::new(0, 0)),
                            )
                            .await?;

                            Ok(())
                        }),
                        backoff::future::retry(ExponentialBackoff::default(), || async {
                            self.process_stream().await?;

                            Ok(())
                        })
                    )?;
                } else {
                    backoff::future::retry(ExponentialBackoff::default(), || async {
                        self.process_stream().await?;

                        Ok(())
                    })
                    .await?;
                }
            }
            ClaimMode::ClearBacklog { min_idle, max_idle } => {
                if matches!(self.group_status, ConsumerGroupState::PreviouslyCreated) {
                    try_join!(
                        backoff::future::retry(ExponentialBackoff::default(), || async {
                            self.process_idle_pending(
                                &min_idle,
                                &Some(max_idle.unwrap_or_else(|| Duration::new(0, 0))),
                            )
                            .await?;

                            Ok(())
                        }),
                        backoff::future::retry(ExponentialBackoff::default(), || async {
                            self.process_stream().await?;

                            Ok(())
                        })
                    )?;
                } else {
                    backoff::future::retry(ExponentialBackoff::default(), || async {
                        self.process_stream().await?;

                        Ok(())
                    })
                    .await?;
                }
            }
            ClaimMode::Autoclaim(min_idle) => {
                try_join!(
                    backoff::future::retry(ExponentialBackoff::default(), || async {
                        log::debug!("Autoclaming");
                        self.process_idle_pending(&min_idle, &None).await?;

                        Ok(())
                    }),
                    backoff::future::retry(ExponentialBackoff::default(), || async {
                        self.process_stream().await?;

                        Ok(())
                    })
                )?;
            }
            ClaimMode::NewOnly => {
                backoff::future::retry(ExponentialBackoff::default(), || async {
                    self.process_stream().await?;

                    Ok(())
                })
                .await?;
            }
        }

        Ok(())
    }

    pub fn shutdown_token(&self) -> Arc<CancellationToken> {
        self.ctx.cancel_token.clone()
    }
}
