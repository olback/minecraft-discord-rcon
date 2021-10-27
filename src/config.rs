use {
    serde::Deserialize,
    std::{fs, io},
};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub rcon_host: String,
    pub rcon_port: u16,
    pub rcon_password: String,
    pub discord_token: String,
}

impl Config {
    pub fn load() -> Result<Self, io::Error> {
        let file_contents = fs::read_to_string("Config.toml")?;
        Ok(toml::from_str::<Self>(&file_contents)?)
    }
}
