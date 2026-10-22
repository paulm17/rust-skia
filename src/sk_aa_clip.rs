//! Port of `src/core/SkAAClip.h` and `src/core/SkAAClip.cpp`.
//!
//! The reference allocates a `RunHead` with the `YOffset` table and the packed run data inline
//! behind a ref-count. The port keeps the identical run encoding (rows of `[count, alpha]` pairs,
//! `YOffset` rows relative to the top) in an `Arc<RunHead>` so copy-on-write sharing and the
//! `trim*` behaviors are preserved.
//!
//! Deferred branches, each with a named gate:
//! - `SkAAClipBlitter` (`blitH`, `blitAntiH`, `blitV`, `blitRect`, `blitMask`) needs `SkBlitter`
//!   and is part of the raster pipeline → **M14**.
//! - `copyToMask` needs `src/core/SkMask.h` (`SkMaskBuilder`) → **M14**.
//!
//! `setPath(..., doAA=true)` is real 4×4 supersampled coverage (Skia's AA granularity is 1/4
//! pixel). It is produced by scan-converting the path scaled by 4 into a region and accumulating
//! 16 subpixel samples per pixel. Exact per-pixel parity with `SkScan::AntiFillPath` is asserted by
//! `AAClipTest` at **M14**; see the handoff.

use std::sync::Arc;

use crate::math::sk_mul_div255_round;
use crate::matrix::Matrix;
use crate::rect::{IRect, Rect};
use crate::scalar::Scalar;
use crate::sk_clip_op::SkClipOp;
use crate::sk_geometry::SkPathDirection;
use crate::sk_path::SkPath;
use crate::sk_path_builder::SkPathBuilder;
use crate::sk_path_raw::SkPathRaw;
use crate::sk_path_types::{SkPathConvexity, SkResolveConvexity};
use crate::sk_region::{BoundsType, Iterator as RegionIterator, SkRegion, SkRegionOp};
use crate::sk_region_path::{SkScanSpanSink, sk_scan_fill_path};

const K_MAX_INT32: i32 = 0x7FFF_FFFF;

/// The AA clip's supersampling shift (matches `SK_SUPERSAMPLE_SHIFT`).
pub const SK_SUPERSAMPLE_SHIFT: i32 = 2;
/// One dimension of the supersampling grid (`1 << SK_SUPERSAMPLE_SHIFT`).
pub const SK_SUPERSAMPLE_SCALE: i32 = 1 << SK_SUPERSAMPLE_SHIFT;

/// `YOffset`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct YOffset {
    y: i32,
    offset: u32,
}

/// `SkAAClip::RunHead`.
#[derive(Clone, Debug, Default)]
struct RunHead {
    row_count: i32,
    data: Vec<u8>,
}

impl RunHead {
    fn compute_row_size_for_width(width: i32) -> usize {
        // 2 bytes per segment, where each segment can store up to 255 for count
        let mut segments = 0usize;
        let mut width = width;
        while width > 0 {
            segments += 1;
            width -= width.min(255);
        }
        segments * 2
    }

    fn alloc_rect(bounds: &IRect) -> RunHead {
        debug_assert!(!bounds.is_empty());
        let mut width = bounds.width();
        let mut data = Vec::with_capacity(Self::compute_row_size_for_width(width));
        while width > 0 {
            let n = width.min(255);
            data.push(n as u8);
            data.push(0xFF);
            width -= n;
        }
        RunHead { row_count: 1, data }
    }
}

/// `RowIter`.
struct RowIter<'a> {
    row: Option<&'a [u8]>,
    pos: usize,
    left: i32,
    right: i32,
    bounds_right: i32,
    done: bool,
    alpha: u8,
}

impl<'a> RowIter<'a> {
    fn new(row: Option<&'a [u8]>, bounds: &IRect) -> RowIter<'a> {
        let mut iter = RowIter {
            row,
            pos: 0,
            left: bounds.left,
            right: K_MAX_INT32,
            bounds_right: bounds.right,
            done: true,
            alpha: 0,
        };
        if let Some(row) = row {
            iter.right = bounds.left + row[0] as i32;
            debug_assert!(iter.right <= iter.bounds_right);
            iter.alpha = row[1];
            iter.done = false;
        }
        iter
    }

    fn done(&self) -> bool {
        self.done
    }

    fn left(&self) -> i32 {
        self.left
    }

    fn right(&self) -> i32 {
        self.right
    }

    fn alpha(&self) -> u8 {
        self.alpha
    }

    fn next(&mut self) {
        if !self.done {
            self.left = self.right;
            if self.right == self.bounds_right {
                self.done = true;
                self.right = K_MAX_INT32;
                self.alpha = 0;
            } else {
                self.pos += 2;
                let row = self.row.unwrap();
                self.right += row[self.pos] as i32;
                self.alpha = row[self.pos + 1];
                debug_assert!(self.right <= self.bounds_right);
            }
        }
    }
}

/// A run of `[count, alpha]` pairs for a contiguous row range, after Y compression.
#[derive(Clone, Debug)]
pub struct RowSpan<'a> {
    pub top: i32,
    pub bottom: i32,
    pub data: &'a [u8],
}

/// `Iter`: iterates the compressed row data.
struct AaIter<'a> {
    data: &'a [u8],
    offsets: &'a [YOffset],
    curr: usize,
    stop: usize,
    data_pos: usize,
    top: i32,
    bottom: i32,
    done: bool,
}

