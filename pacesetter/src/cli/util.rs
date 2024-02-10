use crate::util::parse_env as parse_env_internal;
use crate::Environment;

pub fn parse_env(s: &str) -> Result<Environment, &'static str> {
    let s = &s.to_lowercase();
    match parse_env_internal(s) {
        Ok(env) => Ok(env),
        Err(_) => Err("Cannot parse environment!"),
    }
}
