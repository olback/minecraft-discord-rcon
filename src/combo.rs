use {
    crate::{config::Config, error::Error},
    discord::{model::ChannelId, Discord},
    std::time::Instant,
};

pub struct Combo {
    last: Instant,
    r: rcon::Connection,
    d: Discord,
}

impl Combo {
    pub async fn new(conf: &Config) -> Result<Self, Error> {
        let discord = Discord::from_bot_token(&conf.discord_token)?;
        println!("Connected to Discrod");

        let com = Combo {
            last: Instant::now(),
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

        Ok(com)
    }

    pub fn is_old(&mut self) -> bool {
        self.last.elapsed().as_secs() > 60 //TODO how old is old?
    }

    pub async fn is_dead(&mut self) -> bool {
        //TODO set bot status depending on server connection?
        match self.rcon_cmd("me").await {
            Ok(_) => false,
            Err(_) => true,
        }
    }

    pub async fn rcon_reconnect(&mut self, conf: &Config) {
        match rcon::Connection::builder()
            .enable_minecraft_quirks(true)
            .connect(
                format!("{}:{}", &conf.rcon_host, &conf.rcon_port),
                &conf.rcon_password,
            )
            .await
        {
            Ok(new_r) => {
                self.r = new_r;
                self.last = Instant::now();
            }
            Err(_) => {}
        }
    }

    pub fn discord_connection(&mut self) -> Result<discord::Connection, Error> {
        let (discord_conn, _) = self.d.connect()?;
        Ok(discord_conn)
    }

    pub async fn rcon_cmd(&mut self, cmd: &str) -> rcon::Result<String> {
        self.r.cmd(cmd).await
    }

    pub async fn send_text(&mut self, command: &str, channel: ChannelId, msg: &str) {
        match self.rcon_cmd(command).await {
            Ok(_) => {
                drop(self.d.send_message(channel, msg, "", false));
                self.last = Instant::now();
            }
            Err(e) => self.send_error(channel, e).await,
        }
    }

    pub async fn send_rcon(&mut self, command: &str, channel: ChannelId) {
        match self.rcon_cmd(command).await {
            Ok(res) => {
                drop(self.d.send_message(channel, &res, "", false));
                self.last = Instant::now();
            }
            Err(e) => self.send_error(channel, e).await,
        }
    }

    async fn send_error(&mut self, channel: ChannelId, e: rcon::Error) {
        match e {
            //TODO handle IO errors
            _ => drop(self.d.send_message(
                channel,
                &format!("Error processing request: \"{}\"", e),
                "",
                false,
            )),
        }
    }
}
