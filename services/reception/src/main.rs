use argh::FromArgs;
use std::error::Error;
use tokio::try_join;

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

    try_join!(producer::run(&config), consumer::run(&config))?;

    Ok(())
}
