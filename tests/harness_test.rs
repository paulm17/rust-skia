// Copyright 2015 Google Inc.; BSD license: tests/LICENSE.skia.
#![allow(non_snake_case)]
#[path = "support/harness.rs"]
mod harness;
mod support;
use harness::{CtsEnforcement, Registry, RunMode};
use support::{Reporter, reporter_assert};

#[allow(useless_ptr_null_checks)] // Faithful upstream non-null reporter assertion.
fn normal(reporter: &mut Reporter) {
    // The upstream assertion tests the supplied non-null reporter pointer.
    reporter_assert!(reporter, !std::ptr::from_ref(reporter).is_null());
}
fn false_condition(reporter: &mut Reporter) {
    reporter.record(
        false,
        "DEF_CONDITIONAL_TEST executed a test with a false condition",
        file!(),
        line!(),
    );
}
fn registry() -> Registry {
    let mut registry = Registry::default();
    registry.register("TestNormal", normal, true);
    registry.register("TestTrueCondition", normal, true); // upstream 1 == 1
    registry.register("TestFalseCondition", false_condition, false); // upstream 1 == 0
    registry
}
fn run(name: &str) {
    let mut reporter = Reporter::default();
    (registry().find(name).expect("registered CPU test"))(&mut reporter);
    reporter.finish();
}
#[test]
fn TestNormal() {
    run("TestNormal");
}
#[test]
fn TestTrueCondition() {
    run("TestTrueCondition");
}
#[test]
fn TestCtsEnforcement() {
    use RunMode::*;
    let mut reporter = Reporter::default();
    let mut verify = |e: CtsEnforcement, api, mode| {
        reporter_assert!(reporter, e.eval(api) == mode);
    };
    let e1 = CtsEnforcement::new(i32::MAX);
    verify(e1, 0, Skip);
    verify(e1, 33, Skip);
    verify(e1, 202604, Skip);
    let e2 = CtsEnforcement::new(33);
    verify(e2, 0, Skip);
    verify(e2, 33, RunStrict);
    verify(e2, 202604, RunStrict);
    let e3 = CtsEnforcement::new(202604);
    verify(e3, 0, Skip);
    verify(e3, 33, Skip);
    verify(e3, 202604, RunStrict);
    let e4 = CtsEnforcement::new(202604).with_workarounds(33);
    verify(e4, 0, Skip);
    verify(e4, 33, RunWithWorkarounds);
    verify(e4, 202604, RunStrict);
    reporter.finish();
}
#[test]
fn supplemental_registry_excludes_false_condition_and_preserves_head_order() {
    let registry = registry();
    assert_eq!(registry.names(), ["TestTrueCondition", "TestNormal"]);
    assert!(registry.find("TestFalseCondition").is_none());
    // Prove that the excluded body retains its intended failure.
    assert!(
        std::panic::catch_unwind(|| {
            let mut reporter = Reporter::default();
            false_condition(&mut reporter);
            reporter.finish();
        })
        .is_err()
    );
}
