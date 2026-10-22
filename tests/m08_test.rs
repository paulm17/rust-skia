// Copyright 2014 Google Inc. BSD: tests/LICENSE.skia.
// Skia b81c69985fd14bb7bfbd356b7618633ce07dc1a4, tests/AsADashTest.cpp.
// Correspondence and adaptations: m08/README.md.
#![allow(non_snake_case)]
mod support;
use rust_skia::*;
use support::{Reporter, reporter_assert as check};

// tests/AsADashTest.cpp:19
#[test]
fn AsADashTest_noneDash() {
    let mut reporter = Reporter::default();
    let pe = SkCornerPathEffect::make(1.0).unwrap();
    // The original unused, default-initialized DashInfo has no Rust Default;
    // keep the unused local with an empty owned interval vector.
    let _info = DashInfo {
        f_intervals: Vec::new(),
        f_phase: 0.0,
    };
    let dashInfo = pe.base().as_a_dash();
    check!(reporter, dashInfo.is_none());
    reporter.finish();
}

// tests/AsADashTest.cpp:27
#[test]
fn AsADashTest_nullInfo() {
    let mut reporter = Reporter::default();
    let inIntervals = [4.0, 2.0, 1.0, 3.0];
    let phase = 2.0;
    let pe = SkDashPathEffect::make(&inIntervals, phase).unwrap();
    let dashInfo = pe.base().as_a_dash();
    check!(reporter, dashInfo.is_some());
    reporter.finish();
}

// tests/AsADashTest.cpp:36
#[test]
fn AsADashTest_usingDash() {
    let mut reporter = Reporter::default();
    let inIntervals = [4.0, 2.0, 1.0, 3.0];
    let totalIntSum = 10.0;
    let phase = 2.0;
    let pe = SkDashPathEffect::make(&inIntervals, phase).unwrap();
    let info = pe.base().as_a_dash();
    check!(reporter, info.is_some());
    // As in C++, subsequent field accesses require an engaged optional.
    let info = info.unwrap();
    check!(reporter, 4 == info.f_intervals.len());
    check!(
        reporter,
        rust_skia::scalar::scalar_mod(phase, totalIntSum) == info.f_phase
    );
    check!(reporter, inIntervals[0] == info.f_intervals[0]);
    check!(reporter, inIntervals[1] == info.f_intervals[1]);
    check!(reporter, inIntervals[2] == info.f_intervals[2]);
    check!(reporter, inIntervals[3] == info.f_intervals[3]);
    reporter.finish();
}

#[path = "m08/supplemental.rs"]
mod supplemental;
