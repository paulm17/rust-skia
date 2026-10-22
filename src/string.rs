//! `SkString`, C-formatting, split helpers, and string utilities.

use std::ffi::CString;
use std::fmt;
use std::sync::Arc;

use crate::string_utf::sk_utf8_encode;

#[derive(Clone)]
pub struct SkString {
    data: Arc<Vec<u8>>,
    size: usize,
}

impl Default for SkString {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for SkString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&String::from_utf8_lossy(self.as_bytes()))
    }
}

impl PartialEq for SkString {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}
impl Eq for SkString {}

impl std::hash::Hash for SkString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
    }
}

impl PartialEq<str> for SkString {
    fn eq(&self, other: &str) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl From<&str> for SkString {
    fn from(value: &str) -> Self {
        Self::from_str(value)
    }
}

impl From<String> for SkString {
    fn from(value: String) -> Self {
        Self::from_str(&value)
    }
}

impl SkString {
    pub fn new() -> Self {
        Self {
            data: Arc::new(vec![0]),
            size: 0,
        }
    }

    pub fn from_str(value: &str) -> Self {
        Self::from_bytes(value.as_bytes())
    }

    pub fn from_bytes(value: &[u8]) -> Self {
        let mut data = Vec::with_capacity(value.len() + 1);
        data.extend_from_slice(value);
        data.push(0);
        Self {
            data: Arc::new(data),
            size: value.len(),
        }
    }

    pub fn from_owned(mut value: Vec<u8>) -> Self {
        let size = value.len();
        value.push(0);
        Self {
            data: Arc::new(value),
            size,
        }
    }