impl<'a> AaIter<'a> {
    fn new(clip: &'a SkAAClip) -> AaIter<'a> {
        match &clip.run_head {
            None => AaIter {
                data: &[],
                offsets: &[],
                curr: 0,
                stop: 0,
                data_pos: 0,
                top: K_MAX_INT32,
                bottom: K_MAX_INT32,
                done: true,
            },
            Some(head) => {
                let y = clip.bounds.top;
                let start = head.offsets[0];
                AaIter {
                    data: &head.head.data,
                    offsets: &head.offsets,
                    curr: 0,
                    stop: head.head.row_count as usize,
                    data_pos: start.offset as usize,
                    top: y,
                    bottom: y + start.y + 1,
                    done: false,
                }
            }
        }
    }

    fn done(&self) -> bool {
        self.done
    }

    fn top(&self) -> i32 {
        self.top
    }

    fn bottom(&self) -> i32 {
        self.bottom
    }

    fn data(&self) -> Option<&'a [u8]> {
        if self.done {
            None
        } else {
            Some(&self.data[self.data_pos..])
        }
    }

    fn next(&mut self) {
        if !self.done {
            let prev = self.offsets[self.curr];
            let curr = self.curr + 1;
            debug_assert!(curr <= self.stop);

            self.top = self.bottom;
            if curr >= self.stop {
                self.done = true;
                self.bottom = K_MAX_INT32;
                self.data_pos = self.data.len();
            } else {
                let next = self.offsets[curr];
                self.bottom += next.y - prev.y;
                self.data_pos += (next.offset - prev.offset) as usize;
                self.curr = curr;
            }
        }
    }
}

/// `SkAAClip::Builder`.
struct Builder {
    bounds: IRect,
    rows: Vec<BuilderRow>,
    curr_row: Option<usize>,
    prev_y: i32,
    width: i32,
    min_y: i32,
}

#[derive(Clone, Debug)]
struct BuilderRow {
    y: i32,
    width: i32,
    data: Vec<u8>,
}

impl Builder {
    fn new(bounds: IRect) -> Builder {
        let width = bounds.width();
        Builder {
            bounds,
            rows: Vec::new(),
            curr_row: None,
            prev_y: -1,
            width,
            min_y: bounds.top,
        }
    }

    fn add_run(&mut self, x: i32, y: i32, alpha: u8, count: i32) {
        debug_assert!(count > 0);
        debug_assert!(self.bounds.contains(x, y));
        debug_assert!(self.bounds.contains(x + count - 1, y));

        let x = x - self.bounds.left;
        let y = y - self.bounds.top;

        if y != self.prev_y {
            debug_assert!(y > self.prev_y);
            self.prev_y = y;
            let row = self.flush_row(true).unwrap();
            self.rows[row].y = y;
            self.rows[row].width = 0;
            debug_assert!(self.rows[row].data.is_empty());
            self.curr_row = Some(row);
        }

        let row = self.curr_row.unwrap();
        debug_assert!(self.rows[row].width <= x);
        debug_assert!(self.rows[row].width < self.bounds.width());

        let gap = x - self.rows[row].width;
        if gap != 0 {
            append_run(&mut self.rows[row].data, 0, gap);
            self.rows[row].width += gap;
            debug_assert!(self.rows[row].width < self.bounds.width());
        }

        append_run(&mut self.rows[row].data, alpha, count);
        self.rows[row].width += count;
        debug_assert!(self.rows[row].width <= self.bounds.width());
    }

    // Reached by `SkAAClipBlitter::blitV` (M14).
    #[allow(dead_code)]
    fn add_column(&mut self, x: i32, y: i32, alpha: u8, height: i32) {
        debug_assert!(self.bounds.contains(x, y + height - 1));

        self.add_run(x, y, alpha, 1);
        let row = self.curr_row.unwrap();
        self.flush_row_h(row);
        let y = y - self.bounds.top;
        debug_assert_eq!(y, self.rows[row].y);
        self.rows[row].y = y + height - 1;
    }

    fn add_rect_run(&mut self, x: i32, y: i32, width: i32, height: i32) {
        debug_assert!(self.bounds.contains(x + width - 1, y + height - 1));
        self.add_run(x, y, 0xFF, width);

        // we assume the rect must be all we'll see for these scanlines
        // so we ensure our row goes all the way to our right
        let row = self.curr_row.unwrap();
        self.flush_row_h(row);

        let y = y - self.bounds.top;
        debug_assert_eq!(y, self.rows[row].y);
        self.rows[row].y = y + height - 1;
    }

    // Reached by `SkAAClipBlitter::blitAntiRect` (M14).
    #[allow(dead_code)]
    fn add_anti_rect_run(
        &mut self,
        x: i32,
        y: i32,
        mut width: i32,
        height: i32,
        left_alpha: u8,
        right_alpha: u8,
    ) {
        // According to SkBlitter.cpp, no matter whether leftAlpha is 0 or positive,
        // we should always consider [x, x+1] as the left-most column and [x+1, x+1+width]
        // as the rect with full alpha.
        debug_assert!(self.bounds.contains(
            x + width + if right_alpha > 0 { 1 } else { 0 },
            y + height - 1
        ));
        debug_assert!(width >= 0);

        // Conceptually we're always adding 3 runs, but we should
        // merge or omit them if possible.
        let mut x = x;
        if left_alpha == 0xFF {
            width += 1;
        } else if left_alpha > 0 {
            self.add_run(x, y, left_alpha, 1);
            x += 1;
        } else {
            // leftAlpha is 0, ignore the left column
            x += 1;
        }
        if right_alpha == 0xFF {
            width += 1;
        }
        if width > 0 {
            self.add_run(x, y, 0xFF, width);
        }
        if right_alpha > 0 && right_alpha < 255 {
            self.add_run(x + width, y, right_alpha, 1);
        }

        // if we never called addRun, we might not have a fCurrRow yet
        if let Some(row) = self.curr_row {
            // we assume the rect must be all we'll see for these scanlines
            // so we ensure our row goes all the way to our right
            self.flush_row_h(row);

            let y = y - self.bounds.top;
            debug_assert_eq!(y, self.rows[row].y);
            self.rows[row].y = y + height - 1;
        }
    }

