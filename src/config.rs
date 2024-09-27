use anyhow::Result;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct Config {
    pub bark_urls: Option<Vec<String>>,
    pub ntfy_urls: Option<Vec<String>>,
    pub least_needed_gpu: Option<i32>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config = std::fs::read_to_string("config.toml")?;
        toml::from_str(config.as_str()).map_err(|e| e.into())
    }
}
