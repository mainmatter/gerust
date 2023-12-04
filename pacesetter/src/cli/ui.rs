use crate::Environment;

pub enum LogType {
    Info,
    Success,
    Error,
}

pub fn log_per_env(
    env: &Environment,
    log_type: LogType,
    dev_log: &str,
    test_log: &str,
    production_log: &str,
) {
    match env {
        Environment::Development => log(log_type, dev_log),
        Environment::Test => log(log_type, test_log),
        Environment::Production => log(log_type, production_log),
    }
}

pub fn log(log_type: LogType, log: &str) {
    match log_type {
        LogType::Info => println!("ℹ️  {}", log),
        LogType::Success => println!("✅ {}", log),
        LogType::Error => println!("❌ {}", log),
    }
}