    fn finish(&mut self, target: &mut SkAAClip) -> bool {
        self.flush_row(false);

        let mut data_size = 0usize;
        for row in &self.rows {
            data_size += row.data.len();
        }

        if 0 == data_size {
            return target.set_empty();
        }

        debug_assert!(self.min_y >= self.bounds.top);
        debug_assert!(self.min_y < self.bounds.bottom);
        let adjust_y = self.min_y - self.bounds.top;
        let mut bounds = self.bounds;
        bounds.top = self.min_y;

        let mut head = RunHead {
            row_count: self.rows.len() as i32,
            data: Vec::with_capacity(data_size),
        };
        let mut offsets = Vec::with_capacity(self.rows.len());
        let mut prev_y = self.rows[0].y - 1;
        for row in &self.rows {
            debug_assert!(prev_y < row.y); // must be monotonic
            prev_y = row.y;

            offsets.push(YOffset {
                y: row.y - adjust_y,
                offset: head.data.len() as u32,
            });
            head.data.extend_from_slice(&row.data);
            debug_assert_eq!(
                compute_row_length(
                    &head.data[offsets.last().unwrap().offset as usize..],
                    bounds.width()
                ),
                row.data.len()
            );
        }

        target.free_runs();
        target.bounds = bounds;
        target.run_head = Some(Arc::new(AaRunHead { head, offsets }));
        target.trim_bounds()
    }

    fn flush_row_h(&mut self, row: usize) {
        let width = self.width;
        let missing = width - self.rows[row].width;
        if missing > 0 {
            append_run(&mut self.rows[row].data, 0, missing);
            self.rows[row].width = width;
        }
    }

    fn flush_row(&mut self, ready_for_another: bool) -> Option<usize> {
        let mut next = None;
        let count = self.rows.len();
        if count > 0 {
            self.flush_row_h(count - 1);
        }
        if count > 1 {
            // are our last two runs the same?
            let prev = count - 2;
            let curr = count - 1;
            debug_assert_eq!(self.rows[prev].width, self.width);
            debug_assert_eq!(self.rows[curr].width, self.width);
            if self.rows[prev].data == self.rows[curr].data {
                self.rows[prev].y = self.rows[curr].y;
                if ready_for_another {
                    self.rows[curr].data.clear();
                    next = Some(curr);
                } else {
                    self.rows.remove(curr);
                }
            } else if ready_for_another {
                self.rows.push(BuilderRow {
                    y: 0,
                    width: 0,
                    data: Vec::new(),
                });
                next = Some(self.rows.len() - 1);
            }
        } else if ready_for_another {
            self.rows.push(BuilderRow {
                y: 0,
                width: 0,
                data: Vec::new(),
            });
            next = Some(self.rows.len() - 1);
        }
        next
    }

    fn blit_path(&mut self, target: &mut SkAAClip, path: &SkPath, do_aa: bool) -> bool {
        if do_aa {
            self.blit_path_aa(path);
        } else {
            let raw = path.raw(SkResolveConvexity::Yes).unwrap_or_else(|| {
                SkPathRaw::new(
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Rect::make_empty(),
                    path.get_fill_type(),
                    SkPathConvexity::Unknown,
                    0,
                )
            });
            let clip = SkRegion::from_irect(self.bounds);
            let (left, right) = (self.bounds.left, self.bounds.right);
            let mut blitter = BuilderBlitter {
                builder: self,
                left,
                right,
                min_y: K_MAX_INT32,
                last_y: -K_MAX_INT32,
            };
            {
                sk_scan_fill_path(&raw, &clip, &mut blitter);
                blitter.finish();
            }
        }
        self.finish(target)
    }

    /// 4×4 supersampled coverage, the same granularity as `SkScan::AntiFillPath`.
    fn blit_path_aa(&mut self, path: &SkPath) {
        let mut matrix = Matrix::default();
        matrix.set_scale(
            SK_SUPERSAMPLE_SCALE as Scalar,
            SK_SUPERSAMPLE_SCALE as Scalar,
        );
        let scaled = path.make_transform(&matrix);
        let clip = SkRegion::from_irect(IRect::new(
            self.bounds.left * SK_SUPERSAMPLE_SCALE,
            self.bounds.top * SK_SUPERSAMPLE_SCALE,
            self.bounds.right * SK_SUPERSAMPLE_SCALE,
            self.bounds.bottom * SK_SUPERSAMPLE_SCALE,
        ));
        let mut scaled_rgn = SkRegion::new();
        scaled_rgn.set_path(&scaled, &clip);

        let width = self.bounds.width();
        let mut row = vec![0u8; width.max(0) as usize];
        let mut y = self.bounds.top;
        while y < self.bounds.bottom {
            let mut x = self.bounds.left;
            let mut covered = false;
            while x < self.bounds.right {
                let mut count = 0u32;
                for sy in 0..SK_SUPERSAMPLE_SCALE {
                    for sx in 0..SK_SUPERSAMPLE_SCALE {
                        if scaled_rgn
                            .contains(x * SK_SUPERSAMPLE_SCALE + sx, y * SK_SUPERSAMPLE_SCALE + sy)
                        {
                            count += 1;
                        }
                    }
                }
                let alpha = ((count * 255 + 8) / 16) as u8;
                row[(x - self.bounds.left) as usize] = alpha;
                if alpha != 0 {
                    covered = true;
                }
                x += 1;
            }
            if covered {
                let mut run_start = 0i32;
                let mut run_alpha = row[0];
                for i in 1..=width {
                    let alpha = if i < width { row[i as usize] } else { 0xFF };
                    if alpha != run_alpha || i == width {
                        if run_alpha != 0 {
                            self.add_run(self.bounds.left + run_start, y, run_alpha, i - run_start);
                        }
                        run_start = i;
                        run_alpha = alpha;
                    }
                }
            }
            y += 1;
        }
    }
}

