use crate::floating_point::double_to_float;
use crate::point::Point;
use crate::sk_cubics::SkCubics;
use crate::sk_quads::SkQuads;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct DPoint {
    x: f64,
    y: f64,
}

impl DPoint {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn from_point(point: Point) -> Self {
        Self {
            x: point.x as f64,
            y: point.y as f64,
        }
    }
}

impl core::ops::Neg for DPoint {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

impl core::ops::Add for DPoint {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl core::ops::Sub for DPoint {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl core::ops::Mul<DPoint> for f64 {
    type Output = DPoint;

    fn mul(self, rhs: DPoint) -> Self::Output {
        DPoint::new(self * rhs.x, self * rhs.y)
    }
}

pub struct SkBezierCubic;

impl SkBezierCubic {
    pub fn eval_at(curve: &[f64; 8], t: f64) -> [f64; 2] {
        let in_x = |n: usize| curve[2 * n];
        let in_y = |n: usize| curve[2 * n + 1];
        if t == 0.0 {
            return [in_x(0), in_y(0)];
        }
        if t == 1.0 {
            return [in_x(3), in_y(3)];
        }
        let one_minus_t = 1.0 - t;
        let one_minus_t_squared = one_minus_t * one_minus_t;
        let a = one_minus_t_squared * one_minus_t;
        let b = 3.0 * one_minus_t_squared * t;
        let t_squared = t * t;
        let c = 3.0 * one_minus_t * t_squared;
        let d = t_squared * t;
        [
            a * in_x(0) + b * in_x(1) + c * in_x(2) + d * in_x(3),
            a * in_y(0) + b * in_y(1) + c * in_y(2) + d * in_y(3),
        ]
    }

    pub fn subdivide(curve: &[f64; 8], t: f64, two_curves: &mut [f64; 14]) {
        debug_assert!((0.0..=1.0).contains(&t));
        let in_x = |n: usize| curve[2 * n];
        let in_y = |n: usize| curve[2 * n + 1];
        two_curves[0] = in_x(0);
        two_curves[1] = in_y(0);
        two_curves[12] = in_x(3);
        two_curves[13] = in_y(3);
        let x01 = interpolate(in_x(0), in_x(1), t);
        let y01 = interpolate(in_y(0), in_y(1), t);
        let x12 = interpolate(in_x(1), in_x(2), t);
        let y12 = interpolate(in_y(1), in_y(2), t);
        let x23 = interpolate(in_x(2), in_x(3), t);
        let y23 = interpolate(in_y(2), in_y(3), t);
        two_curves[2] = x01;
        two_curves[3] = y01;
        two_curves[10] = x23;
        two_curves[11] = y23;
        let ax2 = interpolate(x01, x12, t);
        let ay2 = interpolate(y01, y12, t);
        two_curves[4] = ax2;
        two_curves[5] = ay2;
        let bx1 = interpolate(x12, x23, t);
        let by1 = interpolate(y12, y23, t);
        two_curves[8] = bx1;
        two_curves[9] = by1;
        two_curves[6] = interpolate(ax2, bx1, t);
        two_curves[7] = interpolate(ay2, by1, t);
    }

    pub fn convert_to_polynomial(curve: &[f64; 8], y_values: bool) -> [f64; 4] {
        let offset_curve = if y_values { 1 } else { 0 };
        let p = |n: usize| curve[offset_curve + 2 * n];
        [
            -p(0) + 3.0 * p(1) - 3.0 * p(2) + p(3),
            3.0 * p(0) - 6.0 * p(1) + 3.0 * p(2),
            -3.0 * p(0) + 3.0 * p(1),
            p(0),
        ]
    }

    pub fn intersect_with_horizontal_line(control_points: &[Point], y_intercept: f32) -> Vec<f32> {
        assert!(control_points.len() >= 4);
        let p0 = DPoint::from_point(control_points[0]);
        let p1 = DPoint::from_point(control_points[1]);
        let p2 = DPoint::from_point(control_points[2]);
        let p3 = DPoint::from_point(control_points[3]);
        let a = -p0 + 3.0 * p1 - 3.0 * p2 + p3;
        let b = 3.0 * p0 - 6.0 * p1 + 3.0 * p2;
        let c = -3.0 * p0 + 3.0 * p1;
        let d = p0;
        Self::intersect(a.x, b.x, c.x, d.x, a.y, b.y, c.y, d.y, y_intercept)
    }

    #[allow(clippy::too_many_arguments)] // mirrors the reference's 9-parameter entry point
    pub fn intersect(
        ax: f64,
        bx: f64,
        cx: f64,
        dx: f64,
        ay: f64,
        by: f64,
        cy: f64,
        dy: f64,
        to_intersect: f32,
    ) -> Vec<f32> {
        let mut roots = [0.0; 3];
        let count = SkCubics::roots_real(ay, by, cy, dy - to_intersect as f64, &mut roots);
        let mut intersections = Vec::new();
        for root in roots.iter().take(count) {
            let pinned_t = pin_t_range(*root);
            if (0.0..=1.0).contains(&pinned_t) {
                intersections.push(SkCubics::eval_at(ax, bx, cx, dx, pinned_t) as f32);
            }
        }
        intersections
    }
}

pub struct SkBezierQuad;

impl SkBezierQuad {
    pub fn intersect_with_horizontal_line(control_points: &[Point], y_intercept: f32) -> Vec<f32> {
        assert!(control_points.len() >= 3);
        let p0 = DPoint::from_point(control_points[0]);
        let p1 = DPoint::from_point(control_points[1]);
        let p2 = DPoint::from_point(control_points[2]);
        let a = p0 - 2.0 * p1 + p2;
        let b = p0 - p1;
        let c = p0;
        Self::intersect(a.x, b.x, c.x, a.y, b.y, c.y, y_intercept as f64)
    }

    pub fn intersect(
        ax: f64,
        bx: f64,
        cx: f64,
        ay: f64,
        by: f64,
        cy: f64,
        y_intercept: f64,
    ) -> Vec<f32> {
        let root_result = SkQuads::roots(ay, by, cy - y_intercept);
        let mut intersections = Vec::new();
        let t0 = pin_t_range(root_result.root0);
        if (0.0..=1.0).contains(&t0) {
            intersections.push(SkQuads::eval_at(ax, -2.0 * bx, cx, t0) as f32);
        }
        let t1 = pin_t_range(root_result.root1);
        if (0.0..=1.0).contains(&t1) && t1 != t0 {
            intersections.push(SkQuads::eval_at(ax, -2.0 * bx, cx, t1) as f32);
        }
        intersections
    }
}

fn interpolate(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn pin_t_range(t: f64) -> f64 {
    if double_to_float(t + 1.0) == 1.0f32 {
        0.0
    } else if double_to_float(t) == 1.0f32 {
        1.0
    } else {
        t
    }
}
