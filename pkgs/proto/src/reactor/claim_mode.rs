use std::time::Duration;

/// Claim modes governing work-stealing from other consumers
pub enum ClaimMode {
    /// Claim all pending entries from other consumers irregardless of min-idle-time until none
    /// remain while concurrently processing new entries until shutdown. This is useful for
    /// facilitating work-resumation for use cases in which there's guaranteed to be at most one
    /// consumer and a restart occurs; such case requires no idle-delay to claim pending work
    /// because it can be reasoned the old consumer is offline and has no pending work in progress.
    ClaimAllPending,

    /// Process new entries until shutdown and while concurrently clearing the idle-pending backlog
    /// until none remain and max_idle has elapsed
    ClearBacklog {
        /// min-idle-time filter for XAUTOCLAIM, which transfers ownership to this consumer of
        /// messages pending for more than <min-idle-time>.
        min_idle: Duration,
        /// max-idle time relative to reactor start time to await newly idle entries before
        /// proceeding to only claim new entries
        max_idle: Option<Duration>,
    },

    /// Process new entries until shutdown while continuously autoclaiming entries from other
    /// consumers that have not been acknowledged within the span of min_idle
    ///
    /// Transfers ownership to this consumer of messages pending for more than given Duration
    Autoclaim(Duration),

    /// Only process new entries
    NewOnly,
}
