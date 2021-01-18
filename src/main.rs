use {
    config::Config,
    discord::{model::Event, Discord},
    error::Error,
    std::{thread::sleep, time::Duration},
};

mod config;
mod error;

async fn main_loop(conf: &Config) -> Result<(), Error> {
    let mut rcon_conn = rcon::Connection::builder()
        .enable_minecraft_quirks(true)
        .connect(
            format!("{}:{}", &conf.rcon_host, &conf.rcon_port),
            &conf.rcon_password,
        )
        .await?;
    println!(
        "Connected to Minecraft Server at {}:{}",
        conf.rcon_host, conf.rcon_port
    );

    let discord = Discord::from_bot_token(&conf.discord_token)?;
    let (mut discord_conn, _) = discord.connect()?;
    println!("Connected to Discrod");

    loop {
        match discord_conn.recv_event() {
            Ok(Event::MessageCreate(message)) => {
                if message.channel_id.0 == conf.discord_channel {
                    let msg_parts = message.content.trim().split(" ").collect::<Vec<&str>>();
                    println!("{:#?}", msg_parts);
                    match msg_parts.as_slice() {
                        &["!whitelist", username] => match rcon_conn
                            .cmd(&format!("whitelist add {}", username))
                            .await
                        {
                            Ok(_) => drop(discord.send_message(
                                message.channel_id,
                                &format!("User '{}' has been added to the whitelist.", username),
                                "",
                                false,
                            )),
                            Err(e) => drop(discord.send_message(
                                message.channel_id,
                                &format!("{}", e),
                                "",
                                false,
                            )),
                        },
                        &["!list"] => match rcon_conn.cmd("list").await {
                            Ok(list_text) => drop(discord.send_message(
                                message.channel_id,
                                &list_text,
                                "",
                                false,
                            )),
                            Err(e) => drop(discord.send_message(
                                message.channel_id,
                                &format!("{}", e),
                                "",
                                false,
                            )),
                        },
                        _ => drop(
                            rcon_conn
                                .cmd(&format!(
                                    "say [Discord <{}>] {}",
                                    message.author.name, message.content
                                ))
                                .await,
                        ),
                    }
                    /*if msg_parts.len() == 2 {
                        match rcon_conn
                            .cmd(&format!("whitelist add {}", msg_parts[1]))
                            .await
                        {
                            Ok(_) => drop(discord.send_message(
                                message.channel_id,
                                &format!("User '{}' has been added to the whitelist", msg_parts[1]),
                                "",
                                false,
                            )),
                            Err(e) => {
                                drop(discord.send_message(
                                    message.channel_id,
                                    &format!("{}", e),
                                    "",
                                    false,
                                ));
                                break;
                            }
                        }
                    } else {
                        let _ = discord.send_message(
                            message.channel_id,
                            "Usage: !whitelist <username>",
                            "",
                            false,
                        );
                    }*/
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
    let conf = config::Config::load().expect("Failed to load config");
    println!("{:#?}", conf);
    let mut attempt_timeout = 1;

    loop {
        match main_loop(&conf).await {
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