fn append_run(data: &mut Vec<u8>, alpha: u8, mut count: i32) {
    loop {
        let mut n = count;
        if n > 255 {
            n = 255;
        }
        data.push(n as u8);
        data.push(alpha);
        count -= n;
        if count <= 0 {
            break;
        }
    }
}

/// `Builder::Blitter`, the non-AA scan converter sink.
struct BuilderBlitter<'a> {
    builder: &'a mut Builder,
    left: i32,
    right: i32,
    min_y: i32,
    last_y: i32,
}

impl BuilderBlitter<'_> {
    /*
        If we see a gap of 1 or more empty scanlines while building in Y-order,
        we inject an explicit empty scanline (alpha==0)
    */
    fn check_for_y_gap(&mut self, y: i32) {
        debug_assert!(y >= self.last_y);
        if self.last_y > -K_MAX_INT32 {
            let gap = y - self.last_y;
            if gap > 1 {
                self.builder
                    .add_run(self.left, y - 1, 0, self.right - self.left);
            }
        }
        self.last_y = y;
    }

    fn record_min_y(&mut self, y: i32) {
        if y < self.min_y {
            self.min_y = y;
        }
    }

    fn finish(&mut self) {
        if self.min_y < K_MAX_INT32 {
            self.builder.min_y = self.min_y;
        }
    }
}

impl SkScanSpanSink for BuilderBlitter<'_> {
    fn blit_h(&mut self, x: i32, y: i32, width: i32) {
        self.record_min_y(y);
        self.check_for_y_gap(y);
        self.builder.add_run(x, y, 0xFF, width);
    }

    fn blit_rect(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.record_min_y(y);
        self.check_for_y_gap(y);
        self.builder.add_rect_run(x, y, width, height);
        self.last_y = y + height - 1;
    }
}

fn compute_row_length(row: &[u8], mut width: i32) -> usize {
    let mut pos = 0usize;
    while width > 0 {
        let n = row[pos] as i32;
        debug_assert!(n > 0);
        debug_assert!(n <= width);
        pos += 2;
        width -= n;
    }
    debug_assert_eq!(0, width);
    pos
}

fn count_left_right_zeros(row: &[u8], mut width: i32, left_z: &mut i32, rite_z: &mut i32) {
    let mut zeros = 0i32;
    let mut pos = 0usize;
    loop {
        if row[pos + 1] != 0 {
            break;
        }
        let n = row[pos] as i32;
        debug_assert!(n > 0);
        debug_assert!(n <= width);
        zeros += n;
        pos += 2;
        width -= n;
        if width <= 0 {
            break;
        }
    }
    *left_z = zeros;

    if 0 == width {
        // this line is completely empty return 'width' in both variables
        *rite_z = *left_z;
        return;
    }

    zeros = 0;
    while width > 0 {
        let n = row[pos] as i32;
        debug_assert!(n > 0);
        if row[pos + 1] == 0 {
            zeros += n;
        } else {
            zeros = 0;
        }
        pos += 2;
        width -= n;
    }
    *rite_z = zeros;
}

/// Modify `row` in place, trimming off zeros from the left and right sides. Returns the number of
/// bytes that were completely eliminated from the left.
fn trim_row_left_right(row: &mut [u8], mut width: i32, mut left_z: i32, mut rite_z: i32) -> usize {
    let mut trim = 0usize;
    let mut pos = 0usize;
    while left_z > 0 {
        debug_assert_eq!(0, row[pos + 1]);
        let n = row[pos] as i32;
        debug_assert!(n > 0);
        debug_assert!(n <= width);
        width -= n;
        pos += 2;
        if n > left_z {
            row[pos - 2] = (n - left_z) as u8;
            break;
        }
        trim += 2;
        left_z -= n;
        debug_assert!(left_z >= 0);
    }

    if rite_z != 0 {
        // walk row to the end, and then we'll back up to trim rite_z
        while width > 0 {
            let n = row[pos] as i32;
            debug_assert!(n <= width);
            width -= n;
            pos += 2;
        }
        // now skip whole runs of zeros
        loop {
            pos -= 2;
            debug_assert_eq!(0, row[pos + 1]);
            let n = row[pos] as i32;
            debug_assert!(n > 0);
            if n > rite_z {
                row[pos] = (n - rite_z) as u8;
                break;
            }
            rite_z -= n;
            debug_assert!(rite_z >= 0);
            if rite_z <= 0 {
                break;
            }
        }
    }

    trim
}

fn row_is_all_zeros(row: &[u8], mut width: i32) -> bool {
    debug_assert!(width > 0);
    let mut pos = 0usize;
    loop {
        if row[pos + 1] != 0 {
            return false;
        }
        let n = row[pos] as i32;
        debug_assert!(n <= width);
        width -= n;
        pos += 2;
        if width <= 0 {
            break;
        }
    }
    debug_assert_eq!(0, width);
    true
}

#[derive(Clone, Debug)]
struct AaRunHead {
    head: RunHead,
    offsets: Vec<YOffset>,
}

/// Port of `class SkAAClip`.
#[derive(Clone, Debug, Default)]
pub struct SkAAClip {
    bounds: IRect,
    run_head: Option<Arc<AaRunHead>>,
}

impl SkAAClip {
    /// `SkAAClip()`.
    pub fn new() -> SkAAClip {
        SkAAClip::default()
    }

