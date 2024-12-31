use std::io::Write;

/// A console UI session
pub struct UI<'a> {
    stdout: &'a mut dyn Write,
    errout: &'a mut dyn Write,
    debug: bool,
    indentation: usize,
    log_prefix: String,
    info_prefix: String,
    success_prefix: String,
    error_prefix: String,
}

impl<'a> UI<'a> {
    /// Create a new console UI session with given standard and error outputs.
    ///
    /// If `color` is `true`, output will be colored and formatted with emojis depending on the type of output. If `debug` is `true`, additional output will be printed in some cases, e.g. stack traces for errors.
    ///
    /// Example:
    /// ```
    /// let mut stdout = std::io::stdout();
    /// let mut stderr = std::io::stderr();
    /// let mut ui = UI::new(&mut stdout, &mut stderr, true, true);
    /// ```
    pub fn new(
        stdout: &'a mut dyn Write,
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
            stdout,
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

    /// Prints a general message.
    pub fn log(&mut self, msg: &str) {
        let indentation = self.indentation();
        self.out(&format!("{}{}{}", indentation, self.log_prefix, msg));
    }

    /// Prints an info message.
    ///
    /// If color output is enabled (see [`UI::new`]), the message will be formatted.
    pub fn info(&mut self, msg: &str) {
        let indentation = self.indentation();
        self.out(&format!("{}{}{}", indentation, self.info_prefix, msg));
    }

    /// Prints a success message.
    ///
    /// If color output is enabled (see [`UI::new`]), the message will be formatted.
    pub fn success(&mut self, msg: &str) {
        let indentation = self.indentation();
        self.out(&format!("{}{}{}", indentation, self.success_prefix, msg));
    }

    /// Prints an error message.
    ///
    /// If color output is enabled (see [`UI::new`]), the message will be formatted. If debug output is enabled (see [`UI::new`]), the error's stack trace will be printed as well.
    pub fn error(&mut self, msg: &str, e: &anyhow::Error) {
        let indentation = self.indentation();
        self.errout(&format!("{}{}{}", indentation, self.error_prefix, msg));
        if self.debug {
            self.errout(&format!("{:?}", e));
        }
    }

    /// Increases indentation of subsequently printed messages by 2 spaces.
    ///
    /// Example:
    /// ```
    /// let mut stdout = std::io::stdout();
    /// let mut stderr = std::io::stderr();
    /// let mut ui = UI::new(&mut stdout, &mut stderr, true, true);
    ///
    /// ui.success("All migrations were applied");
    /// ui.indent();
    /// ui.log("Ran 5 migrations");
    ///
    /// // prints:
    /// // > ✅ All migrations were applied
    /// // >    Ran 5 migrations
    /// ```
    pub fn indent(&mut self) {
        self.indentation += 1;
    }

    /// Decreases indentation of subsequently printed messages by 2 spaces.
    ///
    /// If the indentation level is 0 already, this does nothing.
    ///
    /// Example:
    /// ```
    /// let mut stdout = std::io::stdout();
    /// let mut stderr = std::io::stderr();
    /// let mut ui = UI::new(&mut stdout, &mut stderr, true, true);
    ///
    /// ui.success("All migrations were applied");
    /// ui.indent();
    /// ui.log("Ran 5 migrations");
    /// ui.outdent();
    /// ui.info("Now seeding database…")
    ///
    /// // prints:
    /// // > ✅ All migrations were applied
    /// // >    Ran 5 migrations
    /// // > ℹ️ Now seeding database…
    /// ```
    pub fn outdent(&mut self) {
        if self.indentation > 0 {
            self.indentation -= 1;
        }
    }

    fn out(&mut self, msg: &str) {
        writeln!(&mut self.stdout, "{}", msg).expect("Cannot write to the output buffer!");
    }

    fn errout(&mut self, msg: &str) {
        writeln!(&mut self.errout, "{}", msg).expect("Cannot write to the error output buffer!");
    }
}

#[cfg(test)]
mod tests {
    use super::UI;
    use anyhow::anyhow;
    use insta::assert_snapshot;

    #[test]
    fn test_no_color() {
        let mut stdout = create_buffer();
        let mut stderr = create_buffer();
        let mut ui = UI::new(&mut stdout, &mut stderr, false, false);
        ui.log("a general message");
        ui.info("an info message");
        ui.success("a success message ✓");
        ui.error("an error message :(", &anyhow!("oh no…"));

        let output = read_buffer(stdout);
        let error_output = read_buffer(stderr);

        assert_snapshot!(output, @r###"
        a general message
        an info message
        a success message ✓
        "###);
        assert_snapshot!(error_output, @r###"
        an error message :(
        "###);
    }

    #[test]
    fn test_color() {
        let mut stdout = create_buffer();
        let mut stderr = create_buffer();
        let mut ui = UI::new(&mut stdout, &mut stderr, true, false);
        ui.log("a general message");
        ui.info("an info message");
        ui.success("a success message ✓");
        ui.error("an error message :(", &anyhow!("oh no…"));

        let output = read_buffer(stdout);
        let error_output = read_buffer(stderr);

        assert_snapshot!(output, @r###"
           a general message
        ℹ️  an info message
        ✅ a success message ✓
        "###);
        assert_snapshot!(error_output, @r###"
        ❌ an error message :(
        "###);
    }

    #[test]
    fn test_indentation() {
        let mut stdout = create_buffer();
        let mut stderr = create_buffer();
        let mut ui = UI::new(&mut stdout, &mut stderr, false, false);
        ui.log("a log message");
        ui.indent();
        ui.log("an indented message");
        ui.indent();
        ui.indent();
        ui.indent();
        ui.log("more indentation");
        ui.outdent();
        ui.outdent();
        ui.log("less indentation");
        ui.outdent();
        ui.outdent();
        ui.outdent(); // we can call outdent more often than indent – it just stops at 0 indentation
        ui.log("no indentation");

        let output = read_buffer(stdout);

        assert_snapshot!(output, @r###"
           a log message
             an indented message
                   more indentation
               less indentation
           no indentation
        "###);
    }

    fn create_buffer() -> std::io::BufWriter<Vec<u8>> {
        std::io::BufWriter::new(Vec::new())
    }

    fn read_buffer(buffer: std::io::BufWriter<Vec<u8>>) -> String {
        let bytes = buffer.into_inner().unwrap();

        String::from_utf8(bytes).unwrap()
    }
}
