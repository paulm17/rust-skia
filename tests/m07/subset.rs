// Copyright 2015 Google Inc. BSD: tests/LICENSE.skia.
// SubsetPath.cpp/.h fixture utility. No M07 registration calls it directly.
use super::*;
pub struct SubsetPath<'a> {
    path: &'a SkPath,
    selected: Vec<bool>,
    subset: i32,
    tries: i32,
    contours: bool,
}
pub struct SubsetContours<'a>(SubsetPath<'a>);
pub struct SubsetVerbs<'a>(SubsetPath<'a>);
impl<'a> SubsetContours<'a> {
    pub fn new(path: &'a SkPath) -> Self {
        Self(SubsetPath::new(path, true))
    }
    pub fn subset(&mut self, failed: bool, out: &mut SkPath) -> bool {
        self.0.subset(failed, out)
    }
}
impl<'a> SubsetVerbs<'a> {
    pub fn new(path: &'a SkPath) -> Self {
        Self(SubsetPath::new(path, false))
    }
    pub fn subset(&mut self, failed: bool, out: &mut SkPath) -> bool {
        self.0.subset(failed, out)
    }
}
impl<'a> SubsetPath<'a> {
    fn new(path: &'a SkPath, contours: bool) -> Self {
        let mut found = false;
        let mut count = 0;
        let mut it = range(path);
        while let Some((v, _, _)) = it.next() {
            match v {
                M => {}
                L | Q | K | C => {
                    if contours {
                        found = true;
                    } else {
                        count += 1;
                    }
                }
                X => {
                    if contours {
                        count += 1;
                        found = false;
                    }
                }
                _ => {
                    debug_assert!(false, "bad verb");
                    break;
                }
            }
        }
        if contours {
            count += usize::from(found);
        }
        Self {
            path,
            selected: vec![true; count],
            subset: 1,
            tries: count as i32,
            contours,
        }
    }
    fn range(&self) -> (usize, usize) {
        let leading = (self.subset as u32).leading_zeros();
        let parts = 1i32 << (31 - leading);
        let part_index = self.subset - parts;
        debug_assert!(part_index >= 0);
        let count = self.selected.len() as i32;
        (
            (count * part_index / parts) as usize,
            (count * (part_index + 1) / parts) as usize,
        )
    }
    fn subset(&mut self, failed: bool, out: &mut SkPath) -> bool {
        if !failed {
            let (start, end) = self.range();
            for i in start..end {
                self.selected[i] = true;
            }
        }
        let (start, end) = loop {
            let range = loop {
                self.subset += 1;
                let (start, end) = self.range();
                if end - start > 1 {
                    self.tries = self.selected.len() as i32;
                } else if end - start == 1 {
                    self.tries -= 1;
                    if self.tries <= 0 {
                        return false;
                    }
                }
                if start != end {
                    break (start, end);
                }
            };
            if self.selected[range.0] {
                break range;
            }
        };
        for i in start..end {
            self.selected[i] = false;
        }
        eprint!("selected: ");
        for value in &self.selected {
            eprint!("{}", if *value { 'x' } else { '-' });
        }
        *out = self.get_subset_path();
        true
    }
    fn get_subset_path(&self) -> SkPath {
        let mut b = SkPathBuilder::new();
        b.set_fill_type(self.path.get_fill_type());
        if self.selected.is_empty() {
            return b.detach(None);
        }
        let mut index = 0;
        let mut enabled = self.selected[0];
        let mut add_move = true;
        let mut add_line = false;
        let mut it = range(self.path);
        while let Some((v, pts, w)) = it.next() {
            if !self.contours {
                enabled = if matches!(v, L | Q | K | C) {
                    let e = self.selected[index];
                    index += 1;
                    e
                } else {
                    false
                };
            }
            if enabled && add_move {
                b.move_to(pts[0]);
                add_move = false;
            } else if !self.contours && enabled && add_line {
                b.line_to(pts[0]);
                add_line = false;
            }
            match v {
                M => {}
                L => {
                    if enabled {
                        b.line_to(pts[1]);
                    }
                }
                Q => {
                    if enabled {
                        b.quad_to(pts[1], pts[2]);
                    }
                }
                K => {
                    if enabled {
                        b.conic_to(pts[1], pts[2], w);
                    }
                }
                C => {
                    if enabled {
                        b.cubic_to(pts[1], pts[2], pts[3]);
                    }
                }
                X => {
                    if self.contours {
                        if enabled {
                            b.close();
                        }
                        index += 1;
                        if index >= self.selected.len() {
                            continue;
                        }
                        enabled = self.selected[index];
                        add_move = true;
                        continue;
                    } else {
                        b.close();
                        add_move = true;
                        add_line = false;
                        continue;
                    }
                }
                _ => {
                    debug_assert!(false, "bad verb");
                    return b.detach(None);
                }
            }
            if !self.contours {
                add_line = !enabled;
            }
        }
        b.detach(None)
    }
}
