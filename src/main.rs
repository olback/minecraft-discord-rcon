use {
    config::Config,
    discord::model::Event,
    error::Error,
    lazy_static::lazy_static,
    std::{thread::sleep, time::Duration},
};

lazy_static! {
    pub static ref CONFIG: Config = Config::load().expect("Failed to load config");
}

mod combo;
mod config;
mod error;

async fn main_loop(conf: &Config) -> Result<(), Error> {
    let mut com = combo::Combo::new(conf).await?;
    let mut discord_conn = com.discord_connection()?;

    loop {
        match discord_conn.recv_event() {
            Ok(Event::MessageCreate(message)) => {
                if message.channel_id.0 == conf.discord_channel
                    && message.author.id != discord::model::UserId(798657174053060679)
                {
                    let msg_parts = message.content.trim().split(" ").collect::<Vec<&str>>();
                    // println!("{:#?}", msg_parts);
                    match msg_parts.as_slice() {
                        &["!whitelist", username] => {
                            com.send_rcon(
                                &format!("whitelist add {}", username),
                                message.channel_id,
                            )
                            .await
                        }
                        &["!list"] => com.send_rcon("list", message.channel_id).await,
                        &["!status"] => com.send_rcon("cofh tps", message.channel_id).await,
                        &["!say", ..] => drop(
                            com.rcon_cmd(
                                &format!(
                                    "say [Discord <{}>] {}",
                                    message.author.name,
                                    &msg_parts[1..].join(" ")
                                ),
                                true,
                            )
                            .await,
                        ),
                        _ => {
                            /* if com.is_old() && com.is_dead().await {
                                com.rcon_reconnect(conf).await;
                            }*/
                        }
                    }
                }
            }

            Ok(_) => {}
            Err(discord::Error::Closed(code, body)) => {
                eprintln!(
                    "Discord gateway closed connection with code {:?}: {}",
                    code, body
                );
                break;
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let mut attempt_timeout = 1;

    loop {
        match main_loop(&CONFIG).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", e);
                eprintln!("Waiting {} seconds before trying again...", attempt_timeout);
                sleep(Duration::from_secs(attempt_timeout));
                attempt_timeout *= 2;
            }
        }
    }
}
