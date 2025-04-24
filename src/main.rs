use eyre::Result;
use settings::read_settings;

mod mqtt;
mod settings;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let settings = read_settings()?;

    mqtt::start_mqtt_loop(&settings).await?;

    tokio::signal::ctrl_c().await?;

    Ok(())
}
