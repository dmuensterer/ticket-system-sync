use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::sync::OnceLock;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub jira: JiraConfig,
    pub zammad: ZammadConfig,
    pub db_path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JiraConfig {
    pub endpoint: String,
    pub username: String,
    pub token: String,
    pub project_id: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ZammadConfig {
    pub endpoint: String,
    pub token: String,
    pub group: String,
    pub customer: String,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn init() -> Result<()> {
    let config_str = fs::read_to_string("config.yml")?;
    let config: Config = serde_yaml::from_str(&config_str)?;
    CONFIG.set(config).unwrap();
    Ok(())
}

pub fn get() -> &'static Config {
    CONFIG.get().expect("Config not initialized")
}

pub fn get_jira() -> &'static JiraConfig {
    &get().jira
}

pub fn get_zammad() -> &'static ZammadConfig {
    &get().zammad
}
