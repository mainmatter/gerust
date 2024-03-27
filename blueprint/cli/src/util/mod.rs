use anyhow::anyhow;
use {{crate_name}}_config::Environment;

pub mod ui;

pub fn parse_env(s: &str) -> Result<Environment, &'static str> {
    let s = &s.to_lowercase();
    match parse_env_internal(s) {
        Ok(env) => Ok(env),
        Err(_) => Err("Cannot parse environment!"),
    }
}

fn parse_env_internal(env: &str) -> Result<Environment, anyhow::Error> {
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
