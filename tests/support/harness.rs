// Copyright 2009 The Android Open Source Project; Copyright 2022 Google LLC.
// BSD license: tests/LICENSE.skia. Reached Registry/CTS support only.
use super::support::Reporter;
#[derive(Clone, Copy)]
pub struct CtsEnforcement {
    strict: i32,
    workarounds: i32,
}
#[derive(Debug, PartialEq)]
pub enum RunMode {
    Skip,
    RunWithWorkarounds,
    RunStrict,
}
impl CtsEnforcement {
    pub fn new(strict: i32) -> Self {
        Self {
            strict,
            workarounds: i32::MAX,
        }
    }
    pub fn with_workarounds(mut self, version: i32) -> Self {
        debug_assert!(version <= self.strict);
        self.workarounds = version;
        self
    }
    pub fn eval(self, api: i32) -> RunMode {
        if api >= self.strict {
            RunMode::RunStrict
        } else if api >= self.workarounds {
            RunMode::RunWithWorkarounds
        } else {
            RunMode::Skip
        }
    }
}
pub type CpuProc = fn(&mut Reporter);
#[derive(Default)]
pub struct Registry {
    entries: Vec<(&'static str, CpuProc)>,
}
impl Registry {
    pub fn register(&mut self, name: &'static str, proc: CpuProc, condition: bool) {
        if condition {
            self.entries.insert(0, (name, proc));
        }
    }
    pub fn find(&self, name: &str) -> Option<CpuProc> {
        self.entries
            .iter()
            .find(|entry| entry.0 == name)
            .map(|entry| entry.1)
    }
    pub fn names(&self) -> Vec<&'static str> {
        self.entries.iter().map(|entry| entry.0).collect()
    }
}
