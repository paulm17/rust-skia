//! Checksums, MD5, endian helpers, Base64, and parsing.

use crate::containers::SkTArray;
use crate::string::{SkStrSplitMode, SkString};

pub fn sk_checksum_mix(mut hash: u32) -> u32 {
    hash ^= hash >> 16;
    hash = hash.wrapping_mul(0x85ebca6b);
    hash ^= hash >> 13;
    hash = hash.wrapping_mul(0xc2b2ae35);
    hash ^= hash >> 16;
    hash
}

pub fn sk_checksum_cheap_mix(mut hash: u32) -> u32 {
    hash ^= hash >> 16;
    hash = hash.wrapping_mul(0x85ebca6b);
    hash ^= hash >> 16;
    hash
}

fn wymum(a: &mut u64, b: &mut u64) {
    let r = (*a as u128) * (*b as u128);
    *a = r as u64;
    *b = (r >> 64) as u64;
}

fn wymix(mut a: u64, mut b: u64) -> u64 {
    wymum(&mut a, &mut b);
    a ^ b
}

fn wyr4(p: &[u8]) -> u64 {
    u32::from_le_bytes([p[0], p[1], p[2], p[3]]) as u64
}

fn wyr8(p: &[u8]) -> u64 {
    u64::from_le_bytes([p[0], p[1], p[2], p[3], p[4], p[5], p[6], p[7]])
}

fn wyr3(p: &[u8], k: usize) -> u64 {
    ((p[0] as u64) << 16) | ((p[k >> 1] as u64) << 8) | p[k - 1] as u64
}

fn wyhash(key: &[u8], mut seed: u64) -> u64 {
    const SECRET: [u64; 4] = [
        0xa076_1d64_78bd_642f,
        0xe703_7ed1_a0b4_28db,
        0x8ebc_6af0_9c88_c6e3,
        0x5899_65cc_7537_4cc3,
    ];
    let mut p = key;
    seed ^= wymix(seed ^ SECRET[0], SECRET[1]);
    let mut a;
    let mut b;
    if p.len() <= 16 {
        if p.len() >= 4 {
            a = (wyr4(p) << 32) | wyr4(&p[(p.len() >> 3) << 2..]);
            b = (wyr4(&p[p.len() - 4..]) << 32) | wyr4(&p[p.len() - 4 - ((p.len() >> 3) << 2)..]);
        } else if !p.is_empty() {
            a = wyr3(p, p.len());
            b = 0;
        } else {
            a = 0;
            b = 0;
        }
    } else {
        let mut i = p.len();
        if i > 48 {
            let mut see1 = seed;
            let mut see2 = seed;
            loop {
                seed = wymix(wyr8(p) ^ SECRET[1], wyr8(&p[8..]) ^ seed);
                see1 = wymix(wyr8(&p[16..]) ^ SECRET[2], wyr8(&p[24..]) ^ see1);
                see2 = wymix(wyr8(&p[32..]) ^ SECRET[3], wyr8(&p[40..]) ^ see2);
                p = &p[48..];
                i -= 48;
                if i <= 48 {
                    break;
                }
            }
            seed ^= see1 ^ see2;
        }
        while i > 16 {
            seed = wymix(wyr8(p) ^ SECRET[1], wyr8(&p[8..]) ^ seed);
            i -= 16;
            p = &p[16..];
        }
        a = wyr8(&key[key.len() - 16..]);
        b = wyr8(&key[key.len() - 8..]);
    }
    a ^= SECRET[1];
    b ^= seed;
    wymum(&mut a, &mut b);
    wymix(a ^ SECRET[0] ^ key.len() as u64, b ^ SECRET[1])
}

pub fn sk_checksum_hash32(data: &[u8], seed: u32) -> u32 {
    wyhash(data, seed as u64) as u32
}

pub fn sk_checksum_hash64(data: &[u8], seed: u64) -> u64 {
    wyhash(data, seed)
}

pub fn sk_checksum_hash_string(data: impl AsRef<str>) -> u32 {
    let data = data.as_ref();
    sk_checksum_hash32(data.as_bytes(), 0)
}

pub struct SkMD5 {
    state: [u32; 4],
    count: u64,
    buffer: [u8; 64],
    buffer_len: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkMD5Digest {
    pub data: [u8; 16],
}

impl SkMD5Digest {
    pub fn hex(&self) -> SkString {
        let mut out = String::new();
        for b in self.data {
            out.push_str(&format!("{b:02x}"));
        }
        SkString::from_str(&out)
    }
}

impl Default for SkMD5 {
    fn default() -> Self {
        Self::new()
    }
}

impl SkMD5 {
    pub fn new() -> Self {
        Self {
            state: [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476],
            count: 0,
            buffer: [0; 64],
            buffer_len: 0,
        }
    }

