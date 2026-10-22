//! Streams and byte readers/writers.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::data::SkData;

pub trait SkStream {
    fn read(&mut self, buffer: &mut [u8]) -> usize;
    fn skip(&mut self, size: usize) -> usize {
        let mut remaining = size;
        let mut scratch = [0u8; 1024];
        while remaining > 0 {
            let n = remaining.min(scratch.len());
            let read = self.read(&mut scratch[..n]);
            remaining -= read;
            if read < n {
                break;
            }
        }
        size - remaining
    }
    fn peek(&mut self, _buffer: &mut [u8]) -> usize {
        0
    }
    fn is_at_end(&mut self) -> bool;
    fn has_position(&self) -> bool {
        true
    }
    fn position(&self) -> usize;
    fn rewind(&mut self) -> bool {
        self.seek(0)
    }
    fn seek(&mut self, position: usize) -> bool;
    fn move_(&mut self, offset: i32) -> bool {
        let pos = self.position() as i64 + offset as i64;
        if pos < 0 {
            false
        } else {
            self.seek(pos as usize)
        }
    }
    fn has_length(&self) -> bool {
        false
    }
    fn length(&self) -> usize {
        0
    }
    fn get_memory_base(&self) -> Option<&[u8]> {
        None
    }
    fn read_bool(&mut self) -> bool {
        let mut b = [0u8; 1];
        self.read(&mut b) == 1 && b[0] != 0
    }
    fn read_u8(&mut self) -> u8 {
        let mut b = [0u8; 1];
        if self.read(&mut b) == 1 { b[0] } else { 0 }
    }
    fn read_s8(&mut self) -> i8 {
        self.read_u8() as i8
    }
    fn read_u16(&mut self) -> u16 {
        let mut b = [0u8; 2];
        if self.read(&mut b) == 2 {
            u16::from_le_bytes(b)
        } else {
            0
        }
    }
    fn read_s16(&mut self) -> i16 {
        self.read_u16() as i16
    }
    fn read_u32(&mut self) -> u32 {
        let mut b = [0u8; 4];
        if self.read(&mut b) == 4 {
            u32::from_le_bytes(b)
        } else {
            0
        }
    }
    fn read_s32(&mut self) -> i32 {
        self.read_u32() as i32
    }
    fn read_u64(&mut self) -> u64 {
        let mut b = [0u8; 8];
        if self.read(&mut b) == 8 {
            u64::from_le_bytes(b)
        } else {
            0
        }
    }
    fn read_s64(&mut self) -> i64 {
        self.read_u64() as i64
    }
    fn read_scalar(&mut self) -> f32 {
        f32::from_bits(self.read_u32())
    }
    fn read_packed_u32(&mut self) -> Option<u32> {
        fn exact<S: SkStream + ?Sized>(stream: &mut S, buf: &mut [u8]) -> bool {
            stream.read(buf) == buf.len()
        }
        let mut first = [0u8; 1];
        if !exact(self, &mut first) {
            return None;
        }
        match first[0] {
            0xFE => {
                let mut b = [0u8; 2];
                if !exact(self, &mut b) {
                    None
                } else {
                    Some(u16::from_le_bytes(b) as u32)
                }
            }
            0xFF => {
                let mut b = [0u8; 4];
                if !exact(self, &mut b) {
                    None
                } else {
                    Some(u32::from_le_bytes(b))
                }
            }
            v => Some(v as u32),
        }
    }
}

pub trait SkWStream {
    fn write(&mut self, buffer: &[u8]) -> bool;
    fn flush(&mut self) {}
    fn bytes_written(&self) -> usize;
    fn write_u8(&mut self, value: u8) -> bool {
        self.write(&[value])
    }
    fn write_s8(&mut self, value: i8) -> bool {
        self.write_u8(value as u8)
    }
    fn write_u16(&mut self, value: u16) -> bool {
        self.write(&value.to_le_bytes())
    }
    fn write_s16(&mut self, value: i16) -> bool {
        self.write_u16(value as u16)
    }
    fn write_u32(&mut self, value: u32) -> bool {
        self.write(&value.to_le_bytes())
    }
    fn write_s32(&mut self, value: i32) -> bool {
        self.write_u32(value as u32)
    }
    fn write_u64(&mut self, value: u64) -> bool {
        self.write(&value.to_le_bytes())
    }
    fn write_s64(&mut self, value: i64) -> bool {
        self.write_u64(value as u64)
    }
    fn write_scalar(&mut self, value: f32) -> bool {
        self.write_u32(value.to_bits())
    }
    fn write_packed_u32(&mut self, value: u32) -> bool {
        if value <= 0xFD {
            self.write_u8(value as u8)
        } else if value <= 0xFFFF {
            self.write_u8(0xFE) && self.write_u16(value as u16)
        } else {
            self.write_u8(0xFF) && self.write_u32(value)
        }
    }
    fn write_dec_as_text(&mut self, value: i32, min_chars: u32) -> bool {
        let text = format!("{value:0width$}", width = min_chars as usize);
        self.write(text.as_bytes())
    }
    fn write_hex_as_text(&mut self, value: u32, min_chars: u32) -> bool {
        let text = format!("{value:x}");
        let text = format!("{text:0>width$}", width = min_chars as usize);
        self.write(text.as_bytes())
    }
    fn write_scalar_as_text(&mut self, value: f32, precision: u32) -> bool {
        let text = format!("{value:.precision$}", precision = precision as usize);
        self.write(text.as_bytes())
    }
}