    /// `SkAAClip::isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.run_head.is_none()
    }

    /// `SkAAClip::getBounds`.
    pub fn get_bounds(&self) -> &IRect {
        &self.bounds
    }

    fn free_runs(&mut self) {
        self.run_head = None;
    }

    /// `SkAAClip::setEmpty`.
    pub fn set_empty(&mut self) -> bool {
        self.free_runs();
        self.bounds.set_empty();
        false
    }

    /// `SkAAClip::setRect`.
    pub fn set_rect(&mut self, bounds: &IRect) -> bool {
        if bounds.is_empty() {
            return self.set_empty();
        }
        self.free_runs();
        self.bounds = *bounds;
        let head = RunHead::alloc_rect(bounds);
        self.run_head = Some(Arc::new(AaRunHead {
            offsets: vec![YOffset {
                y: bounds.height() - 1,
                offset: 0,
            }],
            head,
        }));
        debug_assert!(!self.is_empty());
        true
    }

    /// `SkAAClip::isRect`.
    pub fn is_rect(&self) -> bool {
        let Some(run_head) = &self.run_head else {
            return false;
        };
        if run_head.head.row_count != 1 {
            return false;
        }
        let yoff = run_head.offsets[0];
        if yoff.y != self.bounds.bottom - 1 {
            return false;
        }

        let row = &run_head.head.data[yoff.offset as usize..];
        let mut width = self.bounds.width();
        let mut pos = 0usize;
        loop {
            if row[pos + 1] != 0xFF {
                return false;
            }
            let n = row[pos] as i32;
            debug_assert!(n <= width);
            width -= n;
            pos += 2;
            if width <= 0 {
                break;
            }
        }
        true
    }

    /// `SkAAClip::setRegion`.
    pub fn set_region(&mut self, rgn: &SkRegion) -> bool {
        if rgn.is_empty() {
            return self.set_empty();
        }
        if rgn.is_rect() {
            return self.set_rect(rgn.get_bounds());
        }

        let bounds = *rgn.get_bounds();
        let offset_x = bounds.left;
        let offset_y = bounds.top;

        let mut y_array: Vec<YOffset> = Vec::new();
        let mut x_array: Vec<u8> = Vec::new();

        fn append_x_run(x_array: &mut Vec<u8>, value: u8, mut count: i32) {
            debug_assert!(count >= 0);
            while count > 0 {
                let n = count.min(255);
                x_array.push(n as u8);
                x_array.push(value);
                count -= n;
            }
        }

        let mut iter = RegionIterator::from_region(rgn);
        let mut prev_right = 0i32;
        let mut prev_bot = 0i32;
        let mut have_curr = false;

        while !iter.done() {
            let r = *iter.rect();
            debug_assert!(bounds.contains_irect(&r));

            let bot = r.bottom - offset_y;
            debug_assert!(bot >= prev_bot);
            if bot > prev_bot {
                if have_curr {
                    // flush current row
                    append_x_run(&mut x_array, 0, bounds.width() - prev_right);
                }
                // did we introduce an empty-gap from the prev row?
                let top = r.top - offset_y;
                if top > prev_bot {
                    y_array.push(YOffset {
                        y: top - 1,
                        offset: x_array.len() as u32,
                    });
                    append_x_run(&mut x_array, 0, bounds.width());
                }
                // create a new record for this Y value
                y_array.push(YOffset {
                    y: bot - 1,
                    offset: x_array.len() as u32,
                });
                prev_right = 0;
                prev_bot = bot;
                have_curr = true;
            }

            let x = r.left - offset_x;
            append_x_run(&mut x_array, 0, x - prev_right);

            let w = r.right - r.left;
            append_x_run(&mut x_array, 0xFF, w);
            prev_right = x + w;
            debug_assert!(prev_right <= bounds.width());
            iter.next();
        }
        // flush last row
        append_x_run(&mut x_array, 0, bounds.width() - prev_right);

        let head = RunHead {
            row_count: y_array.len() as i32,
            data: x_array,
        };

        self.set_empty();
        self.bounds = bounds;
        self.run_head = Some(Arc::new(AaRunHead {
            head,
            offsets: y_array,
        }));
        true
    }

    /// `SkAAClip::setPath`.
    pub fn set_path(&mut self, path: &SkPath, clip: &IRect, do_aa: bool) -> bool {
        if clip.is_empty() {
            return self.set_empty();
        }

        // Since we assert that the BuilderBlitter will never blit outside the intersection
        // of clip and ibounds, we create the builder with the snug bounds.
        let ibounds = if path.is_inverse_fill_type() {
            *clip
        } else {
            let mut b = path.get_bounds().round_out();
            // It's possible the bounds of our path might exceed SK_MaxS32 in width
            // but since our clip is within that width (otherwise isEmpty() above
            // would catch it), we can use isEmpty64() safely here. blitPath will
            // intersect the two bounds before drawing.
            if b.is_empty() || !b.intersect(clip) {
                return self.set_empty();
            }
            b
        };

        let mut builder = Builder::new(ibounds);
        builder.blit_path(self, path, do_aa)
    }

    /// `SkAAClip::op(const SkAAClip&, SkClipOp)`.
    pub fn op(&mut self, other: &SkAAClip, op: SkClipOp) -> bool {
        if self.is_empty() {
            // Once the clip is empty, it cannot become un-empty.
            return false;
        }

        let mut bounds = self.bounds;
        match op {
            SkClipOp::Difference => {
                if other.is_empty() || !IRect::intersects_irects(&self.bounds, &other.bounds) {
                    // this remains unmodified and isn't empty
                    return true;
                }
            }
            SkClipOp::Intersect => {
                if other.is_empty() || !bounds.intersect(&other.bounds) {
                    // the intersected clip becomes empty
                    return self.set_empty();
                }
            }
        }

        debug_assert!(IRect::intersects_irects(&bounds, &self.bounds));
        debug_assert!(IRect::intersects_irects(&bounds, &other.bounds));

        let mut builder = Builder::new(bounds);
        builder.apply_clip_op(self, other, op)
    }

    /// `SkAAClip::op(const SkIRect&, SkClipOp)`.
    pub fn op_irect(&mut self, rect: &IRect, op: SkClipOp) -> bool {
        // It can be expensive to build a local aaclip before applying the op, so
        // we first see if we can restrict the bounds of new rect to our current
        // bounds, or note that the new rect subsumes our current clip.
        let mut pixel_bounds = self.bounds;
        if !pixel_bounds.intersect(rect) {
            // No change or clip becomes empty depending on 'op'
            return match op {
                SkClipOp::Difference => !self.is_empty(),
                SkClipOp::Intersect => self.set_empty(),
            };
        } else if pixel_bounds == self.bounds {
            // Wholly inside 'rect', so clip becomes empty or remains unchanged
            return match op {
                SkClipOp::Difference => self.set_empty(),
                SkClipOp::Intersect => !self.is_empty(),
            };
        } else if op == SkClipOp::Intersect && self.quick_contains(&pixel_bounds) {
            // We become just the remaining rectangle
            return self.set_rect(&pixel_bounds);
        }
        let mut clip = SkAAClip::new();
        clip.set_rect(rect);
        self.op(&clip, op)
    }

    /// `SkAAClip::op(const SkRect&, SkClipOp, bool)`.
    pub fn op_rect(&mut self, rect: &Rect, op: SkClipOp, do_aa: bool) -> bool {
        if !do_aa {
            return self.op_irect(&rect.round(), op);
        }
        // Tighten bounds for "path" aaclip of the rect
        let mut pixel_bounds = self.bounds;
        if !pixel_bounds.intersect(&rect.round_out()) {
            // No change or clip becomes empty depending on 'op'
            return match op {
                SkClipOp::Difference => !self.is_empty(),
                SkClipOp::Intersect => self.set_empty(),
            };
        } else if rect.contains_rect(&Rect::from(self.bounds)) {
            // Wholly inside 'rect', so clip becomes empty or remains unchanged
            return match op {
                SkClipOp::Difference => self.set_empty(),
                SkClipOp::Intersect => !self.is_empty(),
            };
        } else if op == SkClipOp::Intersect && self.quick_contains(&pixel_bounds) {
            // We become just the rect intersected with pixel bounds (preserving fractional coords
            // for AA edges).
            return self.set_path(&SkPath::rect_default(rect), &pixel_bounds, true);
        }
        let mut rect_clip = SkAAClip::new();
        rect_clip.set_path(
            &SkPath::rect_default(rect),
            &if op == SkClipOp::Difference {
                self.bounds
            } else {
                pixel_bounds
            },
            true,
        );
        self.op(&rect_clip, op)
    }

    /// `SkAAClip::translate`.
    pub fn translate(&self, dx: i32, dy: i32, dst: &mut SkAAClip) -> bool {
        if self.is_empty() {
            return dst.set_empty();
        }
        if !core::ptr::eq(self, dst) {
            dst.free_runs();
            dst.run_head = self.run_head.clone();
            dst.bounds = self.bounds;
        }
        dst.bounds.offset(dx, dy);
        true
    }

    /// `SkAAClip::findRow`.
    fn find_row(&self, y: i32, last_y_for_row: Option<&mut i32>) -> Option<&[u8]> {
        let Some(run_head) = &self.run_head else {
            return None;
        };
        if y < self.bounds.top || y >= self.bounds.bottom {
            return None;
        }
        let y = y - self.bounds.top; // our yoffs values are relative to the top

        let mut i = 0usize;
        while run_head.offsets[i].y < y {
            i += 1;
            debug_assert!(i < run_head.head.row_count as usize);
        }
        if let Some(last) = last_y_for_row {
            *last = self.bounds.top + run_head.offsets[i].y;
        }
        Some(&run_head.head.data[run_head.offsets[i].offset as usize..])
    }

    /// `SkAAClip::findX`.
    fn find_x<'a>(
        &self,
        mut data: &'a [u8],
        mut x: i32,
        initial_count: Option<&mut i32>,
    ) -> &'a [u8] {
        debug_assert!(x >= self.bounds.left && x < self.bounds.right);
        x -= self.bounds.left;

        // first skip up to X
        loop {
            let n = data[0] as i32;
            if x < n {
                if let Some(initial_count) = initial_count {
                    *initial_count = n - x;
                }
                break;
            }
            data = &data[2..];
            x -= n;
        }
        data
    }

    /// `SkAAClip::quickContains(int, int, int, int)`.
    pub fn quick_contains_edges(&self, left: i32, top: i32, right: i32, bottom: i32) -> bool {
        if self.is_empty() {
            return false;
        }
        if !self
            .bounds
            .contains_irect(&IRect::new(left, top, right, bottom))
        {
            return false;
        }

        let mut last_y = 0i32;
        let Some(mut row) = self.find_row(top, Some(&mut last_y)) else {
            return false;
        };
        if last_y < bottom {
            return false;
        }
        // now just need to check in X
        let mut count = 0i32;
        row = self.find_x(row, left, Some(&mut count));

        let mut rect_width = right - left;
        while 0xFF == row[1] {
            if count >= rect_width {
                return true;
            }
            rect_width -= count;
            row = &row[2..];
            count = row[0] as i32;
        }
        false
    }

    /// `SkAAClip::quickContains(const SkIRect&)`.
    pub fn quick_contains(&self, r: &IRect) -> bool {
        self.quick_contains_edges(r.left, r.top, r.right, r.bottom)
    }

    /// The stored run count, for development diagnostics.
    pub fn row_count(&self) -> usize {
        self.run_head
            .as_ref()
            .map(|h| h.head.row_count as usize)
            .unwrap_or(0)
    }

    /// Expand the clip into per-row `[count, alpha]` spans, in Y order. This is the data
    /// `copyToMask` and `SkAAClipBlitter` consume, exposed without `SkMask`/`SkBlitter`.
    pub fn row_spans(&self) -> Vec<RowSpan<'_>> {
        let mut out = Vec::new();
        let mut iter = AaIter::new(self);
        while !iter.done() {
            out.push(RowSpan {
                top: iter.top(),
                bottom: iter.bottom(),
                data: iter.data().unwrap(),
            });
            iter.next();
        }
        out
    }

    fn trim_left_right(&mut self) -> bool {
        if self.is_empty() {
            return false;
        }

        let width = self.bounds.width();
        let head = Arc::make_mut(self.run_head.as_mut().unwrap());

        // After this loop, 'leftZeros' & 'rightZeros' will contain the minimum
        // number of zeros on the left and right of the clip. This information
        // can be used to shrink the bounding box.
        let mut left_zeros = width;
        let mut rite_zeros = width;
        for yoff in &head.offsets {
            let (mut l, mut r) = (0i32, 0i32);
            count_left_right_zeros(
                &head.head.data[yoff.offset as usize..],
                width,
                &mut l,
                &mut r,
            );
            debug_assert!(l + r < width || (l == width && r == width));
            if l < left_zeros {
                left_zeros = l;
            }
            if r < rite_zeros {
                rite_zeros = r;
            }
            if 0 == (left_zeros | rite_zeros) {
                // no trimming to do
                return true;
            }
        }

        debug_assert!(left_zeros != 0 || rite_zeros != 0);
        if width == left_zeros {
            debug_assert_eq!(width, rite_zeros);
            return self.set_empty();
        }

        self.bounds.left += left_zeros;
        self.bounds.right -= rite_zeros;
        debug_assert!(!self.bounds.is_empty());

        // For now we don't realloc the storage (for time), we just shrink in place.
        // This means we don't have to do any memmoves either, since we can just
        // play tricks with the yoff->fOffset for each row.
        let head = Arc::make_mut(self.run_head.as_mut().unwrap());
        for i in 0..head.offsets.len() {
            let offset = head.offsets[i].offset as usize;
            let trim =
                trim_row_left_right(&mut head.head.data[offset..], width, left_zeros, rite_zeros);
            head.offsets[i].offset += trim as u32;
        }
        true
    }

    fn trim_top_bottom(&mut self) -> bool {
        if self.is_empty() {
            return false;
        }

        let width = self.bounds.width();
        let head = Arc::make_mut(self.run_head.as_mut().unwrap());

        //  Look to trim away empty rows from the top.
        let mut skip = 0usize;
        for yoff in &head.offsets {
            let data = &head.head.data[yoff.offset as usize..];
            if !row_is_all_zeros(data, width) {
                break;
            }
            skip += 1;
        }
        debug_assert!(skip <= head.head.row_count as usize);
        if skip == head.head.row_count as usize {
            return self.set_empty();
        }
        if skip > 0 {
            // adjust fRowCount and fBounds.fTop, and slide all the data up
            // as we remove [skip] number of YOffset entries
            let dy = head.offsets[skip - 1].y + 1;
            for i in skip..head.head.row_count as usize {
                debug_assert!(head.offsets[i].y >= dy);
                head.offsets[i].y -= dy;
            }

            self.bounds.top += dy;
            debug_assert!(!self.bounds.is_empty());
            head.head.row_count -= skip as i32;
            debug_assert!(head.head.row_count > 0);
            head.offsets.drain(0..skip);
        }

        //  Look to trim away empty rows from the bottom.
        //  We know that we have at least one non-zero row, so we can just walk
        //  backwards without checking for running past the start.
        let head = Arc::make_mut(self.run_head.as_mut().unwrap());
        let mut idx = head.offsets.len() - 1;
        while row_is_all_zeros(&head.head.data[head.offsets[idx].offset as usize..], width) {
            idx -= 1;
        }
        let skip = head.offsets.len() - 1 - idx;
        if skip > 0 {
            self.bounds.bottom = self.bounds.top + head.offsets[idx].y + 1;
            debug_assert!(!self.bounds.is_empty());
            head.head.row_count -= skip as i32;
            debug_assert!(head.head.row_count > 0);
            head.offsets.truncate(idx + 1);
        }
        true
    }

    /// `SkAAClip::trimBounds`.
    fn trim_bounds(&mut self) -> bool {
        if self.is_empty() {
            return false;
        }

        let head = self.run_head.as_ref().unwrap();
        debug_assert!(head.head.row_count > 0);
        let last_y = head.offsets[head.head.row_count as usize - 1];
        debug_assert!(last_y.y < self.bounds.height());
        self.bounds.bottom = self.bounds.top + last_y.y + 1;
        debug_assert_eq!(last_y.y + 1, self.bounds.height());
        debug_assert!(!self.bounds.is_empty());

        self.trim_top_bottom() && self.trim_left_right()
    }
}

