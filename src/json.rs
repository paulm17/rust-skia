//! `SkJSONWriter` fast/pretty JSON writer.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkJSONWriterMode {
    Fast,
    Pretty,
}

pub struct SkJSONWriter {
    out: String,
    mode: SkJSONWriterMode,
    stack: Vec<Scope>,
    first: Vec<bool>,
    after_name: bool,
    indent: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Scope {
    Object,
    Array,
}

impl SkJSONWriter {
    pub fn new(mode: SkJSONWriterMode) -> Self {
        Self {
            out: String::new(),
            mode,
            stack: Vec::new(),
            first: Vec::new(),
            after_name: false,
            indent: 0,
        }
    }
    pub fn finish(self) -> String {
        self.out
    }
    pub fn as_str(&self) -> &str {
        &self.out
    }
    fn write_separator(&mut self) {
        if self.after_name {
            self.after_name = false;
            return;
        }
        if let Some(scope) = self.stack.last() {
            let first = self.first.last_mut().unwrap();
            if !*first {
                self.out.push(',');
            }
            *first = false;
            if *scope == Scope::Object {
                if self.mode == SkJSONWriterMode::Pretty {
                    self.out.push(' ');
                }
            }
        }
        if self.mode == SkJSONWriterMode::Pretty && !self.out.is_empty() {
            self.out.push('\n');
            for _ in 0..self.indent {
                self.out.push_str("   ");
            }
        }
    }
    pub fn begin_object(&mut self) {
        self.write_separator();
        self.out.push('{');
        self.stack.push(Scope::Object);
        self.first.push(true);
        self.indent += 1;
        self.after_name = false;
    }
    pub fn end_object(&mut self) {
        self.stack.pop();
        self.first.pop();
        self.indent = self.indent.saturating_sub(1);
        if self.mode == SkJSONWriterMode::Pretty {
            self.out.push('\n');
            for _ in 0..self.indent {
                self.out.push_str("   ");
            }
        }
        self.out.push('}');
    }
    pub fn begin_array(&mut self) {
        self.write_separator();
        self.out.push('[');
        self.stack.push(Scope::Array);
        self.first.push(true);
        self.indent += 1;
        self.after_name = false;
    }
    pub fn end_array(&mut self) {
        self.stack.pop();
        self.first.pop();
        self.indent = self.indent.saturating_sub(1);
        if self.mode == SkJSONWriterMode::Pretty {
            self.out.push('\n');
            for _ in 0..self.indent {
                self.out.push_str("   ");
            }
        }
        self.out.push(']');
    }
    pub fn append_name(&mut self, name: &str) {
        self.write_separator();
        self.write_escaped(name);
        self.out.push(':');
        self.after_name = true;
    }
    pub fn append_string(&mut self, value: &str) {
        self.write_separator();
        self.write_escaped(value);
    }
    fn write_escaped(&mut self, value: &str) {
        self.out.push('"');
        for c in value.chars() {
            match c {
                '"' => self.out.push_str("\\\""),
                '\\' => self.out.push_str("\\\\"),
                '\n' => self.out.push_str("\\n"),
                '\r' => self.out.push_str("\\r"),
                '\t' => self.out.push_str("\\t"),
                '\u{08}' => self.out.push_str("\\b"),
                '\u{0c}' => self.out.push_str("\\f"),
                c if (c as u32) < 0x20 => {
                    self.out.push_str(&format!("\\u{:04x}", c as u32));
                }
                c => self.out.push(c),
            }
        }
        self.out.push('"');
    }
    pub fn append_int(&mut self, value: i64) {
        self.write_separator();
        self.out.push_str(&value.to_string());
    }
    pub fn append_uint(&mut self, value: u64) {
        self.write_separator();
        self.out.push_str(&value.to_string());
    }
    pub fn append_float(&mut self, value: f64) {
        self.write_separator();
        self.out.push_str(&format!("{value}"));
    }
    pub fn append_bool(&mut self, value: bool) {
        self.write_separator();
        self.out.push_str(if value { "true" } else { "false" });
    }
    pub fn append_null(&mut self) {
        self.write_separator();
        self.out.push_str("null");
    }
    pub fn append_hex(&mut self, value: u32) {
        self.append_string(&format!("0x{value:x}"));
    }
    pub fn append_f(&mut self, fmt: &str, args: &[crate::string::SkFormatArg]) {
        let s = crate::string::sk_format(fmt, args);
        self.append_string(&String::from_utf8_lossy(&s));
    }
}
