use crate::Environment;

pub fn parse_env(s: &str) -> Result<Environment, &'static str> {
    let s = &s.to_lowercase();
    match s.as_str() {
        "dev" => Ok(Environment::Development),
        "development" => Ok(Environment::Development),
        "test" => Ok(Environment::Test),
        "prod" => Ok(Environment::Production),
        "production" => Ok(Environment::Production),
        _ => Err("Cannot parse environment!"),
    }
}
