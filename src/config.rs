use anyhow;
use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub kaist: KaistConfig,
    pub gmail: GmailConfig,
}

#[derive(Deserialize)]
pub struct KaistConfig {
    pub id: String,
    pub pw: String,
}

#[derive(Deserialize)]
pub struct GmailConfig {
    pub id: String,
    pub imap_token: String,
}

pub fn load(path: &str) -> Result<Config, anyhow::Error> {
    let text = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&text)?;
    Ok(config)
}