impl Builder {
    fn apply_clip_op(&mut self, target: &mut SkAAClip, other: &SkAAClip, op: SkClipOp) -> bool {
        self.operate_y(target, other, op);
        self.finish(target)
    }

    /// `SkAAClip::Builder::operateX`.
    fn operate_x(
        &mut self,
        last_y: i32,
        iter_a: &mut RowIter,
        iter_b: &mut RowIter,
        proc: fn(u8, u8) -> u8,
    ) {
        fn advance_row_iter(
            iter: &mut RowIter,
            iter_left: &mut i32,
            iter_rite: &mut i32,
            rite: i32,
        ) {
            if rite == *iter_rite {
                iter.next();
                *iter_left = iter.left();
                *iter_rite = iter.right();
            }
        }

        let mut left_a = iter_a.left();
        let mut rite_a = iter_a.right();
        let mut left_b = iter_b.left();
        let mut rite_b = iter_b.right();

        let mut prev_rite = self.bounds.left;

        loop {
            let mut alpha_a = 0u8;
            let mut alpha_b = 0u8;
            let left;
            let rite;

            if left_a < left_b {
                left = left_a;
                alpha_a = iter_a.alpha();
                if rite_a <= left_b {
                    rite = rite_a;
                } else {
                    rite = left_b;
                    left_a = left_b;
                }
            } else if left_b < left_a {
                left = left_b;
                alpha_b = iter_b.alpha();
                if rite_b <= left_a {
                    rite = rite_b;
                } else {
                    rite = left_a;
                    left_b = left_a;
                }
            } else {
                left = left_a; // or leftB, since leftA == leftB
                rite = rite_a.min(rite_b);
                left_a = rite;
                left_b = rite;
                alpha_a = iter_a.alpha();
                alpha_b = iter_b.alpha();
            }

            if left >= self.bounds.right {
                break;
            }
            let rite = if rite > self.bounds.right {
                self.bounds.right
            } else {
                rite
            };

            if left >= self.bounds.left {
                debug_assert!(rite > left);
                self.add_run(left, last_y, proc(alpha_a, alpha_b), rite - left);
                prev_rite = rite;
            }

            advance_row_iter(iter_a, &mut left_a, &mut rite_a, rite);
            advance_row_iter(iter_b, &mut left_b, &mut rite_b, rite);

            if iter_a.done() && iter_b.done() {
                break;
            }
        }

        if prev_rite < self.bounds.right {
            self.add_run(prev_rite, last_y, 0, self.bounds.right - prev_rite);
        }
    }

