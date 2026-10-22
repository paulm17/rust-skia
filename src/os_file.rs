//! POSIX/std stream file helpers and `SkOSPath`.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::MetadataExt;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::path::{Path, PathBuf};

use crate::data::SkData;
use crate::string::SkString;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkFILEFlag {
    Read,
    Write,
    ReadWrite,
}

pub fn sk_fopen(path: impl AsRef<Path>, flag: SkFILEFlag) -> Option<File> {
    let mut options = OpenOptions::new();
    match flag {
        SkFILEFlag::Read => {
            options.read(true);
        }
        SkFILEFlag::Write => {
            options.write(true).create(true).truncate(true);
        }
        SkFILEFlag::ReadWrite => {
            options.read(true).write(true).create(true);
        }
    }
    options.open(path).ok()
}

pub fn sk_fclose(file: Option<File>) {
    drop(file);
}

pub fn sk_fgetsize(file: &mut File) -> usize {
    let current = file.stream_position().unwrap_or(0);
    let end = file.seek(SeekFrom::End(0)).unwrap_or(current);
    let _ = file.seek(SeekFrom::Start(current));
    end as usize
}

pub fn sk_ftell(file: &mut File) -> usize {
    file.stream_position().unwrap_or(0) as usize
}

pub fn sk_fwrite(file: &mut File, buffer: &[u8]) -> usize {
    file.write_all(buffer).map(|_| buffer.len()).unwrap_or(0)
}

pub fn sk_fflush(file: &mut File) {
    let _ = file.flush();
}

pub fn sk_fsync(file: &File) -> bool {
    file.sync_all().is_ok()
}

pub fn sk_exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}

pub fn sk_isdir(path: impl AsRef<Path>) -> bool {
    path.as_ref().is_dir()
}

pub fn sk_mkdir(path: impl AsRef<Path>) -> bool {
    fs::create_dir(path).is_ok()
}

pub fn sk_fidentical(a: &File, b: &File) -> bool {
    match (a.metadata(), b.metadata()) {
        (Ok(a), Ok(b)) => a.dev() == b.dev() && a.ino() == b.ino(),
        _ => false,
    }
}

pub fn sk_fileno(file: &File) -> i32 {
    file.as_raw_fd()
}

pub fn sk_fdmmap(fd: i32) -> Option<SkData> {
    let mut file = unsafe { File::from_raw_fd(fd) };
    let mut bytes = Vec::new();
    let result = file.read_to_end(&mut bytes);
    std::mem::forget(file);
    result.ok()?;
    Some(SkData::make_from_malloc(bytes))
}

pub fn sk_fmmap(file: &mut File) -> Option<SkData> {
    let current = file.stream_position().ok()?;
    file.seek(SeekFrom::Start(0)).ok()?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).ok()?;
    let _ = file.seek(SeekFrom::Start(current));
    Some(SkData::make_from_malloc(bytes))
}

pub fn sk_fmunmap(_data: SkData) {}

pub fn sk_qread(file: &File, buffer: &mut [u8], offset: u64) -> usize {
    let mut file = match file.try_clone() {
        Ok(f) => f,
        Err(_) => return usize::MAX,
    };
    if file.seek(SeekFrom::Start(offset)).is_err() {
        return usize::MAX;
    }
    file.read(buffer).unwrap_or(usize::MAX)
}

pub struct SkOSFileIter {
    entries: Vec<PathBuf>,
    index: usize,
    suffix: Option<String>,
    want_dirs: bool,
}

impl SkOSFileIter {
    pub fn new(path: impl AsRef<Path>, suffix: Option<&str>) -> Self {
        let mut entries = Vec::new();
        if let Ok(read_dir) = fs::read_dir(path) {
            for entry in read_dir.flatten() {
                entries.push(entry.path());
            }
        }
        entries.sort();
        Self {
            entries,
            index: 0,
            suffix: suffix.map(str::to_string),
            want_dirs: false,
        }
    }

    pub fn next(&mut self) -> Option<PathBuf> {
        while self.index < self.entries.len() {
            let path = self.entries[self.index].clone();
            self.index += 1;
            let is_dir = path.is_dir();
            if is_dir != self.want_dirs {
                continue;
            }
            if let Some(suffix) = &self.suffix {
                if !path.to_string_lossy().ends_with(suffix) {
                    continue;
                }
            }
            return Some(path);
        }
        None
    }
}

pub struct SkOSPath;

impl SkOSPath {
    pub fn join(root: &str, relative: &str) -> SkString {
        if root.is_empty() {
            return SkString::from_str(relative);
        }
        if root.ends_with('/') {
            SkString::from_str(&format!("{root}{relative}"))
        } else {
            SkString::from_str(&format!("{root}/{relative}"))
        }
    }

    pub fn basename(path: &str) -> SkString {
        let path = path.trim_end_matches('/');
        match path.rsplit_once('/') {
            Some((_, name)) => SkString::from_str(name),
            None => SkString::from_str(path),
        }
    }

    pub fn dirname(path: &str) -> SkString {
        let trimmed = path.trim_end_matches('/');
        match trimmed.rfind('/') {
            Some(0) if trimmed.len() == 1 => SkString::from_str("/"),
            Some(0) => SkString::from_str("/"),
            Some(pos) => SkString::from_str(&path[..pos]),
            None => SkString::new(),
        }
    }
}