    pub fn with_capacity(size: usize) -> Self {
        Self {
            data: Arc::new(vec![0; size + 1]),
            size,
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }
    pub fn len(&self) -> usize {
        self.size
    }
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
    pub fn bytes(&self) -> &[u8] {
        &self.data[..self.size]
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes()
    }
    pub fn as_str(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(self.as_bytes())
    }
    pub fn data(&self) -> *mut u8 {
        self.data.as_ptr() as *mut u8
    }
    pub fn c_str(&self) -> *const u8 {
        self.data.as_ptr()
    }
    pub fn is_empty_bytes(&self) -> bool {
        self.size == 0
    }

    fn detach(&mut self) -> &mut Vec<u8> {
        Arc::make_mut(&mut self.data)
    }

    pub fn set(&mut self, value: &[u8]) {
        let mut data = Vec::with_capacity(value.len() + 1);
        data.extend_from_slice(value);
        data.push(0);
        self.data = Arc::new(data);
        self.size = value.len();
    }

    pub fn set_str(&mut self, value: &str) {
        self.set(value.as_bytes());
    }

    pub fn reset(&mut self) {
        self.set(&[]);
    }

    pub fn resize(&mut self, size: usize) {
        let old_size = self.size;
        let vec = self.detach();
        if size <= old_size {
            vec.truncate(size);
            vec.push(0);
        } else {
            vec.resize(size + 1, 0);
        }
        self.size = size;
    }

    pub fn append(&mut self, value: &[u8]) {
        if value.is_empty() {
            return;
        }
        let old_size = self.size;
        let vec = self.detach();
        vec.truncate(old_size);
        vec.extend_from_slice(value);
        vec.push(0);
        self.size = old_size + value.len();
    }

    pub fn append_str(&mut self, value: &str) {
        self.append(value.as_bytes());
    }

    pub fn append_unichar(&mut self, value: u32) {
        let mut buf = [0u8; 4];
        let len = sk_utf8_encode(value, &mut buf);
        self.append(&buf[..len]);
    }

    pub fn prepend(&mut self, value: &[u8]) {
        if value.is_empty() {
            return;
        }
        let mut data = Vec::with_capacity(self.size + value.len() + 1);
        data.extend_from_slice(value);
        data.extend_from_slice(self.bytes());
        data.push(0);
        self.size += value.len();
        self.data = Arc::new(data);
    }

    pub fn prepend_str(&mut self, value: &str) {
        self.prepend(value.as_bytes());
    }

    pub fn insert(&mut self, pos: usize, value: &[u8]) {
        let pos = pos.min(self.size);
        let old_size = self.size;
        let vec = self.detach();
        vec.truncate(old_size);
        vec.splice(pos..pos, value.iter().copied());
        vec.push(0);
        self.size += value.len();
    }

    pub fn remove(&mut self, pos: usize, len: usize) {
        if pos >= self.size {
            return;
        }
        let end = (pos + len).min(self.size);
        let old_size = self.size;
        let vec = self.detach();
        vec.truncate(old_size);
        vec.drain(pos..end);
        vec.push(0);
        self.size -= end - pos;
    }

    pub fn equals(&self, other: &str) -> bool {
        self.as_bytes() == other.as_bytes()
    }
    pub fn equals_bytes(&self, other: &[u8]) -> bool {
        self.as_bytes() == other
    }
    pub fn starts_with(&self, other: &str) -> bool {
        self.as_bytes().starts_with(other.as_bytes())
    }
    pub fn ends_with(&self, other: &str) -> bool {
        self.as_bytes().ends_with(other.as_bytes())
    }
    pub fn contains(&self, other: &str) -> bool {
        if other.is_empty() {
            return true;
        }
        self.as_bytes()
            .windows(other.len())
            .any(|w| w == other.as_bytes())
    }
    pub fn contains_byte(&self, byte: u8) -> bool {
        self.as_bytes().contains(&byte)
    }
    pub fn find(&self, needle: &str) -> Option<usize> {
        if needle.is_empty() {
            return Some(0);
        }
        self.as_bytes()
            .windows(needle.len())
            .position(|w| w == needle.as_bytes())
    }
    pub fn find_byte(&self, byte: u8) -> Option<usize> {
        self.as_bytes().iter().position(|&b| b == byte)
    }

    pub fn append_s32(&mut self, value: i32) {
        self.append_str(&value.to_string());
    }
    pub fn append_u32(&mut self, value: u32) {
        self.append_str(&value.to_string());
    }
    pub fn append_s64(&mut self, value: i64, min_digits: usize) {
        let sign = if value < 0 { "-" } else { "" };
        let digits = value.unsigned_abs().to_string();
        let pad = min_digits.saturating_sub(digits.len());
        self.append_str(sign);
        for _ in 0..pad {
            self.append_str("0");
        }
        self.append_str(&digits);
    }
    pub fn append_u64(&mut self, value: u64, min_digits: usize) {
        let digits = value.to_string();
        let pad = min_digits.saturating_sub(digits.len());
        for _ in 0..pad {
            self.append_str("0");
        }
        self.append_str(&digits);
    }
    pub fn append_scalar(&mut self, value: f32) {
        if value.is_nan() {
            self.append_str("nan");
        } else if value.is_infinite() {
            self.append_str(if value > 0.0 { "inf" } else { "-inf" });
        } else {
            let mut tmp = SkString::new();
            tmp.printf("%.8g", &[SkFormatArg::F64(value as f64)]);
            self.append(tmp.as_bytes());
        }
    }

    pub fn printf(&mut self, fmt: &str, args: &[SkFormatArg]) {
        let out = sk_format(fmt, args);
        self.set(&out);
    }

    pub fn printf0(&mut self, fmt: &str) {
        self.printf(fmt, &[]);
    }

    pub fn appendf(&mut self, fmt: &str, args: &[SkFormatArg]) {
        self.append(&sk_format(fmt, args));
    }

    pub fn prependf(&mut self, fmt: &str, args: &[SkFormatArg]) {
        self.prepend(&sk_format(fmt, args));
    }

    pub fn print_valist(&mut self, fmt: &str, args: &[SkFormatArg]) {
        self.printf(fmt, args);
    }
    pub fn append_valist(&mut self, fmt: &str, args: &[SkFormatArg]) {
        self.appendf(fmt, args);
    }
    pub fn prepend_valist(&mut self, fmt: &str, args: &[SkFormatArg]) {
        self.prependf(fmt, args);
    }
}

impl fmt::Display for SkString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&String::from_utf8_lossy(self.as_bytes()))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkScalarAsStringType {
    Dec,
    Hex,
}

pub fn sk_append_scalar(str: &mut SkString, value: f32, as_type: SkScalarAsStringType) {
    match as_type {
        SkScalarAsStringType::Hex => {
            str.appendf("SkBits2Float(0x%08x)", &[SkFormatArg::U32(value.to_bits())]);
        }
        SkScalarAsStringType::Dec => {
            let mut tmp = SkString::new();
            tmp.printf("%.9g", &[SkFormatArg::F64(value as f64)]);
            if tmp.contains_byte(b'.') {
                tmp.append_str("f");
            }
            str.append(tmp.as_bytes());
        }
    }
}

#[derive(Clone, Debug)]
pub enum SkFormatArg {
    Str(String),
    Bytes(Vec<u8>),
    S32(i32),
    U32(u32),
    S64(i64),
    U64(u64),
    Scalar(f32),
    F64(f64),
    Char(char),
    Bool(bool),
    Ptr(usize),
}

impl From<&str> for SkFormatArg {
    fn from(value: &str) -> Self {
        Self::Str(value.to_string())
    }
}
impl From<String> for SkFormatArg {
    fn from(value: String) -> Self {
        Self::Str(value)
    }
}
impl From<i32> for SkFormatArg {
    fn from(value: i32) -> Self {
        Self::S32(value)
    }
}
impl From<u32> for SkFormatArg {
    fn from(value: u32) -> Self {
        Self::U32(value)
    }
}
impl From<i64> for SkFormatArg {
    fn from(value: i64) -> Self {
        Self::S64(value)
    }
}
impl From<u64> for SkFormatArg {
    fn from(value: u64) -> Self {
        Self::U64(value)
    }
}
impl From<f32> for SkFormatArg {
    fn from(value: f32) -> Self {
        Self::Scalar(value)
    }
}
impl From<f64> for SkFormatArg {
    fn from(value: f64) -> Self {
        Self::F64(value)
    }
}

fn c_format(fmt: &str, arg: Option<&SkFormatArg>) -> Vec<u8> {
    let c_fmt = match CString::new(fmt) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    let mut buf = vec![0u8; 4096];
    let required = unsafe {
        let ptr = buf.as_mut_ptr() as *mut libc::c_char;
        match arg {
            None => libc::snprintf(ptr, buf.len(), c_fmt.as_ptr()),
            Some(SkFormatArg::Str(s)) => {
                let cs = CString::new(s.as_bytes()).unwrap_or_default();
                libc::snprintf(ptr, buf.len(), c_fmt.as_ptr(), cs.as_ptr())
            }
            Some(SkFormatArg::Bytes(b)) => {
                let cs = CString::new(b.as_slice()).unwrap_or_default();
                libc::snprintf(ptr, buf.len(), c_fmt.as_ptr(), cs.as_ptr())
            }
            Some(SkFormatArg::S32(v)) => {
                libc::snprintf(ptr, buf.len(), c_fmt.as_ptr(), *v as libc::c_int)
            }
            Some(SkFormatArg::U32(v)) => {
                libc::snprintf(ptr, buf.len(), c_fmt.as_ptr(), *v as libc::c_uint)
            }
            Some(SkFormatArg::S64(v)) => {
                libc::snprintf(ptr, buf.len(), c_fmt.as_ptr(), *v as libc::c_longlong)
            }
            Some(SkFormatArg::U64(v)) => {
                libc::snprintf(ptr, buf.len(), c_fmt.as_ptr(), *v as libc::c_ulonglong)
            }
            Some(SkFormatArg::Scalar(v)) => {
                libc::snprintf(ptr, buf.len(), c_fmt.as_ptr(), *v as libc::c_double)
            }
            Some(SkFormatArg::F64(v)) => {
                libc::snprintf(ptr, buf.len(), c_fmt.as_ptr(), *v as libc::c_double)
            }
            Some(SkFormatArg::Char(v)) => {
                libc::snprintf(ptr, buf.len(), c_fmt.as_ptr(), *v as libc::c_int)
            }
            Some(SkFormatArg::Bool(v)) => {
                libc::snprintf(ptr, buf.len(), c_fmt.as_ptr(), *v as libc::c_int)
            }
            Some(SkFormatArg::Ptr(v)) => {
                libc::snprintf(ptr, buf.len(), c_fmt.as_ptr(), *v as *const libc::c_void)
            }
        }
    };
    if required < 0 {
        return Vec::new();
    }
    let required = required as usize;
    if required < buf.len() {
        buf.truncate(required);
        return buf;
    }
    let mut large = vec![0u8; required + 1];
    let _ = unsafe {
        let ptr = large.as_mut_ptr() as *mut libc::c_char;
        match arg {
            None => libc::snprintf(ptr, large.len(), c_fmt.as_ptr()),
            Some(SkFormatArg::Str(s)) => {
                let cs = CString::new(s.as_bytes()).unwrap_or_default();
                libc::snprintf(ptr, large.len(), c_fmt.as_ptr(), cs.as_ptr())
            }
            Some(SkFormatArg::Bytes(b)) => {
                let cs = CString::new(b.as_slice()).unwrap_or_default();
                libc::snprintf(ptr, large.len(), c_fmt.as_ptr(), cs.as_ptr())
            }
            Some(SkFormatArg::S32(v)) => {
                libc::snprintf(ptr, large.len(), c_fmt.as_ptr(), *v as libc::c_int)
            }
            Some(SkFormatArg::U32(v)) => {
                libc::snprintf(ptr, large.len(), c_fmt.as_ptr(), *v as libc::c_uint)
            }
            Some(SkFormatArg::S64(v)) => {
                libc::snprintf(ptr, large.len(), c_fmt.as_ptr(), *v as libc::c_longlong)
            }
            Some(SkFormatArg::U64(v)) => {
                libc::snprintf(ptr, large.len(), c_fmt.as_ptr(), *v as libc::c_ulonglong)
            }
            Some(SkFormatArg::Scalar(v)) => {
                libc::snprintf(ptr, large.len(), c_fmt.as_ptr(), *v as libc::c_double)
            }
            Some(SkFormatArg::F64(v)) => {
                libc::snprintf(ptr, large.len(), c_fmt.as_ptr(), *v as libc::c_double)
            }
            Some(SkFormatArg::Char(v)) => {
                libc::snprintf(ptr, large.len(), c_fmt.as_ptr(), *v as libc::c_int)
            }
            Some(SkFormatArg::Bool(v)) => {
                libc::snprintf(ptr, large.len(), c_fmt.as_ptr(), *v as libc::c_int)
            }
            Some(SkFormatArg::Ptr(v)) => {
                libc::snprintf(ptr, large.len(), c_fmt.as_ptr(), *v as *const libc::c_void)
            }
        }
    };
    large.truncate(required);
    large
}

pub fn sk_format(fmt: &str, args: &[SkFormatArg]) -> Vec<u8> {
    let bytes = fmt.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    let mut arg_index = 0;
    while i < bytes.len() {
        if bytes[i] != b'%' {
            out.push(bytes[i]);
            i += 1;
            continue;
        }
        if i + 1 < bytes.len() && bytes[i + 1] == b'%' {
            out.push(b'%');
            i += 2;
            continue;
        }
        let start = i;
        i += 1;
        while i < bytes.len() && b"-+ #0".contains(&bytes[i]) {
            i += 1;
        }
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'.' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
        }
        while i < bytes.len() && b"hljztL".contains(&bytes[i]) {
            i += 1;
        }
        if i >= bytes.len() {
            out.extend_from_slice(&bytes[start..]);
            break;
        }
        let conv = bytes[i] as char;
        i += 1;
        if conv == '%' {
            out.push(b'%');
            continue;
        }
        let spec = &fmt[start..i];
        let arg = args.get(arg_index);
        if arg.is_some() {
            arg_index += 1;
        }
        out.extend_from_slice(&c_format(spec, arg));
    }
    out
}