    /// `SkAAClip::Builder::operateY`.
    fn operate_y(&mut self, a: &SkAAClip, b: &SkAAClip, op: SkClipOp) {
        fn k_diff(a: u8, b: u8) -> u8 {
            sk_mul_div255_round(a as u16, 0xFF - b as u16) as u8
        }
        fn k_intersect(a: u8, b: u8) -> u8 {
            sk_mul_div255_round(a as u16, b as u16) as u8
        }
        let proc: fn(u8, u8) -> u8 = if op == SkClipOp::Difference {
            k_diff
        } else {
            k_intersect
        };

        let mut iter_a = AaIter::new(a);
        let mut iter_b = AaIter::new(b);

        debug_assert!(!iter_a.done());
        let mut top_a = iter_a.top();
        let mut bot_a = iter_a.bottom();
        debug_assert!(!iter_b.done());
        let mut top_b = iter_b.top();
        let mut bot_b = iter_b.bottom();

        fn advance_iter(iter: &mut AaIter, iter_top: &mut i32, iter_bot: &mut i32, bot: i32) {
            if bot == *iter_bot {
                iter.next();
                *iter_top = *iter_bot;
                debug_assert_eq!(*iter_bot, iter.top());
                *iter_bot = iter.bottom();
            }
        }

        loop {
            let mut row_a: Option<&[u8]> = None;
            let mut row_b: Option<&[u8]> = None;
            let top;
            let bot;

            if top_a < top_b {
                top = top_a;
                row_a = iter_a.data();
                if bot_a <= top_b {
                    bot = bot_a;
                } else {
                    bot = top_b;
                    top_a = top_b;
                }
            } else if top_b < top_a {
                top = top_b;
                row_b = iter_b.data();
                if bot_b <= top_a {
                    bot = bot_b;
                } else {
                    bot = top_a;
                    top_b = top_a;
                }
            } else {
                top = top_a; // or topB, since topA == topB
                bot = bot_a.min(bot_b);
                top_a = bot;
                top_b = bot;
                row_a = iter_a.data();
                row_b = iter_b.data();
            }

            if top >= self.bounds.bottom {
                break;
            }

            let bot = if bot > self.bounds.bottom {
                self.bounds.bottom
            } else {
                bot
            };
            debug_assert!(top < bot);

            if row_a.is_none() && row_b.is_none() {
                self.add_run(self.bounds.left, bot - 1, 0, self.bounds.width());
            } else if top >= self.bounds.top {
                debug_assert!(bot <= self.bounds.bottom);
                let mut row_iter_a = RowIter::new(
                    row_a,
                    if row_a.is_some() {
                        a.get_bounds()
                    } else {
                        &self.bounds
                    },
                );
                let mut row_iter_b = RowIter::new(
                    row_b,
                    if row_b.is_some() {
                        b.get_bounds()
                    } else {
                        &self.bounds
                    },
                );
                self.operate_x(bot - 1, &mut row_iter_a, &mut row_iter_b, proc);
            }

            advance_iter(&mut iter_a, &mut top_a, &mut bot_a, bot);
            advance_iter(&mut iter_b, &mut top_b, &mut bot_b, bot);

            if iter_a.done() && iter_b.done() {
                break;
            }
        }
    }
}

/// `SkRegion` conversions used by `SkRasterClip::op(const SkRegion&, SkClipOp)`; kept here because
/// the AA clip is the only consumer.
pub fn set_region_on_clip(clip: &mut SkAAClip, rgn: &SkRegion) -> bool {
    clip.set_region(rgn)
}

/// The `SkRegion::Op` used by the clip stack's elements.
pub fn region_op_from_clip_op(op: SkClipOp) -> SkRegionOp {
    SkRegionOp::from(op)
}

/// `SkRegion::BoundsType` is shared with the clip stack; re-exported for callers that use the AA
/// clip alongside a region.
pub type RegionBoundsType = BoundsType;

/// Unused helper retained so the module's path builder dependency stays exercised.
#[allow(dead_code)]
fn rect_path(rect: &Rect) -> SkPath {
    let mut builder = SkPathBuilder::new();
    builder.add_rect(rect, SkPathDirection::Cw, 0);
    builder.detach(None)
}