pub struct SkNullWStream {
    bytes: usize,
}

impl Default for SkNullWStream {
    fn default() -> Self {
        Self { bytes: 0 }
    }
}

impl SkWStream for SkNullWStream {
    fn write(&mut self, buffer: &[u8]) -> bool {
        self.bytes += buffer.len();
        true
    }
    fn bytes_written(&self) -> usize {
        self.bytes
    }
}

pub struct SkMemoryStream {
    data: SkData,
    offset: usize,
}

impl SkMemoryStream {
    pub fn new(data: SkData) -> Self {
        Self { data, offset: 0 }
    }
    pub fn make_uninitialized(size: usize) -> Self {
        Self::new(SkData::make_uninitialized(size))
    }
    pub fn data(&self) -> &SkData {
        &self.data
    }
    pub fn set_data(&mut self, data: Option<SkData>) {
        self.data = data.unwrap_or_else(SkData::make_empty);
        self.offset = 0;
    }
    pub fn duplicate(&self) -> Option<Self> {
        Some(Self {
            data: self.data.clone(),
            offset: 0,
        })
    }
    pub fn fork(&self) -> Option<Self> {
        Some(Self {
            data: self.data.clone(),
            offset: self.offset,
        })
    }
}

impl SkStream for SkMemoryStream {
    fn read(&mut self, buffer: &mut [u8]) -> usize {
        let bytes = self.data.bytes();
        let remaining = bytes.len().saturating_sub(self.offset);
        let n = remaining.min(buffer.len());
        buffer[..n].copy_from_slice(&bytes[self.offset..self.offset + n]);
        self.offset += n;
        n
    }
    fn peek(&mut self, buffer: &mut [u8]) -> usize {
        let bytes = self.data.bytes();
        let remaining = bytes.len().saturating_sub(self.offset);
        let n = remaining.min(buffer.len());
        buffer[..n].copy_from_slice(&bytes[self.offset..self.offset + n]);
        n
    }
    fn is_at_end(&mut self) -> bool {
        self.offset >= self.data.size()
    }
    fn position(&self) -> usize {
        self.offset
    }
    fn seek(&mut self, position: usize) -> bool {
        self.offset = position.min(self.data.size());
        true
    }
    fn has_length(&self) -> bool {
        true
    }
    fn length(&self) -> usize {
        self.data.size()
    }
    fn get_memory_base(&self) -> Option<&[u8]> {
        if self.data.is_empty() {
            None
        } else {
            Some(self.data.bytes())
        }
    }
}

pub struct SkFILEStream {
    file: File,
    start: u64,
    end: u64,
    current: u64,
}

impl SkFILEStream {
    pub fn new(path: impl AsRef<Path>) -> Option<Self> {
        let mut file = File::open(path).ok()?;
        let end = file.seek(SeekFrom::End(0)).ok()?;
        file.seek(SeekFrom::Start(0)).ok()?;
        Some(Self {
            file,
            start: 0,
            end,
            current: 0,
        })
    }

    fn from_file(mut file: File) -> Option<Self> {
        let start = file.stream_position().ok()?;
        let end = file.seek(SeekFrom::End(0)).ok()?;
        file.seek(SeekFrom::Start(start)).ok()?;
        Some(Self {
            file,
            start,
            end,
            current: start,
        })
    }

    pub fn duplicate(&self) -> Option<Self> {
        let mut file = self.file.try_clone().ok()?;
        file.seek(SeekFrom::Start(self.start)).ok()?;
        Some(Self {
            file,
            start: self.start,
            end: self.end,
            current: self.start,
        })
    }

    pub fn fork(&self) -> Option<Self> {
        let mut dup = self.duplicate()?;
        dup.current = self.current;
        dup.file.seek(SeekFrom::Start(self.current)).ok()?;
        Some(dup)
    }

    pub fn move_long(&mut self, offset: i64) -> bool {
        let base = self.current as i128;
        let target = (base + offset as i128).clamp(self.start as i128, self.end as i128);
        self.current = target as u64;
        true
    }
}

impl SkStream for SkFILEStream {
    fn read(&mut self, buffer: &mut [u8]) -> usize {
        if self.current >= self.end {
            return 0;
        }
        let remaining = (self.end - self.current) as usize;
        let n = remaining.min(buffer.len());
        if self.file.seek(SeekFrom::Start(self.current)).is_err() {
            return 0;
        }
        let read = self.file.read(&mut buffer[..n]).unwrap_or(0);
        self.current += read as u64;
        read
    }
    fn is_at_end(&mut self) -> bool {
        self.current >= self.end
    }
    fn position(&self) -> usize {
        (self.current - self.start) as usize
    }
    fn seek(&mut self, position: usize) -> bool {
        let target = (self.start + position as u64).min(self.end);
        self.current = target;
        true
    }
    fn has_length(&self) -> bool {
        true
    }
    fn length(&self) -> usize {
        (self.end - self.start) as usize
    }
}

