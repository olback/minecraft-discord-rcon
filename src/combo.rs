use discord::{model::ChannelId, Discord};

pub struct Combo {
    pub r: rcon::Connection,
    pub d: Discord,
}

impl Combo {
    fn send_error(&mut self, channel: ChannelId, e: rcon::Error) {
        drop(self.d.send_message(channel, &format!("{}", e), "", false))
    }

    pub async fn send_text(&mut self, command: &str, channel: ChannelId, msg: &str) {
        match self.r.cmd(command).await {
            Ok(_) => drop(self.d.send_message(channel, msg, "", false)),
            Err(e) => self.send_error(channel, e),
        }
    }

    pub async fn send_rcon(&mut self, command: &str, channel: ChannelId) {
        match self.r.cmd(command).await {
            Ok(res) => drop(self.d.send_message(channel, &res, "", false)),
            Err(e) => self.send_error(channel, e),
        }
    }
}
