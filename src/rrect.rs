use crate::float_utils::SkFloatingPoint;
use crate::floating_point::{self, float_midpoint};
use crate::matrix::Matrix;
use crate::point::Point;
use crate::rect::Rect;
use crate::rect_priv::RectPriv;
use crate::scalar::{SCALAR_NEARLY_ZERO, Scalar, scalar_nearly_equal};
use crate::scale_to_sides;
use crate::writer::{SkRBuffer, SkWBuffer};

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Type {
    #[default]
    Empty = 0,
    Rect = 1,
    Oval = 2,
    Simple = 3,
    NinePatch = 4,
    Complex = 5,
}

impl Type {
    pub const LAST: Type = Type::Complex;
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Corner {
    UpperLeft = 0,
    UpperRight = 1,
    LowerRight = 2,
    LowerLeft = 3,
}

impl Corner {
    pub const fn index(self) -> usize {
        self as usize
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RRect {
    pub(crate) rect: Rect,
    pub(crate) radii: [Point; 4],
    pub(crate) kind: Type,
}

const _: () = {
    assert!(core::mem::size_of::<RRect>() == 52);
    assert!(core::mem::offset_of!(RRect, rect) == 0);
    assert!(core::mem::offset_of!(RRect, radii) == 16);
    assert!(core::mem::offset_of!(RRect, kind) == 48);
};

impl PartialEq for RRect {
    fn eq(&self, other: &Self) -> bool {
        self.rect == other.rect
            && self
                .radii
                .iter()
                .zip(other.radii.iter())
                .all(|(a, b)| a == b)
    }
}

impl RRect {
    pub const K_SIZE_IN_MEMORY: usize = 12 * core::mem::size_of::<Scalar>();

    pub const fn new() -> Self {
        Self {
            rect: Rect::make_empty(),
            radii: [Point::new(0.0, 0.0); 4],
            kind: Type::Empty,
        }
    }

    fn with_parts(rect: Rect, radii: [Point; 4], kind: Type) -> Self {
        Self { rect, radii, kind }
    }

    pub fn make_empty() -> Self {
        Self::new()
    }

    pub fn make_rect(r: &Rect) -> Self {
        let mut rr = Self::new();
        rr.set_rect(r);
        rr
    }

    pub fn make_oval(oval: &Rect) -> Self {
        let mut rr = Self::new();
        rr.set_oval(oval);
        rr
    }

    pub fn make_rect_xy(rect: &Rect, x_rad: Scalar, y_rad: Scalar) -> Self {
        let mut rr = Self::new();
        rr.set_rect_xy(rect, x_rad, y_rad);
        rr
    }

    pub fn make_rect_radii(rect: &Rect, radii: &[Point; 4]) -> Self {
        let mut rr = Self::new();
        rr.set_rect_radii(rect, radii);
        rr
    }

    pub fn get_type(&self) -> Type {
        debug_assert!(self.is_valid());
        self.kind
    }

    pub fn kind(&self) -> Type {
        self.kind
    }

    pub const fn is_empty(&self) -> bool {
        matches!(self.kind, Type::Empty)
    }

    pub const fn is_rect(&self) -> bool {
        matches!(self.kind, Type::Rect)
    }

    pub const fn is_oval(&self) -> bool {
        matches!(self.kind, Type::Oval)
    }

    pub const fn is_simple(&self) -> bool {
        matches!(self.kind, Type::Simple)
    }

    pub const fn is_nine_patch(&self) -> bool {
        matches!(self.kind, Type::NinePatch)
    }

    pub const fn is_complex(&self) -> bool {
        matches!(self.kind, Type::Complex)
    }

    pub fn width(&self) -> Scalar {
        self.rect.width()
    }

    pub fn height(&self) -> Scalar {
        self.rect.height()
    }

    pub fn get_simple_radii(&self) -> Point {
        self.radii[Corner::UpperLeft.index()]
    }

    pub fn set_empty(&mut self) {
        *self = Self::new();
    }

    pub fn set_rect(&mut self, rect: &Rect) {
        if !self.initialize_rect(rect) {
            return;
        }
        self.radii = [Point::new(0.0, 0.0); 4];
        self.kind = Type::Rect;
        debug_assert!(self.is_valid());
    }

    pub fn set_oval(&mut self, oval: &Rect) {
        if !self.initialize_rect(oval) {
            return;
        }
        let x_rad = RectPriv::half_width(&self.rect);
        let y_rad = RectPriv::half_height(&self.rect);
        if x_rad == 0.0 || y_rad == 0.0 {
            self.radii = [Point::new(0.0, 0.0); 4];
            self.kind = Type::Rect;
        } else {
            self.radii = [Point::new(x_rad, y_rad); 4];
            self.kind = Type::Oval;
        }
        debug_assert!(self.is_valid());
    }

    pub fn set_rect_xy(&mut self, rect: &Rect, mut x_rad: Scalar, mut y_rad: Scalar) {
        if !self.initialize_rect(rect) {
            return;
        }

        if !floating_point::is_finite2(x_rad, y_rad) {
            x_rad = 0.0;
            y_rad = 0.0;
        }

        if self.rect.width() < x_rad + x_rad || self.rect.height() < y_rad + y_rad {
            let scale = floating_point::ieee_float_divide(self.rect.width(), x_rad + x_rad).min(
                floating_point::ieee_float_divide(self.rect.height(), y_rad + y_rad),
            );
            debug_assert!(scale < 1.0);
            x_rad *= scale;
            y_rad *= scale;
        }

        if x_rad <= 0.0 || y_rad <= 0.0 {
            self.set_rect(rect);
            return;
        }

        self.radii = [Point::new(x_rad, y_rad); 4];
        self.kind = Type::Simple;
        if x_rad >= self.rect.width() / 2.0 && y_rad >= self.rect.height() / 2.0 {
            self.kind = Type::Oval;
            x_rad = RectPriv::half_width(&self.rect);
            y_rad = RectPriv::half_height(&self.rect);
            self.radii = [Point::new(x_rad, y_rad); 4];
        }

        debug_assert!(self.is_valid());
    }

    pub fn set_nine_patch(
        &mut self,
        rect: &Rect,
        mut left_rad: Scalar,
        mut top_rad: Scalar,
        mut right_rad: Scalar,
        mut bottom_rad: Scalar,
    ) {
        if !self.initialize_rect(rect) {
            return;
        }

        if !floating_point::is_finite4(left_rad, top_rad, right_rad, bottom_rad) {
            self.set_rect(rect);
            return;
        }

        left_rad = left_rad.max(0.0);
        top_rad = top_rad.max(0.0);
        right_rad = right_rad.max(0.0);
        bottom_rad = bottom_rad.max(0.0);

        let mut scale = 1.0f32;
        if left_rad + right_rad > self.rect.width() {
            scale = self.rect.width() / (left_rad + right_rad);
        }
        if top_rad + bottom_rad > self.rect.height() {
            scale = scale.min(self.rect.height() / (top_rad + bottom_rad));
        }

        if scale < 1.0 {
            left_rad *= scale;
            top_rad *= scale;
            right_rad *= scale;
            bottom_rad *= scale;
        }

        if left_rad == right_rad && top_rad == bottom_rad {
            if left_rad >= self.rect.width() / 2.0 && top_rad >= self.rect.height() / 2.0 {
                self.kind = Type::Oval;
                left_rad = RectPriv::half_width(&self.rect);
                right_rad = left_rad;
                top_rad = RectPriv::half_height(&self.rect);
                bottom_rad = top_rad;
            } else if left_rad == 0.0 || top_rad == 0.0 {
                self.kind = Type::Rect;
                left_rad = 0.0;
                top_rad = 0.0;
                right_rad = 0.0;
                bottom_rad = 0.0;
            } else {
                self.kind = Type::Simple;
            }
        } else {
            self.kind = Type::NinePatch;
        }

        self.radii[Corner::UpperLeft.index()] = Point::new(left_rad, top_rad);
        self.radii[Corner::UpperRight.index()] = Point::new(right_rad, top_rad);
        self.radii[Corner::LowerRight.index()] = Point::new(right_rad, bottom_rad);
        self.radii[Corner::LowerLeft.index()] = Point::new(left_rad, bottom_rad);

        if Self::clamp_to_zero(&mut self.radii) {
            self.set_rect(rect);
            return;
        }
        if self.kind == Type::NinePatch && !Self::radii_are_nine_patch(&self.radii) {
            self.kind = Type::Complex;
        }

        debug_assert!(self.is_valid());
    }

    pub fn set_rect_radii(&mut self, rect: &Rect, radii: &[Point; 4]) {
        if !self.initialize_rect(rect) {
            return;
        }

        if !radii.iter().all(|r| r.is_finite()) {
            self.set_rect(rect);
            return;
        }

        self.radii = *radii;

        if Self::clamp_to_zero(&mut self.radii) {
            self.set_rect(rect);
            return;
        }

        self.scale_radii();

        if !self.is_valid() {
            self.set_rect(rect);
        }
    }

    pub fn rect(&self) -> &Rect {
        &self.rect
    }

    pub fn get_bounds(&self) -> &Rect {
        &self.rect
    }

    pub fn radii(&self) -> &[Point; 4] {
        &self.radii
    }

    pub fn radii_at(&self, corner: Corner) -> Point {
        self.radii[corner.index()]
    }

    pub fn inset(&self, dx: Scalar, dy: Scalar, dst: &mut RRect) {
        let mut r = self.rect.make_inset(dx, dy);
        let mut degenerate = false;
        if r.right <= r.left {
            degenerate = true;
            let midpoint = float_midpoint(r.left, r.right);
            r.left = midpoint;
            r.right = midpoint;
        }
        if r.bottom <= r.top {
            degenerate = true;
            let midpoint = float_midpoint(r.top, r.bottom);
            r.top = midpoint;
            r.bottom = midpoint;
        }
        if degenerate {
            dst.rect = r;
            dst.radii = [Point::new(0.0, 0.0); 4];
            dst.kind = Type::Empty;
            return;
        }
        if !r.is_finite() {
            *dst = Self::new();
            return;
        }

        let mut radii = self.radii;
        for radius in &mut radii {
            if radius.x != 0.0 {
                radius.x -= dx;
            }
            if radius.y != 0.0 {
                radius.y -= dy;
            }
        }
        dst.set_rect_radii(&r, &radii);
    }

    pub fn inset_in_place(&mut self, dx: Scalar, dy: Scalar) {
        let src = *self;
        src.inset(dx, dy, self);
    }

    pub fn outset(&self, dx: Scalar, dy: Scalar, dst: &mut RRect) {
        self.inset(-dx, -dy, dst);
    }

    pub fn outset_in_place(&mut self, dx: Scalar, dy: Scalar) {
        let src = *self;
        src.outset(dx, dy, self);
    }

    pub fn offset(&mut self, dx: Scalar, dy: Scalar) {
        self.rect.offset(dx, dy);
    }

    pub fn make_offset(&self, dx: Scalar, dy: Scalar) -> Self {
        Self::with_parts(self.rect.make_offset(dx, dy), self.radii, self.kind)
    }

    /// `SkRRect::transform` (non-legacy overload).
    ///
    /// Transforms by `matrix` and returns the result if possible. If the matrix does not
    /// preserve axis-alignment (e.g. rotates, skews, etc.) then this returns `None`.
    pub fn transform(&self, matrix: &Matrix) -> Option<RRect> {
        if matrix.is_identity() {
            return Some(*self);
        }

        if !matrix.preserves_axis_alignment() {
            return None;
        }

        if self.is_empty() {
            return Some(Self::make_empty());
        }

        let new_rect = matrix.map_rect(&self.rect);
        if !new_rect.is_finite() || new_rect.is_empty() {
            return None;
        }

        match self.get_type() {
            Type::Rect => return Some(Self::make_rect(&new_rect)),
            Type::Oval => return Some(Self::make_oval(&new_rect)),
            _ => {}
        }

        let mut raw = crate::sk_path_raw_shapes::SkPathRawShapes::rrect(
            self,
            crate::sk_path_types::SkPathDirection::Cw,
            6,
        );
        matrix.map_points_in_place(&mut raw.f_points);
        Some(crate::sk_path_priv::deduce_rrect_from_contour(
            &new_rect,
            &raw.f_points,
            &raw.f_verbs,
        ))
    }

    pub fn contains(&self, point: &Point) -> bool {
        if !self.rect.contains(point.x, point.y) {
            return false;
        }
        if self.is_rect() {
            return true;
        }
        self.check_corner_containment(point.x, point.y)
    }

    pub fn contains_rect(&self, rect: &Rect) -> bool {
        if !self.rect.contains_rect(rect) {
            return false;
        }
        if self.is_rect() {
            return true;
        }
        self.check_corner_containment(rect.left, rect.top)
            && self.check_corner_containment(rect.right, rect.top)
            && self.check_corner_containment(rect.right, rect.bottom)
            && self.check_corner_containment(rect.left, rect.bottom)
    }

    pub fn is_valid(&self) -> bool {
        if !RRectPriv::are_rect_and_radii_valid(&self.rect, &self.radii) {
            return false;
        }

        let mut all_radii_zero = self.radii[0].x == 0.0 && self.radii[0].y == 0.0;
        let mut all_corners_square = self.radii[0].x == 0.0 || self.radii[0].y == 0.0;
        let mut all_radii_same = true;

        for i in 1..4 {
            if self.radii[i].x != 0.0 || self.radii[i].y != 0.0 {
                all_radii_zero = false;
            }
            if self.radii[i].x != self.radii[i - 1].x || self.radii[i].y != self.radii[i - 1].y {
                all_radii_same = false;
            }
            if self.radii[i].x != 0.0 && self.radii[i].y != 0.0 {
                all_corners_square = false;
            }
        }
        let patches_of_nine = Self::radii_are_nine_patch(&self.radii);

        if self.kind != Type::Empty
            && self.kind != Type::Rect
            && self.kind != Type::Oval
            && self.kind != Type::Simple
            && self.kind != Type::NinePatch
            && self.kind != Type::Complex
        {
            return false;
        }

        match self.kind {
            Type::Empty => {
                if !self.rect.is_empty()
                    || !all_radii_zero
                    || !all_radii_same
                    || !all_corners_square
                {
                    return false;
                }
            }
            Type::Rect => {
                if self.rect.is_empty() || !all_radii_zero || !all_radii_same || !all_corners_square
                {
                    return false;
                }
            }
            Type::Oval => {
                if self.rect.is_empty() || all_radii_zero || !all_radii_same || all_corners_square {
                    return false;
                }

                let desired_width = RectPriv::half_width(&self.rect);
                let desired_height = RectPriv::half_height(&self.rect);
                let desired_width_ulp = SkFloatingPoint::<f32, 4>::new(desired_width);
                let desired_height_ulp = SkFloatingPoint::<f32, 4>::new(desired_height);

                for radius in &self.radii {
                    let x = SkFloatingPoint::<f32, 4>::new(radius.x);
                    let y = SkFloatingPoint::<f32, 4>::new(radius.y);
                    let x_matches = scalar_nearly_equal(radius.x, desired_width)
                        || x.almost_equals(desired_width_ulp);
                    let y_matches = scalar_nearly_equal(radius.y, desired_height)
                        || y.almost_equals(desired_height_ulp);
                    if !x_matches || !y_matches {
                        return false;
                    }
                }
            }
            Type::Simple => {
                if self.rect.is_empty() || all_radii_zero || !all_radii_same || all_corners_square {
                    return false;
                }
            }
            Type::NinePatch => {
                if self.rect.is_empty()
                    || all_radii_zero
                    || all_radii_same
                    || all_corners_square
                    || !patches_of_nine
                {
                    return false;
                }
            }
            Type::Complex => {
                if self.rect.is_empty()
                    || all_radii_zero
                    || all_radii_same
                    || all_corners_square
                    || patches_of_nine
                {
                    return false;
                }
            }
        }

        true
    }

    pub fn write_to_memory(&self, buffer: &mut [u8]) -> usize {
        if buffer.len() < Self::K_SIZE_IN_MEMORY {
            return 0;
        }
        let values = [
            self.rect.left,
            self.rect.top,
            self.rect.right,
            self.rect.bottom,
            self.radii[0].x,
            self.radii[0].y,
            self.radii[1].x,
            self.radii[1].y,
            self.radii[2].x,
            self.radii[2].y,
            self.radii[3].x,
            self.radii[3].y,
        ];
        for (index, value) in values.iter().enumerate() {
            buffer[index * 4..index * 4 + 4].copy_from_slice(&value.to_ne_bytes());
        }
        Self::K_SIZE_IN_MEMORY
    }

    pub fn read_from_memory(&mut self, buffer: &[u8]) -> usize {
        if buffer.len() < Self::K_SIZE_IN_MEMORY {
            return 0;
        }
        let mut values = [0.0f32; 12];
        for (index, value) in values.iter_mut().enumerate() {
            let start = index * 4;
            *value = f32::from_ne_bytes([
                buffer[start],
                buffer[start + 1],
                buffer[start + 2],
                buffer[start + 3],
            ]);
        }
        let rect = Rect::new(values[0], values[1], values[2], values[3]);
        let radii = [
            Point::new(values[4], values[5]),
            Point::new(values[6], values[7]),
            Point::new(values[8], values[9]),
            Point::new(values[10], values[11]),
        ];
        self.set_rect_radii(&rect, &radii);
        Self::K_SIZE_IN_MEMORY
    }

    fn initialize_rect(&mut self, rect: &Rect) -> bool {
        if !rect.is_finite() {
            *self = Self::new();
            return false;
        }
        self.rect = rect.make_sorted();
        if self.rect.is_empty() {
            self.radii = [Point::new(0.0, 0.0); 4];
            self.kind = Type::Empty;
            return false;
        }
        true
    }

    fn compute_type(&mut self) {
        if self.rect.is_empty() {
            debug_assert!(self.rect.is_sorted());
            debug_assert!(self.radii.iter().all(|r| *r == Point::new(0.0, 0.0)));
            self.kind = Type::Empty;
            debug_assert!(self.is_valid());
            return;
        }

        let mut all_radii_equal = true;
        let mut all_corners_square = self.radii[0].x == 0.0 || self.radii[0].y == 0.0;

        for i in 1..4 {
            if self.radii[i].x != 0.0 && self.radii[i].y != 0.0 {
                all_corners_square = false;
            }
            if self.radii[i].x != self.radii[i - 1].x || self.radii[i].y != self.radii[i - 1].y {
                all_radii_equal = false;
            }
        }

        if all_corners_square {
            self.kind = Type::Rect;
            debug_assert!(self.is_valid());
            return;
        }

        if all_radii_equal {
            if self.radii[0].x >= self.rect.width() / 2.0
                && self.radii[0].y >= self.rect.height() / 2.0
            {
                self.kind = Type::Oval;
                let x = RectPriv::half_width(&self.rect);
                let y = RectPriv::half_height(&self.rect);
                self.radii = [Point::new(x, y); 4];
            } else {
                self.kind = Type::Simple;
            }
            debug_assert!(self.is_valid());
            return;
        }

        if Self::radii_are_nine_patch(&self.radii) {
            self.kind = Type::NinePatch;
        } else {
            self.kind = Type::Complex;
        }

        if !self.is_valid() {
            let rect = self.rect;
            self.set_rect(&rect);
            debug_assert!(self.is_valid());
        }
    }

    fn check_corner_containment(&self, x: Scalar, y: Scalar) -> bool {
        let canonical;
        let index;

        if self.is_oval() {
            canonical = Point::new(x - self.rect.center_x(), y - self.rect.center_y());
            index = Corner::UpperLeft;
        } else if x < self.rect.left + self.radii[Corner::UpperLeft.index()].x
            && y < self.rect.top + self.radii[Corner::UpperLeft.index()].y
        {
            index = Corner::UpperLeft;
            canonical = Point::new(
                x - (self.rect.left + self.radii[Corner::UpperLeft.index()].x),
                y - (self.rect.top + self.radii[Corner::UpperLeft.index()].y),
            );
        } else if x < self.rect.left + self.radii[Corner::LowerLeft.index()].x
            && y > self.rect.bottom - self.radii[Corner::LowerLeft.index()].y
        {
            index = Corner::LowerLeft;
            canonical = Point::new(
                x - (self.rect.left + self.radii[Corner::LowerLeft.index()].x),
                y - (self.rect.bottom - self.radii[Corner::LowerLeft.index()].y),
            );
        } else if x > self.rect.right - self.radii[Corner::UpperRight.index()].x
            && y < self.rect.top + self.radii[Corner::UpperRight.index()].y
        {
            index = Corner::UpperRight;
            canonical = Point::new(
                x - (self.rect.right - self.radii[Corner::UpperRight.index()].x),
                y - (self.rect.top + self.radii[Corner::UpperRight.index()].y),
            );
        } else if x > self.rect.right - self.radii[Corner::LowerRight.index()].x
            && y > self.rect.bottom - self.radii[Corner::LowerRight.index()].y
        {
            index = Corner::LowerRight;
            canonical = Point::new(
                x - (self.rect.right - self.radii[Corner::LowerRight.index()].x),
                y - (self.rect.bottom - self.radii[Corner::LowerRight.index()].y),
            );
        } else {
            return true;
        }

        let radius = self.radii[index.index()];
        let dist = canonical.x * canonical.x * radius.y * radius.y
            + canonical.y * canonical.y * radius.x * radius.x;
        dist <= (radius.x * radius.y) * (radius.x * radius.y)
    }

    fn scale_radii(&mut self) -> bool {
        let mut scale = 1.0f64;

        let width = self.rect.right as f64 - self.rect.left as f64;
        let height = self.rect.bottom as f64 - self.rect.top as f64;
        scale =
            Self::compute_min_scale(self.radii[0].x as f64, self.radii[1].x as f64, width, scale);
        scale = Self::compute_min_scale(
            self.radii[1].y as f64,
            self.radii[2].y as f64,
            height,
            scale,
        );
        scale =
            Self::compute_min_scale(self.radii[2].x as f64, self.radii[3].x as f64, width, scale);
        scale = Self::compute_min_scale(
            self.radii[3].y as f64,
            self.radii[0].y as f64,
            height,
            scale,
        );

        Self::flush_pair(&mut self.radii, 0, 1, false);
        Self::flush_pair(&mut self.radii, 1, 2, true);
        Self::flush_pair(&mut self.radii, 2, 3, false);
        Self::flush_pair(&mut self.radii, 3, 0, true);

        if scale < 1.0 {
            let mut a = self.radii[0].x;
            let mut b = self.radii[1].x;
            scale_to_sides::adjust_radii(width, scale, &mut a, &mut b);
            self.radii[0].x = a;
            self.radii[1].x = b;

            let mut a = self.radii[1].y;
            let mut b = self.radii[2].y;
            scale_to_sides::adjust_radii(height, scale, &mut a, &mut b);
            self.radii[1].y = a;
            self.radii[2].y = b;

            let mut a = self.radii[2].x;
            let mut b = self.radii[3].x;
            scale_to_sides::adjust_radii(width, scale, &mut a, &mut b);
            self.radii[2].x = a;
            self.radii[3].x = b;

            let mut a = self.radii[3].y;
            let mut b = self.radii[0].y;
            scale_to_sides::adjust_radii(height, scale, &mut a, &mut b);
            self.radii[3].y = a;
            self.radii[0].y = b;
        }

        Self::clamp_to_zero(&mut self.radii);
        self.compute_type();
        scale < 1.0
    }

    fn flush_pair(radii: &mut [Point; 4], first: usize, second: usize, y_axis: bool) {
        let (a, b) = if y_axis {
            (radii[first].y, radii[second].y)
        } else {
            (radii[first].x, radii[second].x)
        };
        let (new_a, new_b) = flush_to_zero(a, b);
        if y_axis {
            radii[first].y = new_a;
            radii[second].y = new_b;
        } else {
            radii[first].x = new_a;
            radii[second].x = new_b;
        }
    }

    fn compute_min_scale(rad1: f64, rad2: f64, limit: f64, cur_min: f64) -> f64 {
        if rad1 + rad2 > limit {
            cur_min.min(limit / (rad1 + rad2))
        } else {
            cur_min
        }
    }

    fn clamp_to_zero(radii: &mut [Point; 4]) -> bool {
        let mut all_corners_square = true;
        for radius in radii {
            if radius.x <= 0.0 || radius.y <= 0.0 {
                radius.x = 0.0;
                radius.y = 0.0;
            } else {
                all_corners_square = false;
            }
        }
        all_corners_square
    }

    fn radii_are_nine_patch(radii: &[Point; 4]) -> bool {
        radii[Corner::UpperLeft.index()].x == radii[Corner::LowerLeft.index()].x
            && radii[Corner::UpperLeft.index()].y == radii[Corner::UpperRight.index()].y
            && radii[Corner::UpperRight.index()].x == radii[Corner::LowerRight.index()].x
            && radii[Corner::LowerLeft.index()].y == radii[Corner::LowerRight.index()].y
    }
}

fn flush_to_zero(a: Scalar, b: Scalar) -> (Scalar, Scalar) {
    debug_assert!(a >= 0.0);
    debug_assert!(b >= 0.0);
    if a + b == a {
        (a, 0.0)
    } else if a + b == b {
        (0.0, b)
    } else {
        (a, b)
    }
}

fn are_radii_predicates_valid(
    radius_from_min: Scalar,
    radius_from_max: Scalar,
    min_coord: Scalar,
    max_coord: Scalar,
) -> bool {
    if min_coord > max_coord || radius_from_min < 0.0 || radius_from_max < 0.0 {
        return false;
    }

    let limit = max_coord - min_coord;
    let sum = radius_from_min + radius_from_max;
    let pt_from_min = min_coord + radius_from_min;
    let pt_from_max = max_coord - radius_from_max;

    let sum_valid = sum <= limit
        || scalar_nearly_equal(sum, limit)
        || SkFloatingPoint::<f32, 4>::new(sum).almost_equals(SkFloatingPoint::<f32, 4>::new(limit));
    let pts_valid = pt_from_min <= pt_from_max
        || scalar_nearly_equal(pt_from_min, pt_from_max)
        || SkFloatingPoint::<f32, 4>::new(pt_from_min)
            .almost_equals(SkFloatingPoint::<f32, 4>::new(pt_from_max));

    if !sum_valid || !pts_valid {
        return false;
    }

    min_coord <= pt_from_max && pt_from_min <= max_coord
}

pub struct RRectPriv;

impl RRectPriv {
    pub fn is_circle(rr: &RRect) -> bool {
        rr.is_oval() && scalar_nearly_equal(rr.radii[0].x, rr.radii[0].y)
    }

    pub fn get_simple_radii(rr: &RRect) -> Point {
        debug_assert!(!rr.is_complex());
        rr.radii[0]
    }

    pub fn is_simple_circular(rr: &RRect) -> bool {
        rr.is_simple() && scalar_nearly_equal(rr.radii[0].x, rr.radii[0].y)
    }

    pub fn is_nearly_simple_circular(rr: &RRect, tolerance: Scalar) -> bool {
        let simple_radius = rr.radii[0].x;
        scalar_nearly_equal_within(simple_radius, rr.radii[0].y, tolerance)
            && scalar_nearly_equal_within(simple_radius, rr.radii[1].x, tolerance)
            && scalar_nearly_equal_within(simple_radius, rr.radii[1].y, tolerance)
            && scalar_nearly_equal_within(simple_radius, rr.radii[2].x, tolerance)
            && scalar_nearly_equal_within(simple_radius, rr.radii[2].y, tolerance)
            && scalar_nearly_equal_within(simple_radius, rr.radii[3].x, tolerance)
            && scalar_nearly_equal_within(simple_radius, rr.radii[3].y, tolerance)
    }

    pub fn is_nearly_simple_circular_default(rr: &RRect) -> bool {
        Self::is_nearly_simple_circular(rr, SCALAR_NEARLY_ZERO)
    }

    pub fn equal_radii(rr: &RRect) -> bool {
        rr.is_rect() || Self::is_circle(rr) || Self::is_simple_circular(rr)
    }

    pub fn get_radii_array(rr: &RRect) -> &[Point; 4] {
        &rr.radii
    }

    pub fn all_corners_circular(rr: &RRect, tolerance: Scalar) -> bool {
        rr.radii
            .iter()
            .all(|r| scalar_nearly_equal_within(r.x, r.y, tolerance))
    }

    pub fn all_corners_circular_default(rr: &RRect) -> bool {
        Self::all_corners_circular(rr, SCALAR_NEARLY_ZERO)
    }

    pub fn is_relatively_circular(rx: Scalar, ry: Scalar, tolerance: Scalar) -> bool {
        (rx <= tolerance && ry <= tolerance) || (rx - ry).abs() <= tolerance * rx.max(ry)
    }

    pub fn is_relatively_circular_default(rx: Scalar, ry: Scalar) -> bool {
        Self::is_relatively_circular(rx, ry, SCALAR_NEARLY_ZERO)
    }

    pub fn all_corners_relatively_circular(rr: &RRect, tolerance: Scalar) -> bool {
        rr.radii
            .iter()
            .all(|r| Self::is_relatively_circular(r.x, r.y, tolerance))
    }

    pub fn all_corners_relatively_circular_default(rr: &RRect) -> bool {
        Self::all_corners_relatively_circular(rr, SCALAR_NEARLY_ZERO)
    }

    pub fn are_rect_and_radii_valid(rect: &Rect, radii: &[Point; 4]) -> bool {
        if !rect.is_finite() || !rect.is_sorted() {
            return false;
        }
        are_radii_predicates_valid(
            radii[Corner::UpperLeft.index()].x,
            radii[Corner::UpperRight.index()].x,
            rect.left,
            rect.right,
        ) && are_radii_predicates_valid(
            radii[Corner::LowerLeft.index()].x,
            radii[Corner::LowerRight.index()].x,
            rect.left,
            rect.right,
        ) && are_radii_predicates_valid(
            radii[Corner::UpperLeft.index()].y,
            radii[Corner::LowerLeft.index()].y,
            rect.top,
            rect.bottom,
        ) && are_radii_predicates_valid(
            radii[Corner::UpperRight.index()].y,
            radii[Corner::LowerRight.index()].y,
            rect.top,
            rect.bottom,
        )
    }

    pub fn inner_bounds(rr: &RRect) -> Rect {
        if rr.is_empty() || rr.is_rect() {
            return *rr.rect();
        }

        let mut inner_bounds = *rr.get_bounds();
        let tl = rr.radii_at(Corner::UpperLeft);
        let tr = rr.radii_at(Corner::UpperRight);
        let bl = rr.radii_at(Corner::LowerLeft);
        let br = rr.radii_at(Corner::LowerRight);

        let left_shift = tl.x.max(bl.x);
        let top_shift = tl.y.max(tr.y);
        let right_shift = tr.x.max(br.x);
        let bottom_shift = bl.y.max(br.y);

        let dw = left_shift + right_shift;
        let dh = top_shift + bottom_shift;

        let horiz_area = (inner_bounds.width() - dw) * inner_bounds.height();
        let vert_area = (inner_bounds.height() - dh) * inner_bounds.width();
        let k_scale = (1.0 - crate::scalar::SCALAR_ROOT2_OVER2) + 1.0e-5;
        let inner_area =
            (inner_bounds.width() - k_scale * dw) * (inner_bounds.height() - k_scale * dh);

        if horiz_area > vert_area && horiz_area > inner_area {
            inner_bounds.left += left_shift;
            inner_bounds.right -= right_shift;
        } else if vert_area > inner_area {
            inner_bounds.top += top_shift;
            inner_bounds.bottom -= bottom_shift;
        } else if inner_area > 0.0 {
            inner_bounds.left += k_scale * left_shift;
            inner_bounds.right -= k_scale * right_shift;
            inner_bounds.top += k_scale * top_shift;
            inner_bounds.bottom -= k_scale * bottom_shift;
        } else {
            return Rect::make_empty();
        }

        debug_assert!(inner_bounds.is_sorted() && !inner_bounds.is_empty());
        inner_bounds
    }

    pub fn read_from_buffer(buffer: &mut SkRBuffer, rr: &mut RRect) -> bool {
        if buffer.available() < RRect::K_SIZE_IN_MEMORY {
            return false;
        }
        let Some(bytes) = buffer.read(RRect::K_SIZE_IN_MEMORY) else {
            return false;
        };
        rr.read_from_memory(bytes) == RRect::K_SIZE_IN_MEMORY
    }

    pub fn write_to_buffer(rr: &RRect, buffer: &mut SkWBuffer) -> bool {
        let mut storage = [0u8; RRect::K_SIZE_IN_MEMORY];
        rr.write_to_memory(&mut storage);
        buffer.write(&storage)
    }
}

use crate::scalar::scalar_nearly_equal_within;

#[cfg(test)]
mod tests {
    #![allow(clippy::excessive_precision)]

    use super::*;

    #[test]
    fn dev_rrect_rect_oval_and_radii() {
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let rr = RRect::make_rect_xy(&rect, 10.0, 10.0);
        assert!(rr.is_simple());
        assert!(rr.is_valid());
        assert_eq!(rr.get_simple_radii(), Point::new(10.0, 10.0));

        let oval = RRect::make_oval(&Rect::new(0.0, 0.0, 100.0, 50.0));
        assert!(oval.is_oval());
        assert!(!RRectPriv::is_circle(&oval));
    }

    #[test]
    fn dev_rrect_serialization_round_trip() {
        let rr = RRect::make_rect_xy(&Rect::new(1.0, 2.0, 30.0, 40.0), 5.0, 6.0);
        let mut bytes = [0u8; RRect::K_SIZE_IN_MEMORY];
        assert_eq!(rr.write_to_memory(&mut bytes), RRect::K_SIZE_IN_MEMORY);
        let mut restored = RRect::make_empty();
        assert_eq!(restored.read_from_memory(&bytes), RRect::K_SIZE_IN_MEMORY);
        assert_eq!(rr, restored);
    }

    #[test]
    fn dev_rrect_fuzzer_regressions() {
        let buffers: [[u8; RRect::K_SIZE_IN_MEMORY]; 3] = [
            [
                0x0a, 0x00, 0x00, 0xff, 0x00, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7f, 0x7f, 0x7f,
                0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f,
                0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x02, 0x00, 0x00,
                0x00, 0x00, 0x20, 0x00, 0x02, 0x00,
            ],
            [
                0x5d, 0xff, 0xff, 0x5d, 0x0a, 0x60, 0x0a, 0x0a, 0x0a, 0x7e, 0x0a, 0x5a, 0x0a, 0x12,
                0x3a, 0x3a, 0x3a, 0x3a, 0x3a, 0x3a, 0x3a, 0x3a, 0x3a, 0x3a, 0x3a, 0x3a, 0x3a, 0x3a,
                0x3a, 0x3a, 0x3a, 0x3a, 0x00, 0x00, 0x00, 0x0a, 0x0a, 0x0a, 0x0a, 0x26, 0x0a, 0x0a,
                0x0a, 0x0a, 0xff, 0xff, 0x0a, 0x0a,
            ],
            [
                0xfe, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0e, 0x04, 0xdd, 0xdd, 0x15, 0xfe, 0x00,
                0x00, 0x04, 0x05, 0x7e, 0x00, 0x00, 0x00, 0xff, 0x08, 0x04, 0xff, 0xff, 0xfe, 0xfe,
                0xff, 0x32, 0x32, 0x32, 0x32, 0x00, 0x32, 0x32, 0x04, 0xdd, 0x3d, 0x1c, 0xfe, 0x89,
                0x04, 0x0a, 0x0e, 0x05, 0x7e, 0x0a,
            ],
        ];
        for buffer in &buffers {
            let mut rr = RRect::new();
            assert_eq!(rr.read_from_memory(buffer), RRect::K_SIZE_IN_MEMORY);
        }
    }

    #[test]
    fn dev_rrect_regressions() {
        let val_a = f32::from_bits(0x4b4b4b4a);
        let val_b = f32::from_bits(0x4b4b4b4b);
        let rect = Rect::new(0.0, 0.0, val_b, val_b);
        assert!(RRect::make_rect_xy(&rect, val_a, val_a).is_valid());
        let mut rr = RRect::new();
        rr.set_nine_patch(&rect, val_a, val_a, val_a, val_a);
        assert!(rr.is_valid());
        assert!(RRect::make_rect_radii(&rect, &[Point::new(val_a, val_a); 4]).is_valid());

        let width = 37.408840f32;
        let height = 73.093933f32;
        let oval = RRect::make_oval(&Rect::new(0.0, 0.0, width, height));
        let offset = oval.make_offset(-5430.867188, -26967.046875);
        assert!(offset.is_valid());
        assert!(offset.is_oval());

        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let invalid = [
            [
                Point::new(0.0, 0.0),
                Point::new(0.0, 60.0),
                Point::new(0.0, 60.0),
                Point::new(0.0, 0.0),
            ],
            [
                Point::new(70.0, 0.0),
                Point::new(70.0, 0.0),
                Point::new(0.0, 0.0),
                Point::new(0.0, 0.0),
            ],
            [
                Point::new(0.0, 0.0),
                Point::new(0.0, 0.0),
                Point::new(60.0, 0.0),
                Point::new(60.0, 0.0),
            ],
            [
                Point::new(0.0, 55.0),
                Point::new(0.0, 0.0),
                Point::new(0.0, 0.0),
                Point::new(0.0, 55.0),
            ],
            [
                Point::new(-5.0, 0.0),
                Point::new(0.0, 0.0),
                Point::new(0.0, 0.0),
                Point::new(0.0, 0.0),
            ],
            [
                Point::new(0.0, -5.0),
                Point::new(0.0, 0.0),
                Point::new(0.0, 0.0),
                Point::new(0.0, 0.0),
            ],
        ];
        for radii in &invalid {
            assert!(!RRectPriv::are_rect_and_radii_valid(&rect, radii));
        }

        let mut scaled = RRect::new();
        scaled.set_rect_radii(&rect, &invalid[0]);
        assert!(scaled.is_valid());

        let fuzzed_rect = Rect::new(1.35631564e-19, 1.35631564e-19, 163968.5, 800.501953);
        let fuzzed_radii = [
            Point::new(1.35631564e-19, 1.35631564e-19),
            Point::new(1.35631564e-19, 640.501953),
            Point::new(1.35631564e-19, 640.501953),
            Point::new(1.35631564e-19, 1.35631564e-19),
        ];
        assert!(!RRectPriv::are_rect_and_radii_valid(
            &fuzzed_rect,
            &fuzzed_radii
        ));
    }

    #[test]
    fn dev_rrect_inner_bounds_and_validation() {
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let rr = RRect::make_rect_xy(&rect, 20.0, 20.0);
        let inner = RRectPriv::inner_bounds(&rr);
        assert!(!inner.is_empty());
        let radii = [
            Point::new(0.0, 0.0),
            Point::new(0.0, 60.0),
            Point::new(0.0, 60.0),
            Point::new(0.0, 0.0),
        ];
        assert!(!RRectPriv::are_rect_and_radii_valid(&rect, &radii));
    }
}
