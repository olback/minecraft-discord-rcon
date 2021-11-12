use {
    crate::{config::Config, error::Result},
    serenity::{
        async_trait,
        client::{Context, EventHandler},
        model::{
            channel::Message,
            gateway::{Activity, Ready},
            user::OnlineStatus,
        },
    },
    std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

macro_rules! try_reply {
    ($expr:expr) => {
        if let Err(err) = $expr {
            eprintln!("{:#?}", err)
        }
    };
}

pub struct Commands {
    config: Config,
    // Background task active?
    task_active: AtomicBool,
}

impl Commands {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            task_active: AtomicBool::new(false),
        }
    }
}

#[async_trait]
impl EventHandler for Commands {
    async fn message(&self, ctx: Context, msg: Message) {
        match msg
            .content
            .trim()
            .split_whitespace()
            .collect::<Vec<_>>()
            .as_slice()
        {
            ["!ping"] => {
                try_reply!(msg.reply(ctx, "Pong!").await)
            }
            ["!list"] => match rcon_cmd(&self.config, "list").await {
                Ok(res) => try_reply!(msg.reply(ctx, res).await),
                Err(e) => try_reply!(msg.reply(ctx, format!("{:?}", e)).await),
            },
            ["!whitelist", username] => {
                match rcon_cmd(&self.config, &format!("whitelist add {}", username)).await {
                    Ok(res) => try_reply!(msg.reply(ctx, res).await),
                    Err(e) => try_reply!(msg.reply(ctx, format!("{:?}", e)).await),
                }
            }
            ["!mc", rest @ ..] => {
                if self.config.admins.contains(&msg.author.id.0) {
                    match rcon_cmd(&self.config, &rest.join(" ")).await {
                        Ok(res) => try_reply!(msg.reply(ctx, res).await),
                        Err(e) => try_reply!(msg.reply(ctx, format!("{:?}", e)).await),
                    }
                } else {
                    try_reply!(msg.reply(ctx, "no :upside_down:").await)
                }
            }
            _ => {}
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let actx = Arc::new(ctx);
        let config = self.config.clone();

        if !self.task_active.load(Ordering::SeqCst) {
            println!("Spawning player list thread");
            self.task_active.store(true, Ordering::SeqCst);
            tokio::spawn(async move {
                loop {
                    match player_list(&config).await {
                        Ok(list) => {
                            if list.is_empty() {
                                actx.set_presence(None, OnlineStatus::Idle).await;
                            } else {
                                actx.set_presence(
                                    Some(Activity::playing(format!("Minecraft: {}", list.len()))),
                                    OnlineStatus::Online,
                                )
                                .await;
                            }
                            println!("{:?}", list);
                        }
                        Err(e) => {
                            actx.set_presence(None, OnlineStatus::DoNotDisturb).await;
                            eprintln!("{:#?}", e)
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                }
            });
        }
    }
}

async fn player_list(config: &Config) -> Result<Vec<String>> {
    Ok(rcon_cmd(config, "list")
        .await?
        .split(':')
        .skip(1)
        .collect::<String>()
        .split(',')
        .filter_map(|u| {
            let trimmed = u.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .collect::<Vec<_>>())
}

async fn rcon_con(config: &Config) -> Result<rcon::Connection> {
    Ok(rcon::Connection::builder()
        .enable_minecraft_quirks(true)
        .connect(
            format!("{}:{}", config.rcon_host, config.rcon_port),
            &config.rcon_password,
        )
        .await?)
}

async fn rcon_cmd(config: &Config, cmd: &str) -> Result<String> {
    let mut con = rcon_con(config).await?;
    Ok(con.cmd(cmd).await?)
}
