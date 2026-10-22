// Copyright 2016 Google Inc. BSD: tests/LICENSE.skia.
// Tests/FakeStreams.h: owned SkData and delegated memory-stream operations.
use super::*;
pub struct NotAssetMemStream {
    stream: SkMemoryStream,
}
impl NotAssetMemStream {
    pub fn new(d: SkData) -> Self {
        Self {
            stream: SkMemoryStream::new(d),
        }
    }
}
impl SkStream for NotAssetMemStream {
    fn read(&mut self, b: &mut [u8]) -> usize {
        self.stream.read(b)
    }
    fn peek(&mut self, b: &mut [u8]) -> usize {
        self.stream.peek(b)
    }
    fn rewind(&mut self) -> bool {
        self.stream.rewind()
    }
    fn is_at_end(&mut self) -> bool {
        self.stream.is_at_end()
    }
    fn has_position(&self) -> bool {
        false
    }
    fn has_length(&self) -> bool {
        false
    }
    fn position(&self) -> usize {
        0
    }
    fn seek(&mut self, _: usize) -> bool {
        false
    }
}
pub struct NonseekableStream {
    stream: NotAssetMemStream,
}
impl NonseekableStream {
    pub fn new(d: SkData) -> Self {
        Self {
            stream: NotAssetMemStream::new(d),
        }
    }
}
impl SkStream for NonseekableStream {
    fn read(&mut self, b: &mut [u8]) -> usize {
        self.stream.read(b)
    }
    fn peek(&mut self, b: &mut [u8]) -> usize {
        self.stream.peek(b)
    }
    fn rewind(&mut self) -> bool {
        false
    }
    fn is_at_end(&mut self) -> bool {
        self.stream.is_at_end()
    }
    fn has_position(&self) -> bool {
        false
    }
    fn has_length(&self) -> bool {
        false
    }
    fn position(&self) -> usize {
        0
    }
    fn seek(&mut self, _: usize) -> bool {
        false
    }
}
pub struct HaltingStream {
    total: usize,
    limit: usize,
    stream: SkMemoryStream,
}
impl HaltingStream {
    pub fn new(d: SkData, initial: usize) -> Self {
        Self {
            total: d.size(),
            limit: initial,
            stream: SkMemoryStream::new(d),
        }
    }
    pub fn add_new_data(&mut self, extra: usize) {
        self.limit = self.total.min(self.limit + extra);
    }
    pub fn is_all_data_received(&self) -> bool {
        self.limit == self.total
    }
}
impl SkStream for HaltingStream {
    fn read(&mut self, b: &mut [u8]) -> usize {
        let size = if self.stream.position() + b.len() > self.limit {
            self.limit - self.stream.position()
        } else {
            b.len()
        };
        self.stream.read(&mut b[..size])
    }
    fn is_at_end(&mut self) -> bool {
        self.stream.is_at_end()
    }
    fn has_length(&self) -> bool {
        true
    }
    fn length(&self) -> usize {
        self.limit
    }
    fn has_position(&self) -> bool {
        true
    }
    fn position(&self) -> usize {
        self.stream.position()
    }
    fn rewind(&mut self) -> bool {
        self.stream.rewind()
    }
    fn move_(&mut self, offset: i32) -> bool {
        self.stream.move_(offset)
    }
    fn seek(&mut self, pos: usize) -> bool {
        self.stream.seek(pos)
    }
}
