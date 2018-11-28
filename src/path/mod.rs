use crate::Canvas;
use std::collections::VecDeque;

use crate::vector::{Point2, Vector2};

pub trait Path {
    fn stroke(&self, c: &mut Canvas, width: f32);
}

pub trait Loop: Path {
    fn fill(&self, c: &mut Canvas);
}

pub struct OpenMultiPath {
    parts: Vec<Box<Curve>>,
}

pub struct ClosedMultiPath {
    parts: Vec<Box<Curve>>,
}

pub struct Circle {
    center: Point2,
    radius: f32,
}

impl Circle {
    pub fn new(center: Point2, radius: f32) -> Circle {
        Circle { center, radius, }
    }
}

impl Path for Circle {
    fn stroke(&self, c: &mut Canvas, width: f32) {
        let inner_radius = (self.radius) - (width / 2f32);
        let outer_radius = (self.radius) + (width / 2f32);

        c.rasterize_stroked_circle(self.center, inner_radius, outer_radius);
    }
}

impl Loop for Circle {
    fn fill(&self, c: &mut Canvas) {
        c.rasterize_stroked_circle(self.center, 0f32, self.radius);
    }
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
    fn get_point(&self, t: f32) -> Point2;
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

    fn get_point(&self, t: f32) -> Point2 {
        let x = (1f32 - t) * self.p0.get_x() + t * self.p1.get_x();
        let y = (1f32 - t) * self.p0.get_y() + t * self.p1.get_y();

        Point2::new(x, y)
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
            p0: [begin.get_x(), begin.get_y()],
            p1: [control.get_x(), control.get_y()],
            p2: [end.get_x(), end.get_y()],
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
        let line_segment_hyperbola =
            |length: f32| (((length * length) + 100f32).sqrt() + 1f32) as u64;
        let approx_len = self.approximate_length();
        let line_segments = line_segment_hyperbola(approx_len);

        let mut left_edge: Vec<Point2> = Vec::new();
        let mut right_edge: VecDeque<Point2> = VecDeque::new();

        for i in 0..=line_segments {
            let t = (i as f32) / (line_segments as f32);
            let curr_point = self.get_point(t);
            let [dx, dy] = self.derivative(t);
            let norm = Vector2::new(dy, -dx).unit() * half_width;

            let left = curr_point - norm;
            let right = curr_point + norm;

            left_edge.push(left);
            right_edge.push_front(right);
        }

        let point = left_edge.into_iter().chain(right_edge).collect::<Vec<_>>();
        c.rasterize_convex_filled_polygon(&point[..]);
    }
}

impl Curve for QuadBezierCurve {
    fn approximate_length(&self) -> f32 {
        (square(self.p1[0] - self.p0[0]) + square(self.p1[1] - self.p0[1])).sqrt()
            + (square(self.p2[0] - self.p1[0]) + square(self.p2[1] - self.p1[1])).sqrt()
    }

    fn get_point(&self, t: f32) -> Point2 {
        let x = square(1f32 - t) * self.p0[0]
            + 2f32 * t * (1f32 - t) * self.p1[0]
            + square(t) * self.p2[0];
        let y = square(1f32 - t) * self.p0[1]
            + 2f32 * t * (1f32 - t) * self.p1[1]
            + square(t) * self.p2[1];
        Point2::new(x, y)
    }

    fn derivative(&self, t: f32) -> [f32; 2] {
        let dx = (-2f32 + 2f32 * t) * self.p0[0]
            + (2f32 - 4f32 * t) * self.p1[0]
            + 2f32 * t * self.p2[0];
        let dy = (-2f32 + 2f32 * t) * self.p0[1]
            + (2f32 - 4f32 * t) * self.p1[1]
            + 2f32 * t * self.p2[1];
        [dx, dy]
    }
}
fn square(x: f32) -> f32 {
    x * x
}
