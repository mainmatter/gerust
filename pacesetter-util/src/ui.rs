use std::cmp::max;
use std::io::Write;

pub struct UI<'a> {
    out: &'a mut dyn Write,
    errout: &'a mut dyn Write,
    debug: bool,
    indentation: usize,
    log_prefix: String,
    info_prefix: String,
    success_prefix: String,
    error_prefix: String,
}

impl<'a> UI<'a> {
    pub fn new(
        out: &'a mut dyn Write,
        errout: &'a mut dyn Write,
        color: bool,
        debug: bool,
    ) -> UI<'a> {
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
            out,
            errout,
            debug,
            indentation: 0,
            log_prefix,
            info_prefix,
            success_prefix,
            error_prefix,
        }
    }

    fn indentation(&mut self) -> String {
        "  ".repeat(self.indentation)
    }

    pub fn log(&mut self, msg: &str) {
        let indentation = self.indentation();
        self.out(&format!("{}{}{}", indentation, self.log_prefix, msg));
    }

    pub fn info(&mut self, msg: &str) {
        let indentation = self.indentation();
        self.out(&format!("{}{}{}", indentation, self.info_prefix, msg));
    }

    pub fn success(&mut self, msg: &str) {
        let indentation = self.indentation();
        self.out(&format!("{}{}{}", indentation, self.success_prefix, msg));
    }

    pub fn error(&mut self, msg: &str, e: anyhow::Error) {
        let indentation = self.indentation();
        self.errout(&format!("{}{}{}", indentation, self.error_prefix, msg));
        if self.debug {
            self.errout(&format!("{:?}", e));
        }
    }

    pub fn indent(&mut self) {
        self.indentation += 1;
    }

    pub fn outdent(&mut self) {
        self.indentation = max(0, self.indentation - 1);
    }

    fn out(&mut self, msg: &str) {
        writeln!(&mut self.out, "{}", msg).expect("Cannot write to the output buffer!");
    }

    fn errout(&mut self, msg: &str) {
        writeln!(&mut self.errout, "{}", msg).expect("Cannot write to the error output buffer!");
    }
}
