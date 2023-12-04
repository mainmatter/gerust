use crate::util::Environment;
use dotenvy::dotenv;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::Deserialize;
use std::{net::SocketAddr, str::FromStr};

#[derive(Deserialize, Clone, Debug)]
pub struct ServerConfig {
    pub interface: String,
    pub port: i32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct DatabaseConfig {
    pub url: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            interface: String::from("127.0.0.1"),
            port: 3000,
        }
    }
}

impl ServerConfig {
    pub fn get_bind_addr(&self) -> SocketAddr {
        SocketAddr::from_str(format!("{}:{}", self.interface, self.port).as_str()).unwrap_or_else(
            |_| {
                panic!(
                    r#"Could not parse bind addr "{}:{}"!"#,
                    self.interface, self.port
                )
            },
        )
    }
}

pub fn load_config<'a, T>(env: &Environment) -> T
where
    T: Deserialize<'a>,
{
    match env {
        Environment::Development => {
            dotenv().ok();
        }
        Environment::Test => {
            dotenvy::from_filename(".env.test").ok();
        }
        _ => { /* don't use any .env file for production */ }
    }
    dotenv().ok();

    let env_config_file = match env {
        Environment::Development => "development.toml",
        Environment::Production => "production.toml",
        Environment::Test => "test.toml",
    };

    let config: T = Figment::new()
        .merge(Toml::file("config/app.toml"))
        .merge(Toml::file(format!(
            "config/environments/{}",
            env_config_file
        )))
        .merge(Env::prefixed("SERVER_").map(|k| format!("server.{}", k.as_str()).into()))
        .merge(Env::prefixed("DATABASE_").map(|k| format!("database.{}", k.as_str()).into()))
        .extract()
        .expect("Could not read configuration!");

    config
}
