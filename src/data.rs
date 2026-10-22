//! `SkData` and `SkDataTable` owned byte storage.

use std::fs::File;
use std::io::Read;
use std::os::unix::io::FromRawFd;
use std::sync::{Arc, OnceLock};

use crate::containers::SkSpan;

pub struct NoArg;
pub struct BytesArg;

pub trait DataRelease<A> {
    fn release(self, bytes: &[u8]);
}
impl<F: FnOnce() + Send + Sync + 'static> DataRelease<NoArg> for F {
    fn release(self, _bytes: &[u8]) {
        self()
    }
}
impl<F: FnOnce(&[u8]) + Send + Sync + 'static> DataRelease<BytesArg> for F {
    fn release(self, bytes: &[u8]) {
        self(bytes)
    }
}

type ReleaseProc = Box<dyn FnOnce(&[u8]) + Send + Sync>;

struct DataInner {
    bytes: Vec<u8>,
    borrowed: Option<&'static [u8]>,
    release: Option<ReleaseProc>,
    parent: Option<Arc<DataInner>>,
    offset: usize,
    len: usize,
}

impl DataInner {
    fn slice(&self) -> &[u8] {
        if let Some(parent) = &self.parent {
            &parent.slice()[self.offset..self.offset + self.len]
        } else if let Some(borrowed) = self.borrowed {
            &borrowed[..self.len]
        } else {
            &self.bytes[..self.len]
        }
    }

    fn prefix_ptr(&self) -> *const u8 {
        if let Some(parent) = &self.parent {
            unsafe { parent.prefix_ptr().add(self.offset) }
        } else if let Some(borrowed) = self.borrowed {
            borrowed.as_ptr()
        } else {
            self.bytes.as_ptr()
        }
    }
}

impl Drop for DataInner {
    fn drop(&mut self) {
        if let Some(release) = self.release.take() {
            release(self.slice());
        }
    }
}

#[derive(Clone)]
pub struct SkData {
    inner: Arc<DataInner>,
}

impl std::fmt::Debug for SkData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkData")
            .field("size", &self.size())
            .finish()
    }
}

fn empty_data() -> Arc<DataInner> {
    static EMPTY: OnceLock<Arc<DataInner>> = OnceLock::new();
    EMPTY
        .get_or_init(|| {
            Arc::new(DataInner {
                bytes: Vec::new(),
                borrowed: None,
                release: None,
                parent: None,
                offset: 0,
                len: 0,
            })
        })
        .clone()
}

impl Default for SkData {
    fn default() -> Self {
        Self::make_empty()
    }
}

impl SkData {
    pub fn make_empty() -> Self {
        Self {
            inner: empty_data(),
        }
    }

    fn from_vec(bytes: Vec<u8>) -> Self {
        if bytes.is_empty() {
            return Self::make_empty();
        }
        let len = bytes.len();
        Self {
            inner: Arc::new(DataInner {
                bytes,
                borrowed: None,
                release: None,
                parent: None,
                offset: 0,
                len,
            }),
        }
    }

    pub fn make_with_copy(bytes: &[u8]) -> Self {
        Self::from_vec(bytes.to_vec())
    }

    pub fn make_uninitialized(size: usize) -> Self {
        Self::from_vec(vec![0; size])
    }

    pub fn make_uninitialized_vec(size: usize) -> Vec<u8> {
        vec![0; size]
    }

    pub fn make_with_proc<F, A>(bytes: Vec<u8>, release: F) -> Self
    where
        F: DataRelease<A> + Send + Sync + 'static,
        A: 'static,
    {
        let len = bytes.len();
        let release: ReleaseProc = Box::new(move |bytes| release.release(bytes));
        Self {
            inner: Arc::new(DataInner {
                bytes,
                borrowed: None,
                release: Some(release),
                parent: None,
                offset: 0,
                len,
            }),
        }
    }

    pub fn make_with_cstring(value: Option<&str>) -> Self {
        match value {
            Some(s) => {
                let mut bytes = s.as_bytes().to_vec();
                bytes.push(0);
                Self::from_vec(bytes)
            }
            None => Self::from_vec(vec![0]),
        }
    }

    pub fn make_from_malloc(bytes: Vec<u8>) -> Self {
        Self::from_vec(bytes)
    }

    pub fn make_without_copy(bytes: &'static [u8]) -> Self {
        Self {
            inner: Arc::new(DataInner {
                bytes: Vec::new(),
                borrowed: Some(bytes),
                release: None,
                parent: None,
                offset: 0,
                len: bytes.len(),
            }),
        }
    }

