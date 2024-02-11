use pacesetter::config::{DatabaseConfig, ServerConfig};
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    // add your config settings here…
}
