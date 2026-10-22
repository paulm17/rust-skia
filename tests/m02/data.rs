use super::*;
fn table_equal(r: &mut Reporter, a: &SkDataTable, b: &SkDataTable) {
    check!(r, a.count() == b.count());
    for i in 0..a.count() {
        check!(r, a.at_size(i) == b.at_size(i));
        check!(r, a.at_bytes(i) == b.at_bytes(i));
    }
}
#[test]
fn DataTable() {
    let mut r = Reporter::default();
    let t = [
        SkDataTable::make_empty(),
        SkDataTable::make_copy_arrays(&[], &[], 0),
        SkDataTable::make_copy_array(&[], 0, 0),
        SkDataTable::make_array_proc(&[], &[], 0, None),
    ];
    for a in &t {
        check!(r, a.is_empty());
        check!(r, a.count() == 0);
    }
    for a in &t[1..] {
        table_equal(&mut r, &t[0], a);
    }
    let ints = [1i32, 4, 9, 16, 25, 63];
    let bytes: Vec<u8> = ints.iter().flat_map(|v| v.to_ne_bytes()).collect();
    let t = SkDataTable::make_copy_array(&bytes, 4, ints.len());
    check!(r, t.count() == ints.len());
    for i in 0..ints.len() {
        let (p, s) = t.at(i);
        check!(r, t.at_size(i) == 4);
        check!(r, unsafe { p.cast::<i32>().read_unaligned() } == ints[i]);
        check!(r, s == 4);
    }
    let strings = [
        c"",
        c"a",
        c"be",
        c"see",
        c"deigh",
        c"ef",
        c"ggggggggggggggggggggggggggg",
    ];
    let ptrs: Vec<_> = strings.iter().map(|v| v.as_ptr().cast::<u8>()).collect();
    let sizes: Vec<_> = strings
        .iter()
        .map(|v| v.to_bytes_with_nul().len())
        .collect();
    let t = SkDataTable::make_copy_arrays(&ptrs, &sizes, strings.len());
    check!(r, t.count() == strings.len());
    for i in 0..strings.len() {
        let (p, s) = t.at(i);
        check!(r, t.at_size(i) == sizes[i]);
        check!(
            r,
            unsafe { std::ffi::CStr::from_ptr(p.cast()) } == strings[i]
        );
        check!(r, s == sizes[i]);
        check!(
            r,
            t.at_str(i).iter().position(|v| *v == 0).unwrap() == strings[i].to_bytes().len()
        );
    }
    let ints: Vec<i32> = (0..16).collect();
    let bytes: Vec<_> = ints.iter().flat_map(|v| v.to_ne_bytes()).collect();
    let ptrs: Vec<_> = (0..16)
        .map(|i| unsafe { bytes.as_ptr().add(i * 4) })
        .collect();
    let t = SkDataTable::make_array_proc(&ptrs, &[4; 16], 16, None);
    check!(r, t.count() == 16);
    for i in 0..16 {
        let (p, s) = t.at(i);
        check!(r, t.at_size(i) == 4);
        check!(r, unsafe { *p } == i as u8);
        check!(r, s == 4);
    }
    r.finish();
}
fn assert_len(r: &mut Reporter, d: &SkData, n: usize) {
    check!(r, d.size() == n);
}
fn assert_data(r: &mut Reporter, d: &SkData, b: &[u8]) {
    check!(r, d.size() == b.len());
    check!(r, d.bytes() == b);
}
#[test]
fn Data() {
    let mut r = Reporter::default();
    let s = b"We the people, in order to form a more perfect union.";
    let r0 = SkData::make_empty();
    let r1 = SkData::make_with_copy(s);
    let r2 = SkData::make_with_proc(vec![0; 10 * 4], || {});
    let r3 = r1.make_subset(7, 6);
    assert_len(&mut r, &r0, 0);
    assert_len(&mut r, &r1, s.len());
    assert_len(&mut r, &r2, 40);
    assert_len(&mut r, &r3, 6);
    assert_data(&mut r, &r1, s);
    assert_data(&mut r, &r3, b"people");
    assert_len(&mut r, &r1.make_subset(s.len(), 10), 0);
    assert_len(&mut r, &r1.make_subset(0, 0), 0);
    let a = SkData::make_with_copy(b"Hello world\0");
    let b = SkData::make_with_cstring(Some("Hello world"));
    check!(r, a.equals(&b));
    let c = SkData::make_with_cstring(None);
    check!(r, c.size() == 1);
    check!(r, c.bytes()[0] == 0);
    let path = std::env::temp_dir().join(format!("rust-skia-data-test-{}", std::process::id()));
    {
        use std::io::Write;
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"abcdefghijklmnopqrstuvwxyz").unwrap();
    }
    let file = std::fs::File::open(&path).unwrap();
    let d = SkData::make_from_file(&path);
    check!(r, d.is_some());
    let d = d.unwrap();
    check!(r, d.size() == 26);
    check!(r, d.bytes() == b"abcdefghijklmnopqrstuvwxyz");
    use std::os::fd::AsRawFd;
    let d = SkData::make_from_fd(file.as_raw_fd());
    check!(r, d.is_some());
    let d = d.unwrap();
    check!(r, d.size() == 26);
    check!(r, d.bytes() == b"abcdefghijklmnopqrstuvwxyz");
    std::fs::remove_file(path).unwrap();
    let d = SkData::make_without_copy(&[1, 2, 3, 4]);
    for (off, len) in [(5, 0), (4, 2), (0, 5), (3, 2)] {
        check!(r, d.share_subset(off, len).is_none());
        check!(r, d.copy_subset(off, len).is_none());
    }
    for (off, len) in [(0, 0), (2, 0), (4, 0)] {
        check!(r, d.share_subset(off, len).unwrap().empty());
        check!(r, d.copy_subset(off, len).unwrap().empty());
    }
    for (off, len) in [(0, 4), (2, 2), (0, 2), (1, 2)] {
        let s = &d.bytes()[off..off + len];
        let a = d.share_subset(off, len).unwrap();
        check!(r, a.size() == s.len() && a.data() == s.as_ptr());
        let b = d.copy_subset(off, len).unwrap();
        check!(r, b.size() != s.len() || b.data() != s.as_ptr());
        check!(r, b.bytes() == s);
    }
    let array: Vec<u8> = (0i32..10).flat_map(|i| i.to_ne_bytes()).collect();
    let d = SkData::make_with_copy(&array);
    check!(r, d.size() == 40);
    check!(r, d.bytes() == array);
    let d1 = d.share_subset(8, 16).unwrap();
    check!(r, d1.size() == 16);
    check!(r, d1.data() == unsafe { d.data().add(8) });
    let d2 = d.copy_subset(8, 16).unwrap();
    check!(r, d2.size() == 16);
    check!(r, d2.bytes() == &array[8..24]);
    check!(r, d == d);
    check!(r, d != d1);
    check!(r, d1 == d2);
    r.finish();
}
#[test]
fn Data_empty() {
    let mut r = Reporter::default();
    let array = [
        SkData::make_empty(),
        SkData::make_uninitialized(0),
        SkData::make_from_malloc(Vec::new()),
        SkData::make_with_copy(b""),
        SkData::make_with_proc(Vec::new(), || {}),
        SkData::make_without_copy(&[]),
    ];
    for a in &array {
        check!(r, a.size() == 0);
        for b in &array {
            check!(r, a.equals(b));
        }
    }
    r.finish();
}