    pub fn make_from_file(path: &std::path::Path) -> Option<Self> {
        let mut file = File::open(path).ok()?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).ok()?;
        Some(Self::from_vec(bytes))
    }

    pub fn make_from_fd(fd: std::os::unix::io::RawFd) -> Option<Self> {
        let mut file = unsafe { File::from_raw_fd(fd) };
        let mut bytes = Vec::new();
        let result = file.read_to_end(&mut bytes);
        std::mem::forget(file);
        result.ok()?;
        Some(Self::from_vec(bytes))
    }

    pub fn make_with_cstring_bytes(value: Option<&[u8]>) -> Self {
        match value {
            Some(s) => {
                let mut bytes = s.to_vec();
                bytes.push(0);
                Self::from_vec(bytes)
            }
            None => Self::from_vec(vec![0]),
        }
    }

    pub fn size(&self) -> usize {
        self.inner.slice().len()
    }
    pub fn len(&self) -> usize {
        self.size()
    }
    pub fn empty(&self) -> bool {
        self.size() == 0
    }
    pub fn is_empty(&self) -> bool {
        self.empty()
    }
    pub fn bytes(&self) -> &[u8] {
        self.inner.slice()
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes()
    }
    pub fn data(&self) -> *const u8 {
        self.inner.prefix_ptr()
    }
    pub fn byte_span(&self) -> SkSpan<'_, u8> {
        SkSpan::new(self.bytes())
    }
    pub fn writable_data(&mut self) -> *mut u8 {
        self.inner.prefix_ptr() as *mut u8
    }

    pub fn equals(&self, other: &SkData) -> bool {
        if self.size() == 0 && other.size() == 0 {
            return true;
        }
        self.bytes() == other.bytes()
    }

    pub fn equals_bytes(&self, other: &[u8]) -> bool {
        self.bytes() == other
    }

    pub fn share_subset(&self, offset: usize, length: usize) -> Option<SkData> {
        let size = self.size();
        if offset > size || length > size - offset {
            return None;
        }
        if length == 0 {
            return Some(Self::make_empty());
        }
        if offset == 0 && length == size {
            return Some(self.clone());
        }
        Some(SkData {
            inner: Arc::new(DataInner {
                bytes: Vec::new(),
                borrowed: None,
                release: None,
                parent: Some(self.inner.clone()),
                offset,
                len: length,
            }),
        })
    }

    pub fn copy_subset(&self, offset: usize, length: usize) -> Option<SkData> {
        let size = self.size();
        if offset > size || length > size - offset {
            return None;
        }
        Some(Self::from_vec(
            self.bytes()[offset..offset + length].to_vec(),
        ))
    }

    pub fn make_subset(&self, offset: usize, length: usize) -> Self {
        self.share_subset(offset, length)
            .unwrap_or_else(Self::make_empty)
    }

    pub fn copy_range(&self, offset: usize, length: usize) -> usize {
        let size = self.size();
        if offset >= size || length == 0 {
            return 0;
        }
        (size - offset).min(length)
    }

    pub fn copy_range_into(&self, offset: usize, length: usize, dest: Option<&mut [u8]>) -> usize {
        let n = self.copy_range(offset, length);
        if let Some(dest) = dest {
            dest[..n].copy_from_slice(&self.bytes()[offset..offset + n]);
        }
        n
    }

    pub fn equals_opt(a: Option<&SkData>, b: Option<&SkData>) -> bool {
        match (a, b) {
            (Some(a), Some(b)) => a.equals(b),
            (None, None) => true,
            _ => false,
        }
    }
}

impl PartialEq for SkData {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}
impl Eq for SkData {}

#[derive(Clone, Debug)]
enum TableStorage {
    Fixed { data: Vec<u8>, elem_size: usize },
    Dir(Vec<(Vec<u8>, usize)>),
}

pub struct SkDataTable {
    storage: TableStorage,
    count: usize,
    release: Option<Box<dyn FnOnce()>>,
}

impl Drop for SkDataTable {
    fn drop(&mut self) {
        if let Some(release) = self.release.take() {
            release();
        }
    }
}

impl Default for SkDataTable {
    fn default() -> Self {
        Self::make_empty()
    }
}

impl SkDataTable {
    pub fn make_empty() -> Self {
        Self {
            storage: TableStorage::Dir(Vec::new()),
            count: 0,
            release: None,
        }
    }

    pub fn make_copy_array(array: &[u8], elem_size: usize, count: usize) -> Self {
        if count == 0 || elem_size == 0 {
            return Self::make_empty();
        }
        let data = array[..elem_size * count].to_vec();
        Self {
            storage: TableStorage::Fixed { data, elem_size },
            count,
            release: None,
        }
    }

    pub fn make_copy_arrays(ptrs: &[*const u8], sizes: &[usize], count: usize) -> Self {
        if count == 0 {
            return Self::make_empty();
        }
        let mut dir = Vec::with_capacity(count);
        for i in 0..count {
            let bytes = if sizes[i] == 0 || ptrs[i].is_null() {
                Vec::new()
            } else {
                unsafe { std::slice::from_raw_parts(ptrs[i], sizes[i]).to_vec() }
            };
            dir.push((bytes, sizes[i]));
        }
        Self {
            storage: TableStorage::Dir(dir),
            count,
            release: None,
        }
    }

    pub fn make_array_proc(
        ptrs: &[*const u8],
        sizes: &[usize],
        count: usize,
        free: Option<Box<dyn FnOnce()>>,
    ) -> Self {
        let mut table = Self::make_copy_arrays(ptrs, sizes, count);
        table.release = free;
        table
    }

    pub fn count(&self) -> usize {
        self.count
    }
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    pub fn empty(&self) -> bool {
        self.is_empty()
    }

    pub fn at(&self, index: usize) -> (*const u8, usize) {
        match &self.storage {
            TableStorage::Fixed { data, elem_size } => {
                (unsafe { data.as_ptr().add(index * elem_size) }, *elem_size)
            }
            TableStorage::Dir(entries) => (entries[index].0.as_ptr(), entries[index].1),
        }
    }

    pub fn at_size(&self, index: usize) -> usize {
        self.at(index).1
    }

    pub fn at_bytes(&self, index: usize) -> &[u8] {
        match &self.storage {
            TableStorage::Fixed { data, elem_size } => {
                &data[index * elem_size..(index + 1) * elem_size]
            }
            TableStorage::Dir(entries) => &entries[index].0[..entries[index].1],
        }
    }

    pub fn at_str(&self, index: usize) -> &[u8] {
        self.at_bytes(index)
    }

    pub fn at_t<T: Copy>(&self, index: usize) -> &T {
        assert_eq!(self.at_size(index), std::mem::size_of::<T>());
        unsafe { &*(self.at(index).0 as *const T) }
    }
}
