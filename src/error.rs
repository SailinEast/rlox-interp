pub struct ErrorReporter {
    pub had_error: bool,
}

impl ErrorReporter {
    pub fn new() -> Self {
        Self { had_error: false }
    }

    pub fn error(&mut self, line: usize, message: &str) {
        self.report(line, "", message);
    }

    pub fn report(&mut self, line: usize, location: &str, message: &str) {
        eprintln!("[line {line}] Error{location}: {message}");
        self.had_error = true;
    }

    pub fn reset(&mut self) {
        self.had_error = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_reporter_starts_without_error() {
        let reporter = ErrorReporter::new();
        assert!(!reporter.had_error);
    }

    #[test]
    fn error_sets_had_error() {
        let mut reporter = ErrorReporter::new();
        reporter.error(1, "Unexpected character.");
        assert!(reporter.had_error);
    }

    #[test]
    fn report_sets_had_error() {
        let mut reporter = ErrorReporter::new();
        reporter.report(4, " at 'foo'", "Expect ';' after expression.");
        assert!(reporter.had_error);
    }

    #[test]
    fn reset_clears_had_error() {
        let mut reporter = ErrorReporter::new();
        reporter.error(1, "Something went wrong.");
        assert!(reporter.had_error);

        reporter.reset();
        assert!(!reporter.had_error);
    }
}