    pub fn write(&mut self, data: &[u8]) {
        self.count = self.count.wrapping_add(data.len() as u64);
        let mut offset = 0;
        if self.buffer_len > 0 {
            let take = (64 - self.buffer_len).min(data.len());
            self.buffer[self.buffer_len..self.buffer_len + take].copy_from_slice(&data[..take]);
            self.buffer_len += take;
            offset += take;
            if self.buffer_len == 64 {
                let block = self.buffer;
                self.process(&block);
                self.buffer_len = 0;
            } else {
                return;
            }
        }
        while offset + 64 <= data.len() {
            let mut block = [0u8; 64];
            block.copy_from_slice(&data[offset..offset + 64]);
            self.process(&block);
            offset += 64;
        }
        let remaining = data.len() - offset;
        self.buffer[..remaining].copy_from_slice(&data[offset..]);
        self.buffer_len = remaining;
    }

    pub fn finish(mut self) -> SkMD5Digest {
        let bit_len = self.count.wrapping_mul(8);
        let mut tail = [0u8; 128];
        let mut tail_len = self.buffer_len;
        tail[..tail_len].copy_from_slice(&self.buffer[..tail_len]);
        tail[tail_len] = 0x80;
        tail_len += 1;
        let total = ((tail_len + 8 + 63) / 64) * 64;
        tail[total - 8..total].copy_from_slice(&bit_len.to_le_bytes());
        for i in 0..total / 64 {
            let mut block = [0u8; 64];
            block.copy_from_slice(&tail[i * 64..i * 64 + 64]);
            self.process(&block);
        }
        let mut out = [0u8; 16];
        for (i, word) in self.state.iter().enumerate() {
            out[i * 4..i * 4 + 4].copy_from_slice(&word.to_le_bytes());
        }
        SkMD5Digest { data: out }
    }

    pub fn finish_hex(self) -> SkString {
        self.finish().hex()
    }

