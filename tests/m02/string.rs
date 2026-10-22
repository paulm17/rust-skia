use super::*;
fn spaces(r: &mut Reporter, s: &SkString) {
    check!(r, s.size() == 2000);
    for i in 0..s.size() {
        check!(r, s.bytes()[i] == b' ');
    }
}
fn set_byte(s: &mut SkString, i: usize, b: u8) {
    unsafe {
        *s.data().add(i) = b;
    }
}
#[test]
fn String_overflow() {
    let mut r = Reporter::default();
    let args = [SkFormatArg::Str(" ".into())];
    let mut a = SkStringPrintf("%2000s", &args);
    spaces(&mut r, &a);
    a.set_str("X");
    a.printf("%2000s", &args);
    spaces(&mut r, &a);
    a.set_str("X");
    a.appendf("%1999s", &args);
    check!(r, a.bytes()[0] == b'X');
    set_byte(&mut a, 0, b' ');
    spaces(&mut r, &a);
    a.set_str("X");
    a.prependf("%1999s", &args);
    check!(r, a.bytes()[1999] == b'X');
    set_byte(&mut a, 1999, b' ');
    spaces(&mut r, &a);
    r.finish();
}
#[test]
fn String_Threaded() {
    let s = SkString::from_str("foo");
    std::thread::scope(|scope| {
        let threads: Vec<_> = (0..5)
            .map(|_| {
                scope.spawn(|| {
                    let copy = s.clone();
                    let _ = copy.equals("test");
                })
            })
            .collect();
        for t in threads {
            t.join().unwrap();
        }
    });
}
#[test]
fn String_huge() {
    // Original explicitly disables its manual crash loop.
    let mut size = u32::MAX as usize - 16;
    if false {
        loop {
            let _s = SkString::with_capacity(size);
            size += 1;
        }
    }
}
#[test]
fn String_VAList() {
    let mut r = Reporter::default();
    let args = [
        SkFormatArg::Str("hello".into()),
        SkFormatArg::Char('w'),
        SkFormatArg::Char('o'),
        SkFormatArg::Char('r'),
        SkFormatArg::Char('l'),
        SkFormatArg::Char('d'),
    ];
    let fmt = "%s %c%c%c%c%c";
    let mut s = SkString::from_str("123");
    s.print_valist(fmt, &args);
    check!(r, s.equals("hello world"));
    let mut s = SkString::from_str("123");
    s.append_valist(fmt, &args);
    check!(r, s.equals("123hello world"));
    let mut s = SkString::from_str("123");
    s.prepend_valist(fmt, &args);
    check!(r, s.equals("hello world123"));
    r.finish();
}
#[test]
fn String_VAList_overflow() {
    let mut r = Reporter::default();
    let args = [SkFormatArg::Str(" ".into())];
    let mut s = SkString::from_str("X");
    s.print_valist("%2000s", &args);
    spaces(&mut r, &s);
    let mut s = SkString::from_str("X");
    s.append_valist("%1999s", &args);
    check!(r, s.bytes()[0] == b'X');
    set_byte(&mut s, 0, b' ');
    spaces(&mut r, &s);
    let mut s = SkString::from_str("X");
    s.prepend_valist("%1999s", &args);
    check!(r, s.bytes()[1999] == b'X');
    set_byte(&mut s, 1999, b' ');
    spaces(&mut r, &s);
    r.finish();
}
fn c_equals(s: &SkString, v: &[u8]) -> bool {
    unsafe { std::ffi::CStr::from_ptr(s.c_str().cast()) }.to_bytes() == v
}
#[test]
fn String_resize_to_nothing() {
    let mut r = Reporter::default();
    let mut s = SkString::from_str("hello world!");
    check!(r, s.equals("hello world!"));
    s.resize(0);
    check!(r, s.equals(""));
    r.finish();
}
#[test]
fn String_resize_shrink() {
    let mut r = Reporter::default();
    let mut s = SkString::from_str("hello world!");
    check!(r, s.equals("hello world!"));
    s.resize(5);
    check!(r, s.equals("hello"));
    r.finish();
}
#[test]
fn String_resize_grow() {
    let mut r = Reporter::default();
    let mut s = SkString::from_str("hello world!");
    check!(r, s.equals("hello world!"));
    s.resize(25);
    check!(r, c_equals(&s, b"hello world!"));
    check!(r, s.size() == 25);
    r.finish();
}
#[test]
fn String_resize_after_assignment() {
    let mut r = Reporter::default();
    let mut s = SkString::from_str("hello world!");
    let _t = s.clone();
    check!(r, s.equals("hello world!"));
    s.resize(25);
    check!(r, c_equals(&s, b"hello world!"));
    check!(r, s.size() == 25);
    s.resize(5);
    check!(r, s.equals("hello"));
    r.finish();
}
fn resize_helper(r: &mut Reporter, mut s: SkString) {
    check!(r, s.equals("hello world!"));
    s.resize(5);
    check!(r, s.equals("hello"));
    s.resize(25);
    check!(r, c_equals(&s, b"hello"));
    check!(r, s.size() == 25);
}
#[test]
fn String_resize_after_copy_construction() {
    let mut r = Reporter::default();
    let s = SkString::from_str("hello world!");
    resize_helper(&mut r, s.clone());
    r.finish();
}
#[test]
fn String_SkStrSplit() {
    let mut r = Reporter::default();
    let mut results: Vec<SkString>;

    results = sk_str_split("a-_b_c-dee--f-_-_-g-", "-_", SkStrSplitMode::Coalesce);
    check!(r, results.len() == 6);
    check!(r, results[0].equals("a"));
    check!(r, results[1].equals("b"));
    check!(r, results[2].equals("c"));
    check!(r, results[3].equals("dee"));
    check!(r, results[4].equals("f"));
    check!(r, results[5].equals("g"));

    results.clear();
    results = sk_str_split("\n", "\n", SkStrSplitMode::Coalesce);
    check!(r, results.len() == 0);

    results.clear();
    results = sk_str_split("", "\n", SkStrSplitMode::Coalesce);
    check!(r, results.len() == 0);

    results.clear();
    results = sk_str_split("a", "\n", SkStrSplitMode::Coalesce);
    check!(r, results.len() == 1);
    check!(r, results[0].equals("a"));
    r.finish();
}
#[test]
fn String_SkStrSplit_All() {
    let mut r = Reporter::default();
    let mut results: Vec<SkString>;
    results = sk_str_split("a-_b_c-dee--f-_-_-g-", "-_", SkStrSplitMode::Strict);
    check!(r, results.len() == 13);
    check!(r, results[0].equals("a"));
    check!(r, results[1].equals(""));
    check!(r, results[2].equals("b"));
    check!(r, results[3].equals("c"));
    check!(r, results[4].equals("dee"));
    check!(r, results[5].equals(""));
    check!(r, results[6].equals("f"));
    check!(r, results[7].equals(""));
    check!(r, results[8].equals(""));
    check!(r, results[9].equals(""));
    check!(r, results[10].equals(""));
    check!(r, results[11].equals("g"));
    check!(r, results[12].equals(""));

    results.clear();
    results = sk_str_split("\n", "\n", SkStrSplitMode::Strict);
    check!(r, results.len() == 2);
    check!(r, results[0].equals(""));
    check!(r, results[1].equals(""));

    results.clear();
    results = sk_str_split("", "\n", SkStrSplitMode::Strict);
    check!(r, results.len() == 0);

    results.clear();
    results = sk_str_split("a", "\n", SkStrSplitMode::Strict);
    check!(r, results.len() == 1);
    check!(r, results[0].equals("a"));

    results.clear();
    results = sk_str_split(",,", ",", SkStrSplitMode::Strict);
    check!(r, results.len() == 3);
    check!(r, results[0].equals(""));
    check!(r, results[1].equals(""));
    check!(r, results[2].equals(""));

    results.clear();
    results = sk_str_split(",a,b,", ",", SkStrSplitMode::Strict);
    check!(r, results.len() == 4);
    check!(r, results[0].equals(""));
    check!(r, results[1].equals("a"));
    check!(r, results[2].equals("b"));
    check!(r, results[3].equals(""));
    r.finish();
}
#[test]
fn String_fromUTF16() {
    let mut r = Reporter::default();
    // test data produced with `iconv`.
    let test1 = [
        0xD835, 0xDCD0, 0xD835, 0xDCD1, 0xD835, 0xDCD2, 0xD835, 0xDCD3, 0xD835, 0xDCD4, 0x0020,
        0xD835, 0xDCD5, 0xD835, 0xDCD6, 0xD835, 0xDCD7, 0xD835, 0xDCD8, 0xD835, 0xDCD9,
    ];
    check!(r, sk_string_from_utf16(&test1).equals("𝓐𝓑𝓒𝓓𝓔 𝓕𝓖𝓗𝓘𝓙"));

    let test2 = [
        0x0041, 0x0042, 0x0043, 0x0044, 0x0045, 0x0020, 0x0046, 0x0047, 0x0048, 0x0049, 0x004A,
    ];
    check!(r, sk_string_from_utf16(&test2).equals("ABCDE FGHIJ"));

    let test3 = [
        0x03B1, 0x03B2, 0x03B3, 0x03B4, 0x03B5, 0x0020, 0x03B6, 0x03B7, 0x03B8, 0x03B9, 0x03BA,
    ];
    check!(r, sk_string_from_utf16(&test3).equals("αβγδε ζηθικ"));
    r.finish();
}
