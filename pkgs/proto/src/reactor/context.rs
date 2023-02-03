use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub struct Context<'a> {
    pub(crate) consumer_id: String,
    pub(crate) stream_key: &'a str,
    pub(crate) group_name: &'a str,
    pub(crate) cancel_token: Arc<CancellationToken>,
}

impl<'a> Context<'a> {
    pub fn new(stream_key: &'a str, group_name: &'a str, consumer_id: Option<String>) -> Self {
        Self {
            consumer_id: consumer_id.unwrap_or_else(Self::generate_id),
            stream_key,
            group_name,
            cancel_token: Arc::new(CancellationToken::new()),
        }
    }

    pub(crate) fn generate_id() -> String {
        use rand::distributions::Alphanumeric;
        use rand::Rng;
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect()
    }

    pub fn consumer_id(&self) -> &str {
        &self.consumer_id
    }

    pub fn stream_key(&self) -> &str {
        self.stream_key
    }

    pub fn group_name(&self) -> &str {
        self.group_name
    }

    /// Process claimed stream entries and shutdown
    pub fn shutdown_gracefully(&self) {
        self.cancel_token.cancel();
    }
}