#[allow(non_snake_case)]
pub fn SkStringPrintf(fmt: &str, args: &[SkFormatArg]) -> SkString {
    SkString::from_owned(sk_format(fmt, args))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkStrSplitMode {
    Coalesce,
    Strict,
}

pub fn sk_str_split(str: &str, delimiters: &str, mode: SkStrSplitMode) -> Vec<SkString> {
    let mut out = Vec::new();
    let delim: Vec<char> = delimiters.chars().collect();
    if mode == SkStrSplitMode::Coalesce {
        let trimmed = str.trim_start_matches(|c| delim.contains(&c));
        if trimmed.is_empty() {
            return out;
        }
        for token in trimmed.split(|c| delim.contains(&c)) {
            if !token.is_empty() {
                out.push(SkString::from_str(token));
            }
        }
    } else {
        if str.is_empty() {
            return out;
        }
        for token in str.split(|c| delim.contains(&c)) {
            out.push(SkString::from_str(token));
        }
    }
    out
}

pub fn sk_str_split_const(str: &str, delimiters: &str, mode: SkStrSplitMode) -> Vec<SkString> {
    sk_str_split(str, delimiters, mode)
}

pub fn sk_tab_string(string: &SkString, tab_cnt: i32) -> SkString {
    if tab_cnt <= 0 {
        return string.clone();
    }
    let tabs = "\t".repeat(tab_cnt as usize);
    let mut result = SkString::new();
    let bytes = string.as_bytes();
    let mut start = 0;
    while start < bytes.len() {
        let end = bytes[start..]
            .iter()
            .position(|&b| b == b'\n')
            .map(|p| start + p + 1)
            .unwrap_or(bytes.len());
        let line = &bytes[start..end];
        if !line.is_empty() && line != b"\n" {
            result.append_str(&tabs);
        }
        result.append(line);
        start = end;
    }
    result
}

pub fn sk_string_from_utf16(src: &[u16]) -> SkString {
    let mut out = String::new();
    let mut i = 0;
    while i < src.len() {
        let (cp, len) = sk_utf16_decode(src, i);
        if cp < 0 {
            break;
        }
        let mut buf = [0u8; 4];
        let n = sk_utf8_encode(cp as u32, &mut buf);
        out.push_str(std::str::from_utf8(&buf[..n]).unwrap_or(""));
        i += len;
    }
    SkString::from_str(&out)
}

fn sk_utf16_decode(src: &[u16], i: usize) -> (i32, usize) {
    let cu = src[i] as u32;
    if (0xD800..0xDC00).contains(&cu) {
        if i + 1 < src.len() {
            let cu2 = src[i + 1] as u32;
            if (0xDC00..0xE000).contains(&cu2) {
                let cp = 0x10000 + (((cu - 0xD800) << 10) | (cu2 - 0xDC00));
                return (cp as i32, 2);
            }
        }
        (-1, 1)
    } else if (0xDC00..0xE000).contains(&cu) {
        (-1, 1)
    } else {
        (cu as i32, 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_string_format_and_split() {
        let mut s = SkString::new();
        s.printf("%d %.9g %g", &[0.into(), 3.14f32.into(), 3.14f32.into()]);
        assert_eq!(s.as_bytes(), b"0 3.1400001 3.14");
        let mut t = SkString::new();
        t.printf("%08x", &[0x1234u32.into()]);
        assert_eq!(t.as_bytes(), b"00001234");
        let parts = sk_str_split("a,,b,", ",", SkStrSplitMode::Strict);
        assert_eq!(parts.len(), 4);
        let parts = sk_str_split("a,,b,", ",", SkStrSplitMode::Coalesce);
        assert_eq!(parts.len(), 2);
    }

    #[test]
    fn dev_string_scalar_and_resize() {
        let mut s = SkString::from_str("abc");
        s.resize(6);
        assert_eq!(&s.as_bytes()[..3], b"abc");
        s.resize(2);
        assert_eq!(s.as_bytes(), b"ab");
        let mut f = SkString::new();
        f.append_scalar(0.5);
        assert_eq!(f.as_bytes(), b"0.5");
        f.reset();
        f.append_scalar(f32::MAX);
        assert_eq!(f.as_bytes(), b"3.4028235e+38");
        f.reset();
        f.append_scalar(f32::NEG_INFINITY);
        assert_eq!(f.as_bytes(), b"-inf");
    }
}
