use crate::util::Environment;
use anyhow::Context;
use dotenvy::dotenv;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::Deserialize;
use std::{net::SocketAddr, str::FromStr};

#[derive(Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ServerConfig {
    pub interface: String,
    pub port: i32,
}

#[derive(Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(PartialEq))]
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

pub fn load_config<'a, T>(env: &Environment) -> Result<T, anyhow::Error>
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
        .context("Could not read configuration!")?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize, PartialEq, Debug)]
    pub struct Config {
        #[serde(default)]
        pub server: ServerConfig,
        pub database: DatabaseConfig,

        pub app_setting: String,
    }

    #[test]
    fn test_load_config_development() {
        figment::Jail::expect_with(|jail| {
            let config_dir = jail.create_dir("config")?;
            jail.create_file(
                config_dir.join("app.toml"),
                r#"
                app_setting = "Just a TOML App!"
            "#,
            )?;
            let environments_dir = jail.create_dir("config/environments")?;
            jail.create_file(
                environments_dir.join("development.toml"),
                r#"
                app_setting = "override!"
            "#,
            )?;

            jail.set_env("SERVER_INTERFACE", "localhost");
            jail.set_env("SERVER_PORT", "3000");
            jail.set_env(
                "DATABASE_URL",
                "postgresql://user:pass@localhost:5432/my_app",
            );
            let config = load_config::<Config>(&Environment::Development).unwrap();

            assert_eq!(
                config,
                Config {
                    server: ServerConfig {
                        interface: String::from("localhost"),
                        port: 3000,
                    },
                    database: DatabaseConfig {
                        url: String::from("postgresql://user:pass@localhost:5432/my_app"),
                    },

                    app_setting: String::from("override!"),
                }
            );

            Ok(())
        });
    }

    #[test]
    fn test_load_config_test() {
        figment::Jail::expect_with(|jail| {
            let config_dir = jail.create_dir("config")?;
            jail.create_file(
                config_dir.join("app.toml"),
                r#"
                app_setting = "Just a TOML App!"
            "#,
            )?;
            let environments_dir = jail.create_dir("config/environments")?;
            jail.create_file(
                environments_dir.join("test.toml"),
                r#"
                app_setting = "override!"
            "#,
            )?;

            jail.set_env("SERVER_INTERFACE", "localhost");
            jail.set_env("SERVER_PORT", "3000");
            jail.set_env(
                "DATABASE_URL",
                "postgresql://user:pass@localhost:5432/my_app",
            );
            let config = load_config::<Config>(&Environment::Test).unwrap();

            assert_eq!(
                config,
                Config {
                    server: ServerConfig {
                        interface: String::from("localhost"),
                        port: 3000,
                    },
                    database: DatabaseConfig {
                        url: String::from("postgresql://user:pass@localhost:5432/my_app"),
                    },

                    app_setting: String::from("override!"),
                }
            );

            Ok(())
        });
    }

    #[test]
    fn test_load_config_production() {
        figment::Jail::expect_with(|jail| {
            let config_dir = jail.create_dir("config")?;
            jail.create_file(
                config_dir.join("app.toml"),
                r#"
                app_setting = "Just a TOML App!"
            "#,
            )?;
            let environments_dir = jail.create_dir("config/environments")?;
            jail.create_file(
                environments_dir.join("production.toml"),
                r#"
                app_setting = "override!"
            "#,
            )?;

            jail.set_env("SERVER_INTERFACE", "localhost");
            jail.set_env("SERVER_PORT", "3000");
            jail.set_env(
                "DATABASE_URL",
                "postgresql://user:pass@localhost:5432/my_app",
            );
            let config = load_config::<Config>(&Environment::Production).unwrap();

            assert_eq!(
                config,
                Config {
                    server: ServerConfig {
                        interface: String::from("localhost"),
                        port: 3000,
                    },
                    database: DatabaseConfig {
                        url: String::from("postgresql://user:pass@localhost:5432/my_app"),
                    },

                    app_setting: String::from("override!"),
                }
            );

            Ok(())
        });
    }
}
