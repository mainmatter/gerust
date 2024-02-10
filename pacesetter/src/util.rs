use anyhow::anyhow;
use std::cmp::PartialEq;
use std::env;
use std::fmt::{Display, Formatter};
use tracing::info;
use tracing_panic::panic_hook;
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

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
    fn test_get_env_default() {
        let env = get_env().unwrap();

        assert_eq!(env, Environment::Development);
    }

    #[test]
    fn test_get_env_with_env_var_dev() {
        env::set_var("APP_ENVIRONMENT", "dev");
        let env = get_env().unwrap();

        assert_eq!(env, Environment::Development);
    }

    #[test]
    fn test_get_env_with_env_var_devevelopment() {
        env::set_var("APP_ENVIRONMENT", "development");
        let env = get_env().unwrap();

        assert_eq!(env, Environment::Development);
    }

    #[test]
    fn test_get_env_with_env_var_prod() {
        env::set_var("APP_ENVIRONMENT", "prod");
        let env = get_env().unwrap();

        assert_eq!(env, Environment::Production);
    }

    #[test]
    fn test_get_env_with_env_var_production() {
        env::set_var("APP_ENVIRONMENT", "production");
        let env = get_env().unwrap();

        assert_eq!(env, Environment::Production);
    }

    #[test]
    fn test_get_env_with_env_var_test() {
        env::set_var("APP_ENVIRONMENT", "test");
        let env = get_env().unwrap();

        assert_eq!(env, Environment::Test);
    }

    #[test]
    fn test_get_env_with_env_var_unknown() {
        env::set_var("APP_ENVIRONMENT", "not-an-env");
        let env = get_env();

        assert!(env.is_err())
    }
}
