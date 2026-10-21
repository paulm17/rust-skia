// Copyright 2011 Google Inc.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.skia file in the tests directory.

#[derive(Default)]
pub struct Reporter {
    failures: Vec<String>,
    assertions: usize,
}

impl Reporter {
    pub fn record(&mut self, condition: bool, expression: &str, file: &str, line: u32) {
        self.assertions += 1;
        if !condition {
            self.failures.push(format!("{file}:{line}: {expression}"));
        }
    }

    pub fn finish(self) {
        eprintln!(
            "reporter assertions={} failures={}",
            self.assertions,
            self.failures.len()
        );
        assert!(
            self.failures.is_empty(),
            "{} assertion(s) failed:\n{}",
            self.failures.len(),
            self.failures.join("\n")
        );
    }
}

macro_rules! reporter_assert {
    ($reporter:expr, $condition:expr) => {
        $reporter.record($condition, stringify!($condition), file!(), line!())
    };
}
pub(crate) use reporter_assert;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reporter_collects_failures_and_evaluates_once() {
        let mut reporter = Reporter::default();
        let mut evaluations = 0;
        reporter_assert!(reporter, {
            evaluations += 1;
            false
        });
        reporter_assert!(reporter, false);
        reporter_assert!(reporter, true);
        assert_eq!(evaluations, 1);
        assert_eq!(reporter.failures.len(), 2);
        assert!(
            reporter
                .failures
                .iter()
                .all(|failure| failure.contains(file!()))
        );
        let failure = std::panic::catch_unwind(|| reporter.finish()).unwrap_err();
        let message = failure.downcast_ref::<String>().unwrap();
        assert!(message.contains("2 assertion(s) failed"));
        Reporter::default().finish();
    }
}
