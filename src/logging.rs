//! Logging, tracing, and memory dump interfaces.

use std::sync::{Arc, Mutex, OnceLock};

use crate::string::{SkFormatArg, sk_format};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SkLogPriority {
    Error = 0,
    Warning = 1,
    Info = 2,
    Debug = 3,
}

pub trait SkLogHandler: Send + Sync {
    fn on_log(&self, priority: SkLogPriority, message: &str);
}

static HANDLER: OnceLock<Mutex<Option<Arc<dyn SkLogHandler>>>> = OnceLock::new();

fn handler_slot() -> &'static Mutex<Option<Arc<dyn SkLogHandler>>> {
    HANDLER.get_or_init(|| Mutex::new(None))
}

pub fn sk_set_log_handler(handler: Option<Arc<dyn SkLogHandler>>) {
    *handler_slot().lock().unwrap() = handler;
}

pub fn sk_log(priority: SkLogPriority, fmt: &str, args: &[SkFormatArg]) {
    let bytes = sk_format(fmt, args);
    let message = String::from_utf8_lossy(&bytes);
    let handler = handler_slot().lock().unwrap().clone();
    if let Some(handler) = handler {
        handler.on_log(priority, &message);
    } else {
        let prefix = match priority {
            SkLogPriority::Error => "E",
            SkLogPriority::Warning => "W",
            SkLogPriority::Info => "I",
            SkLogPriority::Debug => "D",
        };
        eprintln!("[{prefix}] {message}");
    }
}

pub fn sk_debugf(fmt: &str, args: &[SkFormatArg]) {
    eprint!("{}", String::from_utf8_lossy(&sk_format(fmt, args)));
}

pub fn sk_abort() {
    std::process::abort();
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkTraceEventPhase {
    Begin,
    End,
    Complete,
    Instant,
}

pub const TRACE_EVENT_PHASE_BEGIN: u8 = b'B';
pub const TRACE_EVENT_PHASE_END: u8 = b'E';
pub const TRACE_EVENT_PHASE_COMPLETE: u8 = b'X';
pub const TRACE_EVENT_PHASE_INSTANT: u8 = b'I';
pub const TRACE_EVENT_PHASE_ASYNC_BEGIN: u8 = b'S';
pub const TRACE_EVENT_PHASE_ASYNC_END: u8 = b'F';

pub trait SkEventTracer: Send + Sync {
    fn get_category_enabled(&self, category_group: u8) -> bool {
        let _ = category_group;
        false
    }
    fn get_name(&self) -> &str {
        "stub"
    }
    fn begin_event(&self, category_group: u8, name: &str) -> u64 {
        let _ = (category_group, name);
        0
    }
    fn end_event(&self, handle: u64) {
        let _ = handle;
    }
    fn on_exit(&self) {}
}

pub struct StubEventTracer;
impl SkEventTracer for StubEventTracer {}

static TRACER: OnceLock<Arc<dyn SkEventTracer>> = OnceLock::new();

pub fn sk_event_tracer() -> &'static Arc<dyn SkEventTracer> {
    TRACER.get_or_init(|| Arc::new(StubEventTracer))
}

pub fn sk_set_event_tracer(tracer: Arc<dyn SkEventTracer>) {
    let _ = TRACER.set(tracer);
}

pub trait SkTraceMemoryDump {
    fn dump_numeric_value(&mut self, dump_name: &str, value_name: &str, units: &str, value: u64);
    fn dump_string_value(&mut self, dump_name: &str, value_name: &str, value: &str) {
        let _ = (dump_name, value_name, value);
    }
    fn set_memory_backing(&mut self, dump_name: &str, backer: &str) {
        let _ = (dump_name, backer);
    }
    fn set_discardable_memory_backing(&mut self, dump_name: &str, backer: &str) {
        self.set_memory_backing(dump_name, backer);
    }
    fn get_requested_details(&self) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkTraceMemoryDumpLevelOfDetail {
    Light,
    ObjectsBreakdowns,
}
