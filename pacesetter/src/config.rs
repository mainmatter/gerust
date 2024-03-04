use crate::util::Environment;
use anyhow::Context;
use dotenvy::dotenv;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// The server configuration.
///
/// This struct keeps all settings specific to the server – currently that is the interface the server binds to
/// but more might be added in the future. The struct is provided pre-defined by Pacesetter and cannot be changed. It
/// **must** be used for the `server` field in the application-specific `Config` struct:
///
/// ```rust
/// #[derive(Deserialize, Clone, Debug)]
/// pub struct Config {
///     #[serde(default)]
///     pub server: ServerConfig,
///     pub database: DatabaseConfig,
///     // add your config settings here…
/// }
/// ```
#[derive(Deserialize, Serialize, Clone, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ServerConfig {
    /// The port to bind to, e.g. 3000
    pub port: u16,

    /// The ip to bind to, e.g. 127.0.0.1 or ::1
    pub ip: IpAddr,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port: 3000,
        }
    }
}

impl ServerConfig {
    /// Returns the full address the server binds to, including both the ip and port.
    ///
    /// This can be used when creating a TCP Listener:
    ///
    /// ```rust
    /// let config: Config = load_config(Environment::Development);
    /// let listener = TcpListener::bind(&config.server.addr).await?;
    /// serve(listener, app.into_make_service()).await?;
    ///  ```
    pub fn addr(&self) -> SocketAddr {
        SocketAddr::new(self.ip, self.port)
    }
}

/// The database configuration.
///
/// This struct keeps all settings specific to the database – currently that is the database URL to use to connect to the database
/// but more might be added in the future. The struct is provided pre-defined by Pacesetter and cannot be changed. It
/// **must** be used for the `database` field in the application-specific `Config` struct:
///
/// ```rust
/// #[derive(Deserialize, Clone, Debug)]
/// pub struct Config {
///     #[serde(default)]
///     pub server: ServerConfig,
///     pub database: DatabaseConfig,
///     // add your config settings here…
/// }
/// ```
#[derive(Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct DatabaseConfig {
    /// The URL to use to connect to the database, e.g. "postgresql://user:password@localhost:5432/database"
    pub url: String,
}

/// Loads the application's configuration.
///
/// Examples:
/// ```rust
/// use my_app_config::Config;
/// use pacesetter::{get_env, load_config};
///
/// let env = get_env();
/// let config: Config = load_config(&env);
/// ```
///
/// The returned `Config` struct is defined in the `config` crate, e.g.
///
/// ```rust
/// #[derive(Deserialize, Debug)]
/// pub struct Config {
///     pub server: ServerConfig,
///     pub database: DatabaseConfig,
///
///     pub app_setting: String,
/// }
/// ```
///
/// `ServerConfig` and `DatabaseConfig` are always populated based on environment variables:
///
/// * SERVER_ID and SERVER_PORT for `ServerConfig`
/// * DATABASE_URL for `DatabaseConfig`
///
/// For the Development and Test environment, this function will define environment variables
/// based on the contents of the `.env` and `.env.test` files respectively. Any `.evn` files are
/// ignored in the production environment.
///
/// All application-specific configuration is read from the respective configuration files in
/// the `config/app.toml` and `config/environments/<environment>.toml` files such that settings
/// in the latter override settings in the former.
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
        .merge(Env::prefixed("APP_").split("__"))
        .extract()
        .context("Could not read configuration!")?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[derive(Deserialize, PartialEq, Debug)]
    pub struct Config {
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

            jail.set_env("APP_SERVER__IP", "127.0.0.1");
            jail.set_env("APP_SERVER__PORT", "3000");
            jail.set_env(
                "APP_DATABASE__URL",
                "postgresql://user:pass@localhost:5432/my_app",
            );
            let config = load_config::<Config>(&Environment::Development).unwrap();

            assert_eq!(
                config,
                Config {
                    server: ServerConfig {
                        ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
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

            jail.set_env("APP_SERVER__IP", "127.0.0.1");
            jail.set_env("APP_SERVER__PORT", "3000");
            jail.set_env(
                "APP_DATABASE__URL",
                "postgresql://user:pass@localhost:5432/my_app",
            );
            let config = load_config::<Config>(&Environment::Test).unwrap();

            assert_eq!(
                config,
                Config {
                    server: ServerConfig {
                        ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
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

            jail.set_env("APP_SERVER__IP", "127.0.0.1");
            jail.set_env("APP_SERVER__PORT", "3000");
            jail.set_env(
                "APP_DATABASE__URL",
                "postgresql://user:pass@localhost:5432/my_app",
            );
            let config = load_config::<Config>(&Environment::Production).unwrap();

            assert_eq!(
                config,
                Config {
                    server: ServerConfig {
                        ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
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
