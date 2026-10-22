// Copyright 2011, 2013 Google Inc. BSD: tests/LICENSE.skia.
use super::*;
use rust_skia::os_file::SkOSPath;
fn test_to_from(r: &mut Reporter, path: &SkPath) {
    let s = SkParsePath::to_svg_string(path, PathEncoding::Absolute);
    let path2 = SkParsePath::from_svg_string(&s.as_str());
    check!(r, path2.is_some());
    let s2 = SkParsePath::to_svg_string(&path2.unwrap(), PathEncoding::Absolute);
    check!(r, s == s2);
    // Original #if 0 path-equality assertion remains disabled: closing edges are explicit.
}
#[test]
fn ParsePath() {
    let mut r = Reporter::default();
    for (s, bounds) in [
        (
            "M1,1 l-2.58-2.828-3.82-0.113, 1.9-3.3223-1.08-3.6702, 3.75,0.7744,3.16-2.1551,0.42,3.8008,3.02,2.3384-3.48,1.574-1.29,3.601z",
            rect(-5.39999962, -10.3142, 5.77000046, 1.0),
        ),
        ("", rect(0.0, 0.0, 0.0, 0.0)),
        ("M0,0L10,10", rect(0.0, 0.0, 10.0, 10.0)),
        ("M-5.5,-0.5 Q 0 0 6,6.50", rect(-5.5, -0.5, 6.0, 6.5)),
    ] {
        let path = SkParsePath::from_svg_string(s);
        check!(r, path.is_some());
        let path = path.unwrap();
        check!(r, bounds == *path.get_bounds());
        test_to_from(&mut r, &path);
    }
    let bounds = rect(0.0, 0.0, 10.0, 10.5);
    let mut b = SkPathBuilder::new();
    b.add_rect(&bounds, Cw, 0);
    test_to_from(&mut r, &b.snapshot(None));
    b.add_oval(&bounds, Cw, 1);
    test_to_from(&mut r, &b.snapshot(None));
    b.add_rrect(&RRect::make_rect_xy(&bounds, 4.0, 4.5), Cw, 6);
    test_to_from(&mut r, &b.snapshot(None));
    r.finish();
}
fn testInvalidPath(r: &mut Reporter, name: &str, input: &str) {
    eprintln!("{name}");
    check!(r, SkParsePath::from_svg_string(input).is_none());
}
#[test]
fn ParsePath_InvalidDoesNotCrash() {
    let mut r = Reporter::default();
    for (name, input) in [
        ("empty move", "M"),
        ("partial move", "M 5"),
        ("partial vertical line", "V"),
        ("partial horizontal line", "H"),
        ("partial cubic", "C 1 2"),
        ("partial continued cubic", "S 6 7"),
        ("partial quad", "Q 3 4 5"),
        ("partial continued quad", "T"),
        ("partial arc", "A 1 2 3 4 5 6"),
        ("partial ~", "~ 7 6 5"),
    ] {
        testInvalidPath(&mut r, name, input);
    }
    r.finish();
}
#[test]
fn ParsePathOptionalCommand() {
    let mut r = Reporter::default();
    for (s, v, n) in [
        ("", 0, 0),
        ("H100 200 ", 3, 3),
        ("H-100-200", 3, 3),
        ("H+100+200", 3, 3),
        ("H.10.20", 3, 3),
        ("H-.10-.20", 3, 3),
        ("H+.10+.20", 3, 3),
        ("L100 100 200 200", 3, 3),
        ("L-100-100-200-200", 3, 3),
        ("L+100+100+200+200", 3, 3),
        ("L.10.10.20.20", 3, 3),
        ("L-.10-.10-.20-.20", 3, 3),
        ("L+.10+.10+.20+.20", 3, 3),
        ("C100 100 200 200 300 300 400 400 500 500 600 600", 3, 7),
        ("C100-100-200-200-300-300-400-400-500-500-600-600", 3, 7),
        ("C100+100+200+200+300+300+400+400+500+500+600+600", 3, 7),
        ("C.10.10.20.20.30.30.40.40.50.50.60.60", 3, 7),
        ("C-.10-.10-.20-.20-.30-.30-.40-.40-.50-.50-.60-.60", 3, 7),
        ("C+.10+.10+.20+.20+.30+.30+.40+.40+.50+.50+.60+.60", 3, 7),
        ("c-1.49.71-2.12 2.5-1.4 4 .71 1.49 2.5 2.12 4 1.4z", 4, 7),
    ] {
        let path = SkParsePath::from_svg_string(s);
        check!(r, path.is_some());
        let path = path.unwrap();
        check!(r, path.count_verbs() == v);
        check!(r, path.count_points() == n);
    }
    r.finish();
}
#[test]
fn ParsePathArcFlags() {
    let mut r = Reporter::default();
    let path =
        SkParsePath::from_svg_string("M10 10a2.143 2.143 0 100-4.285 2.143 2.143 0 000 4.286");
    check!(r, path.is_some());
    let path = path.unwrap();
    check!(r, path.count_verbs() == 5);
    check!(r, path.count_points() == 9);
    r.finish();
}
fn test_dir_with_file(r: &mut Reporter, dir: &str, filename: &str) {
    debug_assert!(!filename.contains('/'));
    let full = SkOSPath::join(Some(dir), Some(filename));
    let expected = dir.len() + filename.len() + usize::from(!dir.ends_with('/') && !dir.is_empty());
    check!(r, full.len() == expected);
    let mut base = SkOSPath::basename(Some(&full.as_str()));
    let dirname = SkOSPath::dirname(Some(&full.as_str()));
    check!(r, base.as_str() == filename);
    let mut stripped = dir.to_string();
    while stripped.len() > 2 && stripped.ends_with('/') {
        stripped.pop();
    }
    if dirname.as_str() != stripped {
        eprintln!("OOUCH {dir} {stripped} {}", dirname.as_str());
    }
    check!(r, dirname.as_str() == stripped);
    check!(r, !base.as_str().contains('/'));
    base = SkOSPath::basename(Some(filename));
    check!(r, base.as_str() == filename);
}
#[test]
fn OSPath() {
    let mut r = Reporter::default();
    let mut dir = "dir".to_string();
    let mut filename = "file".to_string();
    test_dir_with_file(&mut r, &dir, &filename);
    dir.push('/');
    test_dir_with_file(&mut r, &dir, &filename);
    test_dir_with_file(&mut r, &dir, "");
    test_dir_with_file(&mut r, "", &filename);
    dir.push_str("subDir");
    test_dir_with_file(&mut r, &dir, &filename);
    dir.push('/');
    check!(r, SkOSPath::basename(Some(&dir)).len() == 0);
    check!(r, SkOSPath::basename(None).len() == 0);
    dir = "/".into();
    filename = "file".into();
    test_dir_with_file(&mut r, &dir, &filename);
    filename.clear();
    test_dir_with_file(&mut r, &dir, &filename);
    check!(r, SkOSPath::join(None, None).is_empty());
    r.finish();
}
