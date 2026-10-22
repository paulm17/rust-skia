//! `SkWriter32`, `SkRBuffer`, and `SkWBuffer` byte serialization primitives.

use crate::data::SkData;

pub struct SkRBuffer<'a> {
    data: Option<&'a [u8]>,
    pos: usize,
    stop: Option<usize>,
    valid: bool,
}

impl<'a> SkRBuffer<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data: Some(data),
            pos: 0,
            stop: Some(data.len()),
            valid: true,
        }
    }
    pub fn new_with_size(data: &'a [u8], size: usize) -> Self {
        Self {
            data: Some(data),
            pos: 0,
            stop: Some(size.min(data.len())),
            valid: true,
        }
    }
    pub fn new_no_size(data: &'a [u8]) -> Self {
        Self {
            data: Some(data),
            pos: 0,
            stop: None,
            valid: true,
        }
    }
    pub fn is_valid(&self) -> bool {
        self.valid
    }
    pub fn eof(&self) -> bool {
        match self.stop {
            Some(stop) => self.pos >= stop,
            None => true,
        }
    }
    pub fn available(&self) -> usize {
        match self.stop {
            Some(stop) => stop.saturating_sub(self.pos),
            None => 0,
        }
    }
    pub fn read(&mut self, size: usize) -> Option<&'a [u8]> {
        if !self.valid || self.available() < size {
            self.valid = false;
            return None;
        }
        let out = &self.data.unwrap()[self.pos..self.pos + size];
        self.pos += size;
        Some(out)
    }
    pub fn skip(&mut self, size: usize) -> bool {
        if !self.valid || self.available() < size {
            self.valid = false;
            return false;
        }
        self.pos += size;
        true
    }
    pub fn skip_to_align4(&mut self) -> bool {
        let addr = self
            .data
            .map(|d| d.as_ptr() as usize + self.pos)
            .unwrap_or(self.pos);
        let aligned = (addr + 3) & !3;
        self.skip(aligned - addr)
    }
    pub fn read_u8(&mut self) -> u8 {
        self.read(1).map(|b| b[0]).unwrap_or(0)
    }
    pub fn read_u16(&mut self) -> u16 {
        self.read(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .unwrap_or(0)
    }
    pub fn read_u32(&mut self) -> u32 {
        self.read(4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .unwrap_or(0)
    }
    pub fn read_scalar(&mut self) -> f32 {
        f32::from_bits(self.read_u32())
    }
}

pub struct SkWBuffer<'a> {
    data: Option<&'a mut [u8]>,
    pos: usize,
}

impl<'a> SkWBuffer<'a> {
    pub fn new(data: &'a mut [u8]) -> Self {
        Self {
            data: Some(data),
            pos: 0,
        }
    }
    pub fn new_empty() -> Self {
        Self { data: None, pos: 0 }
    }
    pub fn pos(&self) -> usize {
        self.pos
    }
    pub fn write(&mut self, bytes: &[u8]) -> bool {
        if let Some(data) = self.data.as_mut() {
            if self.pos + bytes.len() > data.len() {
                return false;
            }
            data[self.pos..self.pos + bytes.len()].copy_from_slice(bytes);
        }
        self.pos += bytes.len();
        true
    }
    pub fn skip(&mut self, size: usize) -> usize {
        let start = self.pos;
        if let Some(data) = self.data.as_mut() {
            if start + size > data.len() {
                return start;
            }
        }
        self.pos += size;
        start
    }
    pub fn pad_to_align4(&mut self) {
        while self.pos % 4 != 0 {
            self.write(&[0]);
        }
    }
    pub fn write_u8(&mut self, value: u8) -> bool {
        self.write(&[value])
    }
    pub fn write_u16(&mut self, value: u16) -> bool {
        self.write(&value.to_le_bytes())
    }
    pub fn write_u32(&mut self, value: u32) -> bool {
        self.write(&value.to_le_bytes())
    }
}

pub struct SkWriter32 {
    data: Vec<u8>,
}

impl Default for SkWriter32 {
    fn default() -> Self {
        Self::new()
    }
}

impl SkWriter32 {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
    pub fn with_external(_data: &mut [u8]) -> Self {
        Self::new()
    }
    pub fn bytes_written(&self) -> usize {
        self.data.len()
    }
    pub fn size(&self) -> usize {
        self.data.len()
    }
    pub fn reset(&mut self) {
        self.data.clear();
    }
    pub fn reserve(&mut self, size: usize) -> usize {
        assert_eq!(size % 4, 0);
        let offset = self.data.len();
        self.data.resize(offset + size, 0);
        offset
    }
    pub fn reserve_pad(&mut self, size: usize) -> usize {
        let offset = self.reserve(size & !3);
        if size & 3 != 0 {
            self.data.resize(self.data.len() + 4, 0);
        }
        offset
    }
    pub fn write(&mut self, bytes: &[u8]) {
        assert_eq!(bytes.len() % 4, 0);
        self.data.extend_from_slice(bytes);
    }
    pub fn write_pad(&mut self, bytes: &[u8], pad: usize) {
        self.data.extend_from_slice(bytes);
        self.data.resize(self.data.len() + pad, 0);
    }
    pub fn write_u8(&mut self, value: u8) {
        self.write(&(value as u32).to_le_bytes());
    }
    pub fn write_u16(&mut self, value: u16) {
        self.write(&(value as u32).to_le_bytes());
    }
    pub fn write_u32(&mut self, value: u32) {
        self.write(&value.to_le_bytes());
    }
    pub fn write_scalar(&mut self, value: f32) {
        self.write_u32(value.to_bits());
    }
    pub fn write_string(&mut self, value: Option<&str>) {
        let bytes = value.unwrap_or("").as_bytes();
        self.write_u32(bytes.len() as u32);
        self.data.extend_from_slice(bytes);
        self.data.push(0);
        while self.data.len() % 4 != 0 {
            self.data.push(0);
        }
    }
    pub fn write_data(&mut self, data: &SkData) {
        self.write_u32(data.size() as u32);
        self.data.extend_from_slice(data.bytes());
        while self.data.len() % 4 != 0 {
            self.data.push(0);
        }
    }
    pub fn snapshot_as_data(&self) -> SkData {
        SkData::make_with_copy(&self.data)
    }
    pub fn bytes(&self) -> &[u8] {
        &self.data
    }
    pub fn rewind_to_offset(&mut self, offset: usize) {
        assert_eq!(offset % 4, 0);
        assert!(offset <= self.data.len());
        self.data.truncate(offset);
    }
    pub fn write_string_size(value: Option<&str>) -> usize {
        let len = value.unwrap_or("").len();
        (4 + len + 1 + 3) & !3
    }
    pub fn write_data_size(size: usize) -> usize {
        4 + ((size + 3) & !3)
    }
}
