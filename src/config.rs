use lazy_static::lazy_static;
use std::env;

lazy_static! {
    static ref CONTROLLER_CONFIG: Config = Config::from_env();
}

pub fn get_config() -> &'static Config {
    &CONTROLLER_CONFIG
}

#[derive(Debug, Clone)]
pub struct Config {
    pub image: String,
    pub open_head_image: String,
    pub sidecar_image: String,
    pub configmap: String,
    pub blockfrost_key: String,
    pub external_domain: String,
    pub external_port: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            image: env::var("IMAGE").unwrap_or("ghcr.io/cardano-scaling/hydra-node".into()),
            open_head_image: env::var("OPEN_HEAD_IMAGE").expect("Missing OPEN_HEAD_IMAGE env var"),
            sidecar_image: env::var("SIDECAR_IMAGE").expect("Missing SIDECAR_IMAGE env var"),
            configmap: env::var("CONFIGMAP").expect("Missing CONFIGMAP env var"),
            blockfrost_key: env::var("BLOCKFROST_KEY").expect("Missing BLOCKFROST_KEY env var"),
            external_domain: env::var("EXTERNAL_DOMAIN").expect("Missing EXTERNAL_DOMAIN env var."),
            external_port: env::var("EXTERNAL_PORT").expect("Missing EXTERNAL_PORT env var."),
        }
    }
}
