use crate::Environment;
use clap::ArgMatches;

pub fn parse_env(sub_matches: &ArgMatches) -> Environment {
    let env = sub_matches
        .get_one::<String>("env")
        .map(|s| s.as_str())
        .unwrap_or("development");

    if env == "test" {
        Environment::Test
    } else if env == "production" {
        Environment::Production
    } else {
        Environment::Development
    }
}
