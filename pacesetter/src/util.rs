use anyhow::anyhow;
use std::cmp::PartialEq;
use std::env;
use std::fmt::{Display, Formatter};
use tracing::info;
use tracing_panic::panic_hook;
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// The environment the application runs in – either Development, Production, or Test.
///
/// Aspects of the application might behave differently based on the currently active environment.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Development,
    Production,
    Test,
}

impl Display for Environment {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Environment::Development => write!(f, "development"),
            Environment::Production => write!(f, "production"),
            Environment::Test => write!(f, "test"),
        }
    }
}

/// Gets the [`Environment`] from the `APP_ENVIRONMENT` environment variable or defaults to [`Environment::Development`] if that is not set.
///
/// Example
/// ```rust
/// assert_eq(env::var("APP_ENVIRONMENT"), Ok("dev"));
/// let env = get_env();
/// assert_eq(env, Environment::Development);
/// ```
///
/// "dev" and "development" are parsed as [`Environment::Development`], "prod" and "production" are parsed as [`Environment::Production`] and "test" is parsed as [`Environment::Test`]. Parsing environments is case-insensitive.
pub fn get_env() -> Result<Environment, anyhow::Error> {
    // TODO: come up with a better name for the env var!
    match env::var("APP_ENVIRONMENT") {
        Ok(val) => {
            info!(r#"Setting environment from APP_ENVIRONMENT: "{}""#, val);
            parse_env(&val)
        }
        Err(_) => {
            info!("Defaulting to environment: development");
            Ok(Environment::Development)
        }
    }
}

pub(crate) fn parse_env(env: &str) -> Result<Environment, anyhow::Error> {
    let env = &env.to_lowercase();
    match env.as_str() {
        "dev" => Ok(Environment::Development),
        "development" => Ok(Environment::Development),
        "test" => Ok(Environment::Test),
        "prod" => Ok(Environment::Production),
        "production" => Ok(Environment::Production),
        unknown => Err(anyhow!(r#"Unknown environment: "{}"!"#, unknown)),
    }
}

/// Initializes tracing.
///
/// This function
///
/// * registers a [`tracing_subscriber::fmt::Subscriber`]
/// * registers a [`tracing_panic::panic_hook`]
///
/// The function respects the `RUST_LOG` if set or defaults to filtering spans and events with level [`tracing_subscriber::filter::LevelFilter::INFO`] and higher.
pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    std::panic::set_hook(Box::new(panic_hook));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_env_dev() {
        let env = parse_env("dev").unwrap();

        assert_eq!(env, Environment::Development);
    }

    #[test]
    fn test_parse_env_dev_all_caps() {
        let env = parse_env("DEV").unwrap();

        assert_eq!(env, Environment::Development);
    }

    #[test]
    fn test_parse_env_devevelopment() {
        let env = parse_env("development").unwrap();

        assert_eq!(env, Environment::Development);
    }

    #[test]
    fn test_parse_env_prod() {
        let env = parse_env("prod").unwrap();

        assert_eq!(env, Environment::Production);
    }

    #[test]
    fn test_parse_env_production() {
        let env = parse_env("production").unwrap();

        assert_eq!(env, Environment::Production);
    }

    #[test]
    fn test_parse_env_capitalized_production() {
        let env = parse_env("Production").unwrap();

        assert_eq!(env, Environment::Production);
    }

    #[test]
    fn test_parse_env_test() {
        let env = parse_env("test").unwrap();

        assert_eq!(env, Environment::Test);
    }

    #[test]
    fn test_parse_env_weirdly_cased_test() {
        let env = parse_env("tEsT").unwrap();

        assert_eq!(env, Environment::Test);
    }

    #[test]
    fn test_parse_env_invalid() {
        let env = parse_env("not-an-env");

        assert!(env.is_err())
    }
}
