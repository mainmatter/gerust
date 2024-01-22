use std::cmp::max;

#[allow(dead_code)]
pub enum LogType {
    Info,
    Success,
    Error,
}

pub struct UI {
    debug: bool,
    indentation: usize,
    log_prefix: String,
    info_prefix: String,
    success_prefix: String,
    error_prefix: String,
}

impl UI {
    pub fn new(color: bool, debug: bool) -> UI {
        let info_prefix = if color {
            String::from("ℹ️  ")
        } else {
            String::from("")
        };
        let success_prefix = if color {
            String::from("✅ ")
        } else {
            String::from("")
        };
        let error_prefix = if color {
            String::from("❌ ")
        } else {
            String::from("")
        };
        let log_prefix = if color {
            String::from("   ")
        } else {
            String::from("")
        };

        UI {
            debug,
            indentation: 0,
            log_prefix,
            info_prefix,
            success_prefix,
            error_prefix,
        }
    }

    fn indentation(&self) -> String {
        "  ".repeat(self.indentation)
    }

    pub fn log(&self, msg: &str) {
        println!("{}{}{}", self.indentation(), self.log_prefix, msg);
    }

    pub fn info(&self, msg: &str) {
        println!("{}{}{}", self.indentation(), self.info_prefix, msg);
    }

    pub fn success(&self, msg: &str) {
        println!("{}{}{}", self.indentation(), self.success_prefix, msg);
    }

    pub fn error(&self, msg: &str, e: anyhow::Error) {
        eprintln!("{}{}{}", self.indentation(), self.error_prefix, msg);
        if self.debug {
            eprintln!("{:?}", e);
        }
    }

    pub fn indent(&mut self) {
        self.indentation += 1;
    }

    pub fn outdent(&mut self) {
        self.indentation = max(0, self.indentation - 1);
    }
}

pub fn log(log_type: LogType, log: &str) {
    match log_type {
        LogType::Info => println!("ℹ️  {}", log),
        LogType::Success => println!("✅ {}", log),
        LogType::Error => println!("❌ {}", log),
    }
}
