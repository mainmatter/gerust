use std::env;
use std::fmt::{Display, Formatter, Result};
use tracing::info;
use tracing_panic::panic_hook;
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Clone)]
pub enum Environment {
    Development,
    Production,
    Test,
}

impl Display for Environment {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Environment::Development => write!(f, "development"),
            Environment::Production => write!(f, "production"),
            Environment::Test => write!(f, "test"),
        }
    }
}

pub fn get_env() -> Environment {
    // TODO: come up with a better name for the env var!
    match env::var("APP_ENVIRONMENT") {
        Ok(val) => {
            let env = match val.to_lowercase().as_str() {
                "dev" | "development" => Environment::Development,
                "prod" | "production" => Environment::Production,
                "test" => Environment::Test,
                unknown => {
                    panic!(r#"Unknown environment: "{}"!"#, unknown);
                }
            };
            info!("Setting environment from APP_ENVIRONMENT: {}", env);
            env
        }
        Err(_) => {
            info!("Defaulting to environment: development");
            Environment::Development
        }
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
