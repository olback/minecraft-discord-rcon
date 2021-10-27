use {
    commands::Commands,
    config::Config,
    error::Result,
    serenity::{client::bridge::gateway::GatewayIntents, Client},
};

mod commands;
mod config;
mod error;

pub const INTENTS: &[GatewayIntents] = &[GatewayIntents::GUILD_MESSAGES];

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load()?;

    let mut client = Client::builder(&config.discord_token)
        .intents(
            INTENTS
                .iter()
                .fold(GatewayIntents::empty(), |tot, next| tot | *next),
        )
        .event_handler(Commands::new(config))
        .await?;

    client.start().await?;

    Ok(())
}
