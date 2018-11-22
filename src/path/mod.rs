use crate::Canvas;

use crate::vector::{Point2, Vector2};

pub trait Path {
    fn stroke(&self, c: &mut Canvas, width: f32);
}

pub struct OpenMultiPath {
    parts: Vec<Box<Curve>>,
}

pub struct ClosedMultiPath {
    parts: Vec<Box<Curve>>,
}

/// This trait represents a curve defined based on a parametric function.
///
/// A curve in grafizo is defined as a parametric function which gets lineraly
/// interpreted on between two or more points.
pub trait Curve: Path {
    /// This function tells us how long the curve
    /// thinks it is at a maximum.
    ///
    /// This can be used for deciding how many steps are used
    /// to approximate the function.
    ///
    /// Note: The actual length of the curve may not match this.
    fn approximate_length(&self) -> f32;
    fn get_point(&self, t: f32) -> [f32; 2];
    fn derivative(&self, t: f32) -> [f32; 2];
}

pub struct Line {
    p0: Point2,
    p1: Point2,
}

impl Line {
    pub fn new(p0: Point2, p1: Point2) -> Line {
        Line { p0, p1 }
    }
}

impl Path for Line {
    fn stroke(&self, c: &mut Canvas, width: f32) {
        let half_width = width / 2f32;

        let vec = self.p1 - self.p0;
        let norm = Vector2::new(vec.get_y(), -vec.get_x());
        let norm = norm.unit() * half_width;

        // Now to create the rectangle that is our actual "thick line".
        let p1 = self.p0 - norm;
        let p2 = self.p1 - norm;
        let p3 = self.p1 + norm;
        let p4 = self.p0 + norm;

        c.rasterize_filled_rectangle(p1, p2, p3, p4);
    }
}

impl Curve for Line {
    fn approximate_length(&self) -> f32 {
        (square(self.p1.get_x() - self.p0.get_x()) + square(self.p1.get_y() - self.p0.get_y()))
            .sqrt()
    }

    fn get_point(&self, t: f32) -> [f32; 2] {
        let x = (1f32 - t) * self.p0.get_x() + t * self.p1.get_x();
        let y = (1f32 - t) * self.p0.get_y() + t * self.p1.get_y();

        [x, y]
    }

    fn derivative(&self, _: f32) -> [f32; 2] {
        let dx = -self.p0.get_x() + self.p1.get_x();
        let dy = -self.p0.get_y() + self.p1.get_y();

        [dx, dy]
    }
}

pub struct QuadBezierCurve {
    p0: [f32; 2],
    p1: [f32; 2],
    p2: [f32; 2],
}

impl QuadBezierCurve {
    pub fn new(begin: Point2, control: Point2, end: Point2) -> QuadBezierCurve {
        QuadBezierCurve {
            p0: [ begin.get_x(), begin.get_y() ],
            p1: [ control.get_x(), control.get_y() ],
            p2: [ end.get_x(), end.get_y() ],
        }
    }
}

impl Path for QuadBezierCurve {
    fn stroke(&self, c: &mut Canvas, width: f32) {
        let half_width = width / 2f32;

        // We want to use a line-based approximation of
        // our Bezier curve.
        //
        // So for that to work we need to know how many line
        // segments we want to have. We are going to use a hyperbola
        // so we get a somewhat linear approximation for the amount
        // of segments needed while having the count be high for
        // low numbers. The particular hyperbola we'll be using is
        // `sqrt(x * x + 100), for x >= 0` (never actually going to be 0).
        let line_segment_hyperbola = |length: f32| (((length * length) + 100f32).sqrt() + 1f32) as u64;
        let approx_len = self.approximate_length();
        let line_segments = line_segment_hyperbola(approx_len);

        let mut lines: Vec<[Point2; 4]> = Vec::with_capacity(line_segments as usize);
        let prev_pos = 0f32;
        
        let prev_point = self.get_point(prev_pos);
        let prev_x: f32 = prev_point[0];
        let prev_y: f32 = prev_point[1];
        let mut prev_p = Point2::new(prev_x, prev_y);
        
        let prev_deriv = self.derivative(prev_pos);
        let prev_dx = prev_deriv[0];
        let prev_dy = prev_deriv[1];

        let mut prev_normal = Vector2::new(prev_dy, -prev_dx).unit();
        prev_normal = prev_normal * half_width;

        for i in 0..line_segments {
            let next_pos = ((i + 1) as f32) / (line_segments as f32);
            let next_point = self.get_point(next_pos);
            
            let next_x = next_point[0];
            let next_y = next_point[1];
            let next_p = Point2::new(next_x, next_y);
            
            let next_deriv = self.derivative(next_pos);
            
            let next_dx = next_deriv[0];
            let next_dy = next_deriv[1];
            
            let mut next_normal = Vector2::new(next_dy, -next_dx).unit();
            next_normal = next_normal * half_width;

            let v1 = prev_p - prev_normal;
            let v2 = next_p - next_normal;
            let v3 = next_p + next_normal;
            let v4 = prev_p + prev_normal;

            lines.insert(i as usize, [v1, v2, v3, v4]);

            prev_p = next_p;
            prev_normal = next_normal;
        }

        for [v1, v2, v3, v4] in lines.into_iter() {
            c.rasterize_filled_rectangle(v1, v2, v3, v4);
        }
    }
}

impl Curve for QuadBezierCurve {
    fn approximate_length(&self) -> f32 {
        (square(self.p1[0] - self.p0[0]) + square(self.p1[1] - self.p0[1])).sqrt()
            + (square(self.p2[0] - self.p1[0]) + square(self.p2[1] - self.p1[1])).sqrt()
    }

    fn get_point(&self, t: f32) -> [f32; 2] {
        let x = square(1f32 - t) * self.p0[0]
            + 2f32 * t * (1f32 - t) * self.p1[0]
            + square(t) * self.p2[0];
        let y = square(1f32 - t) * self.p0[1]
            + 2f32 * t * (1f32 - t) * self.p1[1]
            + square(t) * self.p2[1];
        [x, y]
    }

    fn derivative(&self, t: f32) -> [f32; 2] {
        let dx = -2f32 * self.p0[0] * (1f32 - t)
            + 2f32 * self.p1[0] * (1f32 - 2f32 * t)
            + 2f32 * self.p2[0] * t;
        let dy = -2f32 * self.p0[0] * (1f32 - t)
            + 2f32 * self.p1[0] * (1f32 - 2f32 * t)
            + 2f32 * self.p2[0] * t;
        [dx, dy]
    }
}
fn square(x: f32) -> f32 {
    x * x
}