pub struct SkFILEWStream {
    file: Option<File>,
    valid: bool,
    bytes: usize,
    path: std::path::PathBuf,
}

impl SkFILEWStream {
    pub fn new(path: impl AsRef<Path>) -> Option<Self> {
        let path = path.as_ref().to_path_buf();
        let file = File::create(&path).ok()?;
        Some(Self {
            file: Some(file),
            valid: true,
            bytes: 0,
            path,
        })
    }
    pub fn is_valid(&self) -> bool {
        self.valid
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl SkWStream for SkFILEWStream {
    fn write(&mut self, buffer: &[u8]) -> bool {
        if !self.valid {
            return false;
        }
        let Some(file) = self.file.as_mut() else {
            self.valid = false;
            return false;
        };
        match file.write_all(buffer) {
            Ok(()) => {
                self.bytes += buffer.len();
                true
            }
            Err(_) => {
                self.valid = false;
                false
            }
        }
    }
    fn flush(&mut self) {
        if let Some(file) = self.file.as_mut() {
            let _ = file.flush();
        }
    }
    fn bytes_written(&self) -> usize {
        self.bytes
    }
}

#[derive(Default)]
pub struct SkDynamicMemoryWStream {
    data: Vec<u8>,
}

impl SkDynamicMemoryWStream {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
    pub fn pad_to_align4(&mut self) {
        while self.data.len() % 4 != 0 {
            self.data.push(0);
        }
    }
    pub fn detach_as_data(&mut self) -> SkData {
        if self.data.is_empty() {
            return SkData::make_empty();
        }
        SkData::make_with_copy(&std::mem::take(&mut self.data))
    }
    pub fn detach_as_vector(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.data)
    }
    pub fn detach_as_stream(&mut self) -> SkMemoryStream {
        let data = std::mem::take(&mut self.data);
        SkMemoryStream::new(if data.is_empty() {
            SkData::make_empty()
        } else {
            SkData::make_with_copy(&data)
        })
    }
    pub fn write_to_and_reset(&mut self, dst: &mut Vec<u8>) {
        dst.extend_from_slice(&self.data);
        self.data.clear();
    }
    pub fn read(&self, offset: usize, buffer: &mut [u8]) -> bool {
        if buffer.len() > self.data.len().saturating_sub(offset) {
            return false;
        }
        buffer.copy_from_slice(&self.data[offset..offset + buffer.len()]);
        true
    }
    pub fn copy_to(&self, dst: &mut [u8]) -> bool {
        if dst.len() < self.data.len() {
            return false;
        }
        dst[..self.data.len()].copy_from_slice(&self.data);
        true
    }
    pub fn bytes(&self) -> &[u8] {
        &self.data
    }
}

impl SkWStream for SkDynamicMemoryWStream {
    fn write(&mut self, buffer: &[u8]) -> bool {
        self.data.extend_from_slice(buffer);
        true
    }
    fn bytes_written(&self) -> usize {
        self.data.len()
    }
}

pub fn remaining_length_is_below(stream: &dyn SkStream, requested: usize) -> bool {
    !stream.has_length() || stream.length().saturating_sub(stream.position()) < requested
}

pub fn sk_stream_copy(dst: &mut dyn SkWStream, src: &mut dyn SkStream) -> bool {
    let mut buffer = [0u8; 4096];
    loop {
        let read = src.read(&mut buffer);
        if read == 0 {
            return src.is_at_end();
        }
        if !dst.write(&buffer[..read]) {
            return false;
        }
    }
}

pub fn sk_copy_stream_to_data(stream: &mut dyn SkStream) -> Option<SkData> {
    if stream.has_length() {
        let len = stream.length().saturating_sub(stream.position());
        let mut bytes = vec![0u8; len];
        let mut read = 0;
        while read < len {
            let n = stream.read(&mut bytes[read..]);
            if n == 0 {
                break;
            }
            read += n;
        }
        bytes.truncate(read);
        return Some(SkData::make_with_copy(&bytes));
    }
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        let n = stream.read(&mut chunk);
        if n == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..n]);
    }
    Some(SkData::make_with_copy(&bytes))
}

pub fn sk_stream_make_from_file(path: impl AsRef<Path>) -> Option<Box<dyn SkStream>> {
    let data = SkData::make_from_file(path.as_ref())?;
    Some(Box::new(SkMemoryStream::new(data)))
}

pub fn sk_stream_file_from_stdio(file: File) -> Option<SkFILEStream> {
    SkFILEStream::from_file(file)
}

/// Packed uint size used by `SkWStream`.
pub fn sk_size_of_packed_u32(value: u32) -> usize {
    if value <= 0xFD {
        1
    } else if value <= 0xFFFF {
        3
    } else {
        5
    }
}
