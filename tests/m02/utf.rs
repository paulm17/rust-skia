use super::*;
#[test]
fn SkUTF_UTF16() {
    let mut r = Reporter::default();
    for uni in [0x10000, 0x18080, 0x20202, 0xFFFFF, 0x101234] {
        let buf = sk_utf8_to_utf16(uni);
        check!(r, buf.len() == 2);
        check!(r, sk_utf16_count(&buf) == 1);
        let mut p = 0;
        let c = sk_utf16_next(&buf, &mut p);
        check!(r, c == uni as i32);
        check!(r, p == 2);
    }
    r.finish();
}
#[test]
fn SkUTF_UTF8() {
    let mut r = Reporter::default();
    let cases: &[(&[u8], i32)] = &[
        (b"a", 97),
        (b"\x7f", 0x7f),
        (b"\xC2\x80", 0x80),
        (b"\xC3\x83", (3 << 6) | 3),
        (b"\xDF\xBF", 0x7ff),
        (b"\xE0\xA0\x80", 0x800),
        (b"\xE0\xB0\xB8", 0xC38),
        (b"\xE3\x83\x83", (3 << 12) | (3 << 6) | 3),
        (b"\xEF\xBF\xBF", 0xFFFF),
        (b"\xF0\x90\x80\x80", 0x10000),
        (b"\xF3\x83\x83\x83", (3 << 18) | (3 << 12) | (3 << 6) | 3),
    ];
    for (b, uni) in cases {
        let mut p = 0;
        let n = sk_utf8_count(b);
        let u = sk_utf8_next(b, &mut p);
        check!(r, n == 1);
        check!(r, u == *uni);
        check!(r, p == b.len());
    }
    r.finish();
}
#[test]
fn SkUTF_CountUTF8() {
    let mut r = Reporter::default();
    let cases: &[(i32, &[u8])] = &[
        (0, b""),
        (1, b"X"),
        (2, b"XX"),
        (1, b"\xC2\xA1"),
        (2, b"X\xC2\xA1"),
        (3, b"XX\xC2\xA1"),
        (1, b"\xE1\xA1\xA1"),
        (2, b"X\xE1\xA1\xA1"),
        (3, b"XX\xE1\xA1\xA1"),
        (1, b"\xF0\xA1\xA1\xA1"),
        (2, b"X\xF0\xA1\xA1\xA1"),
        (3, b"XX\xF0\xA1\xA1\xA1"),
        (-1, b"\xFC"),
        (-1, b"\xFC\xA1"),
        (-1, b"\xFC\xA1\xA1"),
        (-1, b"\xFC\xA1\xA1\xA1"),
        (-1, b"\xC2"),
        (-1, b"\xA1"),
        (-1, b"\xA1\xA1"),
        (-1, b"\xE1\xA1"),
        (-1, b"\xA1\xA1\xA1"),
        (-1, b"\xF0\xA1"),
        (-1, b"\xA1\xA1\xA1\xA1"),
    ];
    for (e, b) in cases {
        let actual = sk_utf8_count(b) as i64;
        eprintln!("CountUTF8 input={b:02x?} expected={e} actual={actual}");
        check!(r, *e as i64 == actual);
    }
    r.finish();
}
#[test]
fn SkUTF_NextUTF8_ToUTF8() {
    let mut r = Reporter::default();
    let cases: &[(i32, &[u8])] = &[
        (-1, b"\xFC"),
        (-1, b""),
        (0x0058, b"X"),
        (0x00A1, b"\xC2\xA1"),
        (0x1861, b"\xE1\xA1\xA1"),
        (0x010330, b"\xF0\x90\x8C\xB0"),
    ];
    for (expected, b) in cases {
        let mut p = 0;
        let uni = sk_utf8_next(b, &mut p);
        check!(r, p == b.len());
        check!(r, uni == *expected);
        let mut buff = [0u8; 5];
        let len = sk_utf8_encode(uni as u32, (&mut buff[..4]).try_into().unwrap());
        if buff[len] != 0 {
            check!(r, false);
            continue;
        }
        if uni == -1 {
            eprintln!("ToUTF8(-1) expected=0 actual={len}");
            check!(r, len == 0);
            continue;
        }
        if len == 0 {
            check!(r, false);
            continue;
        }
        if len > 4 {
            check!(r, false);
            continue;
        }
        let nul = buff.iter().position(|v| *v == 0).unwrap();
        check!(r, len == nul);
        check!(r, len == b.len());
        check!(r, &buff[..len] == *b);
    }
    r.finish();
}
