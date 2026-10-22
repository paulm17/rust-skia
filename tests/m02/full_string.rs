use super::*;
#[test]
fn String() {
    let mut r = Reporter::default();
    let mut a = SkString::new();
    let mut b = SkString::with_capacity(0);
    let mut c = SkString::from_str("");
    let mut d = SkString::from_bytes(&[]);
    let mut v = SkString::from_str("");
    check!(r, a.is_empty());
    check!(r, a == b && a == c && a == d && a == v);
    a.set_str("hello");
    b.set(b"hellox".get(..5).unwrap());
    c.set(a.bytes());
    d.resize(5);
    unsafe {
        std::ptr::copy_nonoverlapping(b"helloz".as_ptr(), d.data(), 5);
    }
    v.set(&b"hellooooo"[..5]);

    check!(r, !a.is_empty());
    check!(r, a.size() == 5);
    check!(r, a == b && a == c && a == d && a == v);
    check!(r, a.equals_bytes(&b"hello"[..5]));
    check!(r, a.equals("hello"));
    check!(r, !a.equals("help"));

    check!(r, a.starts_with("hell"));
    check!(r, a.starts_with("h"));
    check!(r, !a.starts_with("ell"));
    check!(r, !a.starts_with("e"));
    check!(r, a.starts_with(""));
    check!(r, a.ends_with("llo"));
    check!(r, a.ends_with("o"));
    check!(r, !a.ends_with("ll"));
    check!(r, !a.ends_with("l"));
    check!(r, a.ends_with(""));
    check!(r, a.contains("he"));
    check!(r, a.contains("ll"));
    check!(r, a.contains("lo"));
    check!(r, a.contains("hello"));
    check!(r, !a.contains("hellohello"));
    check!(r, a.contains(""));
    check!(r, a.contains_byte(b'e'));
    check!(r, !a.contains_byte(b'z'));

    v.prepend(b"[[");
    v.append(b"]]");
    check!(r, v.equals("[[hello]]"));

    v.insert(2, &b"?!"[..1]);
    check!(r, v.equals("[[?hello]]"));

    let mut e = a.clone();
    let mut f = SkString::from_str("hello");
    let g = SkString::from_bytes(&b"helloz"[..5]);

    check!(r, a == e && a == f && a == g);

    b.set_str("world");
    c = b.clone();
    check!(r, a != b && a != c && b == c);

    a.append_str(" world");
    e.append(&b"worldz"[..5]);
    e.insert(5, b" ");
    f.set_str("world");
    f.prepend_str("hello ");
    check!(r, a.equals("hello world") && a == e && a == f);

    a.reset();
    b.resize(0);
    check!(r, a.is_empty() && b.is_empty() && a == b);

    a.set_str("a");
    a.set_str("ab");
    a.set_str("abc");
    a.set_str("abcd");

    a.set_str("");
    a.append_s32(0x7FFFFFFFu32 as i32);
    check!(r, a.equals("2147483647"));
    a.set_str("");
    a.append_s32(0x80000001u32 as i32);
    check!(r, a.equals("-2147483647"));
    a.set_str("");
    a.append_s32(0x80000000u32 as i32);
    check!(r, a.equals("-2147483648"));

    a.set_str("");
    a.append_u32(0x7FFFFFFFu32);
    check!(r, a.equals("2147483647"));
    a.set_str("");
    a.append_u32(0x80000001u32);
    check!(r, a.equals("2147483649"));
    a.set_str("");
    a.append_u32(0xFFFFFFFFu32);
    check!(r, a.equals("4294967295"));

    a.set_str("");
    a.append_s64(0x7FFFFFFFFFFFFFFFu64 as i64, 0);
    check!(r, a.equals("9223372036854775807"));
    a.set_str("");
    a.append_s64(0x8000000000000001u64 as i64, 0);
    check!(r, a.equals("-9223372036854775807"));
    a.set_str("");
    a.append_s64(0x8000000000000000u64 as i64, 0);
    check!(r, a.equals("-9223372036854775808"));
    a.set_str("");
    a.append_s64(0x0000000001000000u64 as i64, 15);
    check!(r, a.equals("000000016777216"));
    a.set_str("");
    a.append_s64(0xFFFFFFFFFF000000u64 as i64, 15);
    check!(r, a.equals("-000000016777216"));

    a.set_str("");
    a.append_u64(0x7FFFFFFFFFFFFFFFu64, 0);
    check!(r, a.equals("9223372036854775807"));
    a.set_str("");
    a.append_u64(0x8000000000000001u64, 0);
    check!(r, a.equals("9223372036854775809"));
    a.set_str("");
    a.append_u64(0xFFFFFFFFFFFFFFFFu64, 0);
    check!(r, a.equals("18446744073709551615"));
    a.set_str("");
    a.append_u64(0x0000000001000000u64, 15);
    check!(r, a.equals("000000016777216"));

    a.printf("%i", &[SkFormatArg::S32(0)]);
    check!(r, a.equals("0"));
    a.printf("%g", &[SkFormatArg::F64(3.14)]);
    check!(r, a.equals("3.14"));
    a.printf("hello %s", &[SkFormatArg::Str("skia".into())]);
    check!(r, a.equals("hello skia"));

    let records: [(f32, &str); 10] = [
        (0.0, "0"),
        (1.0, "1"),
        (-1.0, "-1"),
        (0.5, "0.5"),
        (f32::INFINITY, "inf"),
        (-f32::INFINITY, "-inf"),
        (f32::NAN, "nan"),
        (-f32::NAN, "nan"),
        (3.4028234e38, "3.4028235e+38"),
        (-3.4028234e38, "-3.4028235e+38"),
    ];
    for i in 0..records.len() {
        a.reset();
        a.append_scalar(records[i].0);
        check!(r, a.size() <= 15);
        if !a.equals(records[i].1) {
            eprintln!("received <{}> expected <{}>", a.as_str(), records[i].1);
            check!(r, false);
        }
    }

    check!(r, SkStringPrintf("%i", &[SkFormatArg::S32(0)]).equals("0"));

    r.finish();
}
