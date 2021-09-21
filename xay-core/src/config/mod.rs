use std::path::Path;

use once_cell::sync::OnceCell;
use serde::Deserialize;

use crate::proxy::vmess::VMessEndpoint;

#[derive(Deserialize, Debug)]
pub struct Rule {
    pub addr: String,
    pub port: i32,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Endpoint {
    #[serde(rename = "vmess")]
    VMess(VMessEndpoint),
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub port: i32,
    pub endpoints: Vec<Endpoint>,
}

pub static CONFIG: OnceCell<Config> = OnceCell::new();

pub fn get_config() -> &'static Config {
    CONFIG.get().expect("please init config")
}

pub async fn load_config<P: AsRef<Path>>(p: P) -> anyhow::Result<()> {
    let file_content = unsafe { String::from_utf8_unchecked(tokio::fs::read(p.as_ref()).await?) };
    let config = serde_yaml::from_str::<Config>(&file_content)?;

    CONFIG.set(config).unwrap();
    Ok(())
}