    fn process(&mut self, block: &[u8; 64]) {
        let mut m = [0u32; 16];
        for i in 0..16 {
            m[i] = u32::from_le_bytes([
                block[i * 4],
                block[i * 4 + 1],
                block[i * 4 + 2],
                block[i * 4 + 3],
            ]);
        }
        let (mut a, mut b, mut c, mut d) =
            (self.state[0], self.state[1], self.state[2], self.state[3]);
        let s = [
            7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20,
            5, 9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
            6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
        ];
        for i in 0..64 {
            let (f, g) = if i < 16 {
                ((b & c) | (!b & d), i)
            } else if i < 32 {
                ((d & b) | (!d & c), (5 * i + 1) % 16)
            } else if i < 48 {
                (b ^ c ^ d, (3 * i + 5) % 16)
            } else {
                (c ^ (b | !d), (7 * i) % 16)
            };
            let k = (((i + 1) as f64).sin().abs() * 4294967296.0) as u32;
            let tmp = d;
            d = c;
            c = b;
            b = b.wrapping_add(
                a.wrapping_add(f)
                    .wrapping_add(k)
                    .wrapping_add(m[g])
                    .rotate_left(s[i]),
            );
            a = tmp;
        }
        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
    }
}

pub const fn sk_bswap16(value: u16) -> u16 {
    value.swap_bytes()
}
pub const fn sk_bswap32(value: u32) -> u32 {
    value.swap_bytes()
}
pub const fn sk_bswap64(value: u64) -> u64 {
    value.swap_bytes()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkBase64Error {
    None,
    BadChar,
    Pad,
}

pub fn sk_base64_encoded_size(size: usize) -> usize {
    size.div_ceil(3) * 4
}

pub fn sk_base64_encode(src: &[u8]) -> Vec<u8> {
    sk_base64_encode_with(src, None)
}

pub fn sk_base64_encode_with(src: &[u8], alphabet: Option<&[u8; 64]>) -> Vec<u8> {
    let default = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let table = alphabet.unwrap_or(default);
    let mut out = Vec::with_capacity(sk_base64_encoded_size(src.len()));
    for chunk in src.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(table[((n >> 18) & 0x3F) as usize]);
        out.push(table[((n >> 12) & 0x3F) as usize]);
        out.push(if chunk.len() > 1 {
            table[((n >> 6) & 0x3F) as usize]
        } else {
            b'='
        });
        out.push(if chunk.len() > 2 {
            table[(n & 0x3F) as usize]
        } else {
            b'='
        });
    }
    out
}

pub fn sk_base64_decode(src: &[u8]) -> Result<Vec<u8>, SkBase64Error> {
    let mut clean = Vec::new();
    for &b in src {
        if b == 0 {
            break;
        }
        if b <= b' ' {
            continue;
        }
        clean.push(b);
    }
    if clean.len() % 4 == 1 {
        return Err(SkBase64Error::BadChar);
    }
    let mut out = Vec::new();
    let mut i = 0;
    let mut pad_seen = false;
    while i < clean.len() {
        let mut vals = [0i32; 4];
        let mut count = 0;
        for j in 0..4 {
            if i + j >= clean.len() {
                vals[j] = -2;
                count += 1;
                continue;
            }
            let b = clean[i + j];
            match b {
                b'=' => {
                    vals[j] = -2;
                    pad_seen = true;
                    count += 1;
                }
                _ => {
                    if pad_seen {
                        return Err(SkBase64Error::Pad);
                    }
                    vals[j] = decode_base64_char(b).ok_or(SkBase64Error::BadChar)?;
                }
            }
        }
        if count > 2 {
            return Err(SkBase64Error::Pad);
        }
        let n = ((vals[0] as u32) << 18)
            | ((vals[1] as u32) << 12)
            | (((vals[2].max(0) as u32) & 0x3F) << 6)
            | ((vals[3].max(0) as u32) & 0x3F);
        out.push((n >> 16) as u8);
        if count < 2 {
            out.push((n >> 8) as u8);
        }
        if count < 1 {
            out.push(n as u8);
        }
        i += 4;
    }
    Ok(out)
}

fn decode_base64_char(b: u8) -> Option<i32> {
    match b {
        b'A'..=b'Z' => Some((b - b'A') as i32),
        b'a'..=b'z' => Some((b - b'a' + 26) as i32),
        b'0'..=b'9' => Some((b - b'0' + 52) as i32),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

pub fn sk_parse_find_s32(s: &str) -> Option<i32> {
    s.trim().parse().ok()
}
pub fn sk_parse_find_scalar(s: &str) -> Option<f32> {
    let v: f64 = s.trim().parse().ok()?;
    Some(v as f32)
}
pub fn sk_parse_find_bool(s: &str) -> Option<bool> {
    match s.trim() {
        "true" | "yes" | "1" => Some(true),
        "false" | "no" | "0" => Some(false),
        _ => None,
    }
}
pub fn sk_parse_find_hex(s: &str) -> Option<u32> {
    let s = s.trim();
    let s = s
        .strip_prefix("0x")
        .or_else(|| s.strip_prefix("0X"))
        .unwrap_or(s);
    u32::from_str_radix(s, 16).ok()
}
pub fn sk_parse_find_scalars(s: &str) -> SkTArray<f32> {
    let mut out = SkTArray::new();
    for part in s.split(|c: char| c == ',' || c.is_whitespace()) {
        if let Some(v) = sk_parse_find_scalar(part) {
            out.push_back(v);
        }
    }
    out
}
pub fn sk_parse_find_list(s: &str) -> Vec<SkString> {
    crate::string::sk_str_split(s, ",", SkStrSplitMode::Coalesce)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_md5_rfc1321() {
        let mut md5 = SkMD5::new();
        md5.write(b"");
        assert_eq!(
            md5.finish().hex().as_bytes(),
            b"d41d8cd98f00b204e9800998ecf8427e"
        );
        let mut md5 = SkMD5::new();
        md5.write(b"abc");
        assert_eq!(
            md5.finish().hex().as_bytes(),
            b"900150983cd24fb0d6963f7d28e17f72"
        );
        let mut md5 = SkMD5::new();
        md5.write(b"The quick brown fox jumps over the lazy dog");
        assert_eq!(
            md5.finish().hex().as_bytes(),
            b"9e107d9d372bb6826bd81d3542a419d6"
        );
    }

    #[test]
    fn dev_base64_round_trip() {
        let encoded = sk_base64_encode(b"hello");
        assert_eq!(encoded, b"aGVsbG8=");
        assert_eq!(sk_base64_decode(&encoded).unwrap(), b"hello");
        assert!(sk_base64_decode(b"!!!!").is_err());
    }
}
