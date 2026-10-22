// Copyright Google LLC and original Skia authors. BSD license: tests/LICENSE.skia.
// tests/StreamTest.cpp at b81c69985fd14bb7bfbd356b7618633ce07dc1a4.
use super::*;
use std::{
    fs::File,
    io::{Seek, SeekFrom},
    path::PathBuf,
};
const ABC: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
struct TempPath(PathBuf);
impl TempPath {
    fn new(label: &str) -> Self {
        static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        Self(std::env::temp_dir().join(format!(
            "m03-{label}-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        )))
    }
}
impl Drop for TempPath {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
fn resource_root() -> PathBuf {
    std::env::var_os("SKIA_RESOURCE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../skia/resources"))
}
fn test_loop_stream(r: &mut Reporter, stream: &mut dyn SkStream, src: &[u8], repeat: usize) {
    let mut tmp = vec![0; src.len()];
    for _ in 0..repeat {
        let bytes = stream.read(&mut tmp);
        check!(r, bytes == src.len());
        check!(r, tmp == src);
    }
    let bytes = stream.read(&mut tmp[..1]);
    check!(r, 0 == bytes);
    check!(r, stream.is_at_end());
}
fn test_filestreams(r: &mut Reporter) {
    let path = TempPath::new("wstream_test");
    {
        let Some(mut writer) = SkFILEWStream::new(&path.0) else {
            check!(r, false);
            return;
        };
        if !writer.is_valid() {
            check!(r, false);
            return;
        }
        for _ in 0..100 {
            writer.write(ABC);
        }
    }
    {
        let stream = SkFILEStream::new(&path.0);
        check!(r, stream.is_some());
        let mut stream = stream.unwrap();
        test_loop_stream(r, &mut stream, ABC, 100);
        let mut stream2 = stream.duplicate().unwrap();
        test_loop_stream(r, &mut stream2, ABC, 100);
    }
    {
        let file = File::open(&path.0).unwrap();
        let stream = sk_stream_file_from_stdio(file);
        check!(r, stream.is_some());
        let mut stream = stream.unwrap();
        test_loop_stream(r, &mut stream, ABC, 100);
        let mut stream2 = stream.duplicate().unwrap();
        test_loop_stream(r, &mut stream2, ABC, 100);
    }
}
fn TestWStream(r: &mut Reporter) {
    let mut ds = SkDynamicMemoryWStream::new();
    for _ in 0..100 {
        check!(r, ds.write(ABC));
    }
    check!(r, ds.bytes_written() == 100 * 26);
    let mut dst = vec![0u8; 100 * 26 + 1];
    dst[100 * 26] = b'*';
    ds.copy_to(&mut dst);
    check!(r, dst[100 * 26] == b'*');
    for i in 0..100 {
        check!(r, &dst[i * 26..i * 26 + 26] == ABC);
    }
    {
        let mut stream = ds.detach_as_stream();
        check!(r, 100 * 26 == stream.length());
        check!(r, ds.bytes_written() == 0);
        test_loop_stream(r, &mut stream, ABC, 100);
        let mut stream2 = stream.duplicate().unwrap();
        test_loop_stream(r, &mut stream2, ABC, 100);
        let mut stream3 = stream.fork().unwrap();
        check!(r, stream3.is_at_end());
        let mut tmp = [0];
        let bytes = stream.read(&mut tmp);
        check!(r, 0 == bytes);
        stream3.rewind();
        test_loop_stream(r, &mut stream3, ABC, 100);
    }
    for _ in 0..100 {
        check!(r, ds.write(ABC));
    }
    check!(r, ds.bytes_written() == 100 * 26);
    {
        let mut stream = ds.detach_as_stream();
        check!(r, ds.bytes_written() == 0);
        test_loop_stream(r, &mut stream, ABC, 100);
        let mut stream2 = stream.duplicate().unwrap();
        test_loop_stream(r, &mut stream2, ABC, 100);
    }
    drop(dst);
    // GetTmpDir() maps to Rust's configured temporary directory, available on this macOS gate.
    if !std::env::temp_dir().as_os_str().is_empty() {
        test_filestreams(r);
    }
}
fn TestPackedUInt(r: &mut Reporter) {
    let sizes = [
        0, 1, 2, 0xfc, 0xfd, 0xfe, 0xff, 0x100, 0x101, 32767, 32768, 32769, 0xfffd, 0xfffe, 0xffff,
        0x10000, 0x10001, 0xfffffd, 0xfffffe, 0xffffff, 0x1000000, 0x1000001, 0x7ffffffe,
        0x7fffffff, 0x80000000, 0x80000001, 0xfffffffe, 0xffffffff,
    ];
    let mut wstream = SkDynamicMemoryWStream::new();
    for size in sizes {
        let success = wstream.write_packed_u32(size);
        check!(r, success);
    }
    let mut rstream = wstream.detach_as_stream();
    for size in sizes {
        let n = rstream.read_packed_u32();
        if n.is_none() {
            check!(r, false);
        }
        if Some(size) != n {
            check!(r, false);
        }
    }
}
fn TestDereferencingData(r: &mut Reporter, s: &mut SkMemoryStream) {
    check!(r, s.read(&mut []) == 0);
    let mut buf = [0];
    check!(r, s.read(&mut buf) == 0);
    let _ = s.get_memory_base();
    let _ = s.data();
}
fn TestNullData(r: &mut Reporter) {
    // The non-null Rust value representing C++ nullptr data is SkData::make_empty().
    let mut s = SkMemoryStream::new(SkData::make_empty());
    TestDereferencingData(r, &mut s);
    s.set_data(None);
    TestDereferencingData(r, &mut s);
}
// tests/StreamTest.cpp:207
#[test]
fn Stream() {
    let mut r = Reporter::default();
    TestWStream(&mut r);
    TestPackedUInt(&mut r);
    TestNullData(&mut r);
    r.finish();
}
fn compare_peek_to_read(r: &mut Reporter, s: &mut dyn SkStream, bytes_to_peek: usize) -> usize {
    check!(r, bytes_to_peek > 0);
    let mut peek_storage = vec![0; bytes_to_peek];
    let _read_storage = vec![0u8; bytes_to_peek];
    let bytes_peeked = s.peek(&mut peek_storage);
    let bytes_read = s.read(&mut peek_storage);
    check!(r, bytes_read == bytes_to_peek || s.is_at_end());
    // Upstream readPtr aliases peekStorage, so this comparison deliberately compares the same buffer.
    check!(
        r,
        peek_storage[..bytes_peeked] == peek_storage[..bytes_peeked]
    );
    check!(r, bytes_read >= bytes_peeked);
    bytes_read - bytes_peeked
}
fn test_fully_peekable_stream(r: &mut Reporter, s: &mut dyn SkStream, _limit: usize) {
    let mut i = 1;
    while !s.is_at_end() {
        let result = compare_peek_to_read(r, s, i);
        check!(r, result == 0);
        i += 1;
    }
}
// tests/StreamTest.cpp:310; !SK_BUILD_FOR_IOS, default CPU build has no SK_ENABLE_ANDROID_UTILS.
#[cfg(not(target_os = "ios"))]
#[test]
fn StreamPeek() {
    let mut r = Reporter::default();
    let mut mem = SkMemoryStream::new(SkData::make_without_copy(ABC));
    let length = mem.length();
    test_fully_peekable_stream(&mut r, &mut mem, length);
    if std::env::temp_dir().as_os_str().is_empty() {
        check!(r, false);
        r.finish();
        return;
    }
    let path = TempPath::new("file");
    {
        let Some(mut w) = SkFILEWStream::new(&path.0) else {
            check!(r, false);
            r.finish();
            return;
        };
        let data = std::fs::read(resource_root().join("images/baby_tux.webp"));
        let Ok(data) = data else {
            check!(r, false);
            r.finish();
            return;
        };
        if data.is_empty() {
            check!(r, false);
            r.finish();
            return;
        }
        if !w.is_valid() || !w.write(&data) {
            check!(r, false);
            r.finish();
            return;
        }
    }
    let file = SkFILEStream::new(&path.0);
    check!(r, file.is_some());
    let Some(mut file) = file else {
        r.finish();
        return;
    };
    let mut storage = vec![0; file.length()];
    for i in 1..file.length() {
        check!(r, file.peek(&mut storage[..i]) == 0);
    }
    r.finish();
}
fn stream_peek_test(r: &mut Reporter, asset: &mut dyn SkStream, expected: &SkData) {
    if asset.length() != expected.size() {
        check!(r, false);
        return;
    }
    let mut rand = SkRandom::new();
    let mut buffer = [0; 4096];
    let expect = expected.bytes();
    for i in 0..asset.length() {
        let max_size = buffer.len().min(asset.length() - i) as u32;
        let size = rand.next_range_u(1, max_size) as usize;
        debug_assert!(size >= 1);
        debug_assert!(size <= buffer.len());
        debug_assert!(size + i <= asset.length());
        if asset.peek(&mut buffer[..size]) < size {
            check!(r, false);
            return;
        }
        if buffer[..size] != expect[i..i + size] {
            check!(r, false);
            return;
        }
        let mut value = [0];
        check!(r, 1 == asset.read(&mut value));
        if value[0] != expect[i] {
            check!(r, false);
            return;
        }
    }
}
// tests/StreamTest.cpp:390
#[test]
fn StreamPeek_BlockMemoryStream() {
    let mut r = Reporter::default();
    let seed = 1234;
    let mut values = SkRandom::from_seed(seed);
    let mut rand = SkRandom::from_seed(seed << 1);
    let mut buffer = [0u8; 4096];
    let mut stream = SkDynamicMemoryWStream::new();
    let mut total = 0;
    for _ in 0..32 {
        let size = rand.next_range_u(1, buffer.len() as u32) as usize;
        for j in 0..size {
            buffer[j] = (values.next_u() & 0xff) as u8;
        }
        stream.write(&buffer[..size]);
        total += size;
        check!(r, total == stream.bytes_written());
    }
    let mut asset = stream.detach_as_stream();
    let mut expected = SkData::make_uninitialized(asset.length());
    values.set_seed(seed);
    for i in 0..asset.length() {
        unsafe {
            *expected.writable_data().add(i) = (values.next_u() & 0xff) as u8;
        }
    }
    stream_peek_test(&mut r, &mut asset, &expected);
    r.finish();
}
// tests/StreamTest.cpp:420. The helper remains a production call.
#[test]
fn RemainingLengthIsBelow_MemoryStream() {
    use rust_skia::stream::remaining_length_is_below as below;
    let mut r = Reporter::default();
    let mut s = SkMemoryStream::make_uninitialized(100);
    check!(r, !below(&s, 0));
    check!(r, !below(&s, 90));
    check!(r, !below(&s, 100));
    check!(r, below(&s, 101));
    check!(r, below(&s, usize::MAX));
    let mut buff = [0; 75];
    check!(r, s.read(&mut buff) == 75);
    check!(r, !below(&s, 0));
    check!(r, !below(&s, 24));
    check!(r, !below(&s, 25));
    check!(r, below(&s, 26));
    check!(r, below(&s, 100));
    check!(r, below(&s, usize::MAX));
    r.finish();
}
struct DumbStream<'a> {
    data: &'a [u8],
    index: usize,
}
impl SkStream for DumbStream<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> usize {
        let count = (self.data.len() - self.index).min(buffer.len());
        if count != 0 {
            buffer[..count].copy_from_slice(&self.data[self.index..self.index + count]);
            self.index += count;
        }
        count
    }
    fn is_at_end(&mut self) -> bool {
        self.index == self.data.len()
    }
    fn has_position(&self) -> bool {
        false
    }
    fn position(&self) -> usize {
        0
    }
    fn seek(&mut self, _: usize) -> bool {
        false
    }
    // No length or positioning capability: preserve the original DumbStream's unknown-length branch.
}
fn stream_copy_test(r: &mut Reporter, src: &[u8], stream: &mut dyn SkStream) {
    let mut tgt = SkDynamicMemoryWStream::new();
    if !rust_skia::stream::sk_stream_copy(&mut tgt, stream) {
        check!(r, false);
        return;
    }
    let data = tgt.detach_as_data();
    if data.size() != src.len() {
        check!(r, false);
        return;
    }
    if data.bytes() != src {
        check!(r, false);
    }
}
// tests/StreamTest.cpp:482
#[test]
fn DynamicMemoryWStream_detachAsData() {
    let mut r = Reporter::default();
    let n = 40000;
    let mut stream = SkDynamicMemoryWStream::new();
    for _ in 0..n {
        stream.write(ABC);
    }
    check!(r, stream.bytes_written() == n * ABC.len());
    let data = stream.detach_as_data();
    check!(r, data.size() == n * ABC.len());
    let mut ptr = 0;
    for _ in 0..n {
        if &data.bytes()[ptr..ptr + ABC.len()] != ABC {
            check!(r, false);
            r.finish();
            return;
        }
        ptr += ABC.len();
    }
    r.finish();
}
// tests/StreamTest.cpp:502
#[test]
fn DynamicMemoryWStream_detachAsVector() {
    let mut r = Reporter::default();
    let n = 40000;
    let mut stream = SkDynamicMemoryWStream::new();
    for _ in 0..n {
        stream.write(ABC);
    }
    check!(r, stream.bytes_written() == n * ABC.len());
    let data = stream.detach_as_vector();
    check!(r, data.len() == n * ABC.len());
    for i in 0..n {
        if &data[i * ABC.len()..(i + 1) * ABC.len()] != ABC {
            check!(r, false);
            r.finish();
            return;
        }
    }
    r.finish();
}
// tests/StreamTest.cpp:520
#[test]
fn StreamCopy() {
    let mut r = Reporter::default();
    let mut random = SkRandom::from_seed(123456);
    let mut src = vec![0; 10000];
    for j in 0..10000 {
        src[j] = (random.next_u() & 0xff) as u8;
    }
    let mut dumb = DumbStream {
        data: &src,
        index: 0,
    };
    stream_copy_test(&mut r, &src, &mut dumb);
    let mut smart = SkMemoryStream::new(SkData::make_with_copy(&src));
    stream_copy_test(&mut r, &src, &mut smart);
    r.finish();
}
// tests/StreamTest.cpp:534
#[test]
fn StreamEmptyStreamMemoryBase() {
    let mut r = Reporter::default();
    let mut tmp = SkDynamicMemoryWStream::new();
    let asset = tmp.detach_as_stream();
    check!(r, asset.get_memory_base().is_none());
    r.finish();
}
fn test_all(
    r: &mut Reporter,
    s: &mut SkFILEStream,
    expected: &[u8],
    size: usize,
    middle: usize,
    recurse: bool,
) {
    let remaining = expected.len();
    check!(r, s.length() == remaining);
    check!(r, s.position() == 0);
    // test_full_read
    {
        let mut actual = vec![0; remaining];
        check!(r, s.read(&mut actual) == remaining);
        check!(r, expected == actual);
        check!(r, s.position() == s.length());
        check!(r, s.is_at_end());
    }
    // test_rewind
    {
        check!(r, s.rewind());
        check!(r, s.position() == 0);
        let mut actual = vec![0; remaining];
        check!(r, s.read(&mut actual) == remaining);
        check!(r, expected == actual);
    }
    // test_move: signed long is 64-bit on macOS; do not truncate LONG_MIN to i32.
    {
        check!(r, s.move_long(-(size as i64)));
        check!(r, s.position() == 0);
        check!(r, s.move_long(i64::MIN));
        check!(r, s.position() == 0);
        let mut actual = vec![0; remaining];
        check!(r, s.read(&mut actual) == remaining);
        check!(r, expected == actual);
        check!(r, s.is_at_end());
        check!(r, s.position() == remaining);
        check!(r, s.move_long(1));
        check!(r, s.is_at_end());
        check!(r, s.position() == remaining);
    }
    // test_seek
    {
        let arbitrary = middle / 2;
        check!(r, s.seek(arbitrary));
        check!(r, s.position() == arbitrary);
        let mini = remaining - arbitrary;
        let mut actual = vec![0; mini];
        check!(r, s.read(&mut actual) == mini);
        check!(r, &expected[arbitrary..] == actual);
    }
    // test_seek_beginning
    {
        check!(r, s.seek(0));
        check!(r, s.position() == 0);
        let mut actual = vec![0; remaining];
        check!(r, s.read(&mut actual) == remaining);
        check!(r, expected == actual);
    }
    // test_seek_end
    {
        check!(r, s.is_at_end());
        check!(r, s.seek(remaining + 1));
        check!(r, s.is_at_end());
        check!(r, s.position() == remaining);
        let middle = remaining / 2;
        check!(r, s.seek(middle));
        check!(r, !s.is_at_end());
        check!(r, s.position() == middle);
        check!(r, s.seek(remaining * 2));
        check!(r, s.is_at_end());
        check!(r, s.position() == remaining);
        check!(r, s.seek(i64::MAX as usize));
        check!(r, s.is_at_end());
        check!(r, s.position() == remaining);
    }
    if recurse {
        if let Some(mut duplicate) = s.duplicate() {
            test_all(r, &mut duplicate, expected, size, middle, false);
        } else {
            check!(r, false);
        }
        if let Some(mut fork) = s.fork() {
            check!(r, fork.is_at_end());
            check!(r, fork.length() == remaining);
            check!(r, fork.rewind());
            test_all(r, &mut fork, expected, size, middle, false);
        } else {
            check!(r, false);
        }
    }
}
// tests/StreamTest.cpp:540
#[test]
fn FILEStreamWithOffset() {
    let mut r = Reporter::default();
    let root = resource_root();
    if root.as_os_str().is_empty() {
        return;
    }
    let filename = root.join("images/baby_tux.png");
    let Some(mut stream1) = SkFILEStream::new(&filename) else {
        check!(r, false);
        r.finish();
        return;
    };
    check!(r, stream1.has_length());
    check!(r, stream1.has_position());
    let size = stream1.length();
    let middle = size / 2;
    if !stream1.seek(middle) {
        check!(r, false);
        r.finish();
        return;
    }
    check!(r, stream1.position() == middle);
    let Ok(mut file) = File::open(&filename) else {
        check!(r, false);
        r.finish();
        return;
    };
    if file.seek(SeekFrom::Start(middle as u64)).is_err() {
        check!(r, false);
        r.finish();
        return;
    }
    let mut stream2 = sk_stream_file_from_stdio(file).unwrap();
    let remaining = size - middle;
    let mut expected = vec![0; remaining];
    check!(r, stream1.read(&mut expected) == remaining);
    test_all(&mut r, &mut stream2, &expected, size, middle, true);
    r.finish();
}
// tests/StreamTest.cpp:701
#[test]
fn RBuffer() {
    let mut r = Reporter::default();
    let value = 0i32.to_ne_bytes();
    let mut buffer = SkRBuffer::new_with_size(&value, 4);
    check!(r, buffer.is_valid());
    let tmp = buffer.read(4);
    check!(r, tmp.is_some());
    check!(r, buffer.is_valid());
    check!(r, buffer.read(4).is_none());
    check!(r, !buffer.is_valid());
    r.finish();
}
