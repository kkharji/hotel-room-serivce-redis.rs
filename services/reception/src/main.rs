use argh::FromArgs;
use rsc::{RSClient, RSConfig};
use std::error::Error;
use tokio::try_join;
// use tokio::try_join;

mod consumer;
mod gen;
mod producer;

#[derive(FromArgs)]
/// Reception server configuration
pub struct ReceptionConfig {
    /// manager name
    #[argh(option)]
    name: Option<String>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    logger::init("reception");

    let mut config: ReceptionConfig = argh::from_env();
    config.name.get_or_insert("Sara".into());

    log::info!("Starting");

    let client = RSClient::new(RSConfig {
        stream_group_name: "Reception".into(),
        stream_consumer_name: config.name.clone(),
        stream_autoclaim_min_idle_time: 5000,
        stream_autoclaim_block_interval: 3000,
        ..RSConfig::default()
    })
    .await?;

    client.ensure_events([proto::JOB_TOPIC].iter()).await?;

    try_join!(
        producer::run(&client, &config),
        consumer::run(&client, &config)
    )?;

    Ok(())
}
