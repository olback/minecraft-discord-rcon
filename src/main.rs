use {
    config::Config,
    discord::{model::Event, Discord},
    error::Error,
    std::{thread::sleep, time::Duration},
};

mod combo;
mod config;
mod error;

async fn main_loop(conf: &Config) -> Result<(), Error> {
    let discord = Discord::from_bot_token(&conf.discord_token)?;
    let (mut discord_conn, _) = discord.connect()?;
    println!("Connected to Discrod");

    let mut com = combo::Combo {
        r: rcon::Connection::builder()
            .enable_minecraft_quirks(true)
            .connect(
                format!("{}:{}", &conf.rcon_host, &conf.rcon_port),
                &conf.rcon_password,
            )
            .await?,
        d: discord,
    };

    println!(
        "Connected to Minecraft Server at {}:{}",
        conf.rcon_host, conf.rcon_port
    );

    loop {
        match discord_conn.recv_event() {
            Ok(Event::MessageCreate(message)) => {
                if message.channel_id.0 == conf.discord_channel
                    && message.author.id != discord::model::UserId(798657174053060679)
                {
                    let msg_parts = message.content.trim().split(" ").collect::<Vec<&str>>();
                    println!("{:#?}", msg_parts);
                    match msg_parts.as_slice() {
                        &["!whitelist", username] => {
                            com.send_text(
                                &format!("whitelist add {}", username),
                                message.channel_id,
                                &format!("User '{}' has been added to the whitelist.", username),
                            )
                            .await
                        }
                        &["!list"] => com.send_rcon("list", message.channel_id).await,
                        &["!status"] => com.send_rcon("cofh tps", message.channel_id).await,
                        &["!say", _] => drop(
                            com.r
                                .cmd(&format!(
                                    "say [Discord <{}>] {}",
                                    message.author.name, message.content
                                ))
                                .await,
                        ),
                        _ => {}
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
