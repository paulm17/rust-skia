//! `SkTime` helpers.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub fn sk_time_get_time_ns() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as i64
}

pub fn sk_time_get_time_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

pub fn sk_time_get_perf_counter() -> f64 {
    static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    let start = START.get_or_init(Instant::now);
    start.elapsed().as_secs_f64()
}

pub fn sk_time_get_perf_counter_ns() -> i64 {
    (sk_time_get_perf_counter() * 1_000_000_000.0) as i64
}

pub fn sk_time_sleep_ns(ns: i64) {
    if ns > 0 {
        std::thread::sleep(Duration::from_nanos(ns as u64));
    }
}

pub struct SkTime;

impl SkTime {
    pub fn get_time_ns() -> i64 {
        sk_time_get_time_ns()
    }
    pub fn get_time_ms() -> i64 {
        sk_time_get_time_ms()
    }
    pub fn get_perf_counter() -> f64 {
        sk_time_get_perf_counter()
    }
    pub fn get_perf_counter_ns() -> i64 {
        sk_time_get_perf_counter_ns()
    }
    pub fn sleep_ns(ns: i64) {
        sk_time_sleep_ns(ns);
    }
}
