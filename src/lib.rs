extern crate colorbuf;

use std::collections::HashMap;
use std::result::Result;

use colorbuf::ColorBuf;

pub mod path;
pub mod vector;

use self::vector::{Point2, Vector2};

pub struct CanvasColorBuf {
    buf: HashMap<[u64; 2], colorbuf::Color>,
    width: u64,
    height: u64,
}

impl colorbuf::ColorBuf for CanvasColorBuf {
    fn get_pixel(&self, x: u64, y: u64) -> Result<colorbuf::Color, colorbuf::ColorBufError> {
        if x >= self.width || y >= self.height {
            return Err(colorbuf::ColorBufError::InvalidCoordinate);
        }

        let entry = [x, y];
        Ok(self.buf.get(&entry).unwrap().clone())
    }

    fn set_pixel(
        &mut self,
        x: u64,
        y: u64,
        color: &colorbuf::Color,
    ) -> Result<(), colorbuf::ColorBufError> {
        if x >= self.width || y >= self.height {
            return Err(colorbuf::ColorBufError::InvalidCoordinate);
        }

        let entry = [x, y];
        self.buf.insert(entry, color.clone());
        Ok(())
    }

    fn get_width(&self) -> u64 {
        self.width
    }

    fn get_height(&self) -> u64 {
        self.height
    }
}

impl CanvasColorBuf {
    fn new(width: u64, height: u64, color: colorbuf::Color) -> CanvasColorBuf {
        let mut ret = CanvasColorBuf {
            buf: HashMap::new(),
            width: width,
            height: height,
        };

        for x in 0..width {
            for y in 0..height {
                let entry = [x, y];
                ret.buf.insert(entry, color.clone());
            }
        }

        ret
    }
}

pub struct Canvas {
    backing: CanvasColorBuf,
    current_color: colorbuf::Color,
    antialias_enabled: bool,
}

impl Canvas {
    pub fn new(width: u64, height: u64, color: colorbuf::Color) -> Canvas {
        Canvas {
            backing: CanvasColorBuf::new(width, height, color),
            current_color: color,
            antialias_enabled: true,
        }
    }

    pub fn set_draw_color(&mut self, new_color: colorbuf::Color) {
        self.current_color = new_color;
    }

    pub fn enable_antialias(&mut self, enable: bool) {
        self.antialias_enabled = enable;
    }

    fn rasterize_stroked_circle(&mut self, center: Point2, inner_radius: f32, outer_radius: f32) {
        // Calculate the bounding box of the circle,
        // and round it to be the closest pixels.
        let min_x = ((center.get_x() - outer_radius - 1f32).floor() as i32).max(0);
        let max_x = ((center.get_x() + outer_radius + 1f32).ceil() as i32)
            .min((self.backing.get_width() - 1) as i32);
        let min_y = ((center.get_y() - outer_radius - 1f32).floor() as i32).max(0);
        let max_y = ((center.get_y() + outer_radius + 1f32).ceil() as i32)
            .min((self.backing.get_height() - 1) as i32);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let cur_point = Point2::new(x as f32, y as f32);
                if !self.antialias_enabled {
                    let dist_from_center = (cur_point - center).length();
                    if dist_from_center < inner_radius || dist_from_center > outer_radius {
                        continue;
                    }
                    self.backing
                        .set_pixel(x as u64, y as u64, &self.current_color)
                        .unwrap();
                    continue;
                }

                // Antialiasing
                // We check whether our point is within acceptiable
                // range from the center
                let circle_helper = |x, y| {
                    let p = Point2::new(x, y);
                    let dist = (p - center).length();
                    (dist >= inner_radius) && (dist <= outer_radius)
                };
                // We have antialiasing, so let us check the corners first for some heuristic reasons.
                let x_fac = x as f32;
                let y_fac = y as f32;
                let corner_offsets = [
                    [0f32, 0f32],
                    [3f32 / 4f32, 0f32],
                    [3f32 / 4f32, 3f32 / 4f32],
                    [0f32, 3f32 / 4f32],
                ];
                let corner_locs = corner_offsets
                    .iter()
                    .map(|[xoff, yoff]| [x_fac + xoff, y_fac + yoff])
                    .collect::<Vec<_>>();
                let corners_inside = corner_locs
                    .iter()
                    .map(|&[x, y]| circle_helper(x, y))
                    .collect::<Vec<_>>();
                let is_empty = !(corners_inside.iter().fold(false, |acc, &x| acc || x));
                if is_empty {
                    // No corners touch so we aren't close enough to the circle.
                    // Now, there are some literal edge-cases with this detection method where
                    // this heuristic fails, but for our usage this is accurate enough.
                    continue;
                }
                let is_full = corners_inside.iter().fold(true, |acc, &x| acc && x);
                if is_full {
                    // We are fully contained within the circle edge. While this has similar
                    // shortcomings to the last one, again, this is good enough for us.
                    self.backing
                        .set_pixel(x as u64, y as u64, &self.current_color)
                        .unwrap();
                    continue;
                }
                // We are at a position where some of our subpixels are within the circle
                // and some are without. I.e. we are at a pixel where we should apply
                // anti-aliasing to.

                // TODO: Make the amount of subpixels variable.
                let subpixels_per_side = 16;
                let mut subs_within_polygon =
                    vec![false; subpixels_per_side * subpixels_per_side];
                for y_sub in 0..subpixels_per_side {
                    for x_sub in 0..subpixels_per_side {
                        let x_off = (x_sub as f32) / (subpixels_per_side as f32);
                        let y_off = (x_sub as f32) / (subpixels_per_side as f32);

                        let sub_x = x_fac + x_off;
                        let sub_y = y_fac + y_off;

                        subs_within_polygon[y_sub * subpixels_per_side + x_sub] =
                            circle_helper(sub_x, sub_y);
                    }
                }
                let aa_blend_proportion = subs_within_polygon
                    .into_iter()
                    .fold(0, |acc, x| acc + if x { 1 } else { 0 });
                let blend_factor = (aa_blend_proportion as f32)
                    / ((subpixels_per_side * subpixels_per_side) as f32);
                let blent_color = colorbuf::Color {
                    r: self.current_color.r,
                    g: self.current_color.g,
                    b: self.current_color.b,
                    a: self.current_color.a * blend_factor,
                };

                // TODO: Make gamma changeable
                let gamma = 2.2f32;

                let cur_color = self.backing.get_pixel(x as u64, y as u64).unwrap();

                let out_a = blent_color.a + cur_color.a * (1f32 - blent_color.a);
                let out_r = (blent_color.r.powf(gamma) * blent_color.a
                             + cur_color.r.powf(gamma) * (1f32 - blent_color.a))
                    .powf(1f32 / gamma);
                let out_g = (blent_color.g.powf(gamma) * blent_color.a
                             + cur_color.g.powf(gamma) * (1f32 - blent_color.a))
                    .powf(1f32 / gamma);
                let out_b = (blent_color.b.powf(gamma) * blent_color.a
                             + cur_color.b.powf(gamma) * (1f32 - blent_color.a))
                    .powf(1f32 / gamma);

                let out_color = colorbuf::Color {
                    r: out_r,
                    g: out_g,
                    b: out_b,
                    a: out_a,
                };

                self.backing
                    .set_pixel(x as u64, y as u64, &out_color)
                    .unwrap();
            }
        }
    }

    fn rasterize_filled_circle(&mut self, center: Point2, radius: f32) {}

    fn rasterize_convex_filled_polygon(&mut self, points: &[Point2]) {
        // We must calculate the bounding box of our polygon,
        // and rounding them to the closest integers.
        let xs = points.iter().map(|p| p.get_x()).collect::<Vec<_>>();
        let ys = points.iter().map(|p| p.get_y()).collect::<Vec<_>>();

        let min_x = (helper_get_min(xs.clone()).unwrap().floor() as i32 - 1).max(0);
        let max_x = (helper_get_max(xs).unwrap().ceil() as i32 + 1)
            .min((self.backing.get_width() - 1) as i32);
        let min_y = (helper_get_min(ys.clone()).unwrap().floor() as i32 - 1).max(0);
        let max_y = (helper_get_max(ys).unwrap().ceil() as i32 + 1)
            .min((self.backing.get_height() - 1) as i32);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                // We know that all of these are within the bounding box which limits the necessary
                // checks
                if self.antialias_enabled {
                    let x_fac = x as f32;
                    let y_fac = y as f32;
                    let corner_offsets = [
                        [0f32, 0f32],
                        [3f32 / 4f32, 0f32],
                        [3f32 / 4f32, 3f32 / 4f32],
                        [0f32, 3f32 / 4f32],
                    ];
                    let corner_locs = corner_offsets
                        .iter()
                        .map(|[xoff, yoff]| [x_fac + xoff, y_fac + yoff])
                        .collect::<Vec<_>>();
                    let corners_inside = corner_locs
                        .iter()
                        .map(|&[x, y]| helper_even_odd_rule(x, y, &points[..]))
                        .collect::<Vec<_>>();
                    let is_empty = !(corners_inside.iter().fold(false, |acc, &x| acc || x));
                    if is_empty {
                        // No corners touch so we aren't close enough to the polygon.
                        // Now, there are some literal edge-cases with this detection method where
                        // this heuristic fails, but for our usage this is accurate enough.
                        continue;
                    }
                    let is_full = corners_inside.iter().fold(true, |acc, &x| acc && x);
                    if is_full {
                        // We are fully contained within the polygon. While this has similar
                        // shortcomings to the last one, again, this is good enough for us.
                        self.backing
                            .set_pixel(x as u64, y as u64, &self.current_color)
                            .unwrap();
                        continue;
                    }
                    // We are at a position where some of our subpixels are within the polygon
                    // and some are without. I.e. we are at a pixel where we should apply
                    // anti-aliasing to.

                    // TODO: Make the amount of subpixels variable.
                    let subpixels_per_side = 16;
                    let mut subs_within_polygon =
                        vec![false; subpixels_per_side * subpixels_per_side];
                    for y_sub in 0..subpixels_per_side {
                        for x_sub in 0..subpixels_per_side {
                            let x_off = (x_sub as f32) / (subpixels_per_side as f32);
                            let y_off = (x_sub as f32) / (subpixels_per_side as f32);

                            let sub_x = x_fac + x_off;
                            let sub_y = y_fac + y_off;

                            subs_within_polygon[y_sub * subpixels_per_side + x_sub] =
                                helper_even_odd_rule(sub_x, sub_y, &points[..]);
                        }
                    }
                    let aa_blend_proportion = subs_within_polygon
                        .into_iter()
                        .fold(0, |acc, x| acc + if x { 1 } else { 0 });
                    let blend_factor = (aa_blend_proportion as f32)
                        / ((subpixels_per_side * subpixels_per_side) as f32);
                    let blent_color = colorbuf::Color {
                        r: self.current_color.r,
                        g: self.current_color.g,
                        b: self.current_color.b,
                        a: self.current_color.a * blend_factor,
                    };

                    // TODO: Make gamma changeable
                    let gamma = 2.2f32;

                    let cur_color = self.backing.get_pixel(x as u64, y as u64).unwrap();

                    let out_a = blent_color.a + cur_color.a * (1f32 - blent_color.a);
                    let out_r = (blent_color.r.powf(gamma) * blent_color.a
                        + cur_color.r.powf(gamma) * (1f32 - blent_color.a))
                        .powf(1f32 / gamma);
                    let out_g = (blent_color.g.powf(gamma) * blent_color.a
                        + cur_color.g.powf(gamma) * (1f32 - blent_color.a))
                        .powf(1f32 / gamma);
                    let out_b = (blent_color.b.powf(gamma) * blent_color.a
                        + cur_color.b.powf(gamma) * (1f32 - blent_color.a))
                        .powf(1f32 / gamma);

                    let out_color = colorbuf::Color {
                        r: out_r,
                        g: out_g,
                        b: out_b,
                        a: out_a,
                    };

                    self.backing
                        .set_pixel(x as u64, y as u64, &out_color)
                        .unwrap();
                } else {
                    let inside = helper_even_odd_rule(x as f32, y as f32, &points[..]);
                    if inside {
                        self.backing
                            .set_pixel(x as u64, y as u64, &self.current_color)
                            .unwrap();
                    }
                }
            }
        }
    }

    fn rasterize_filled_rectangle(&mut self, p1: Point2, p2: Point2, p3: Point2, p4: Point2) {
        let points = [p1, p2, p3, p4];
        self.rasterize_convex_filled_polygon(&points[..]);
    }

    pub fn to_bytebuffer(
        self,
        bitmap: &mut [u8],
        format: colorbuf::bitmap::ColorFormat,
        depth: colorbuf::bitmap::BitDepth,
        stride: &mut u64,
    ) -> std::result::Result<(), colorbuf::bitmap::BitmapError> {
        colorbuf::bitmap::to_bitmap(self.backing, format, depth, stride, bitmap)
    }
}

fn helper_even_odd_rule(x: f32, y: f32, points: &[Point2]) -> bool {
    let mut inside = false;
    let mut j = points.len() - 1;
    for (i, _) in points.iter().enumerate() {
        if ((points[i].get_y() > (y)) != (points[j].get_y() > (y)))
            && ((x)
                < (points[j].get_x() - points[i].get_x()) * ((y) - points[i].get_y())
                    / (points[j].get_y() - points[i].get_y())
                    + points[i].get_x())
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn helper_get_min<I, O>(i: I) -> Option<O>
where
    O: std::cmp::PartialOrd + std::clone::Clone,
    I: IntoIterator<Item = O>,
{
    i.into_iter().fold(None, |min, x| match min {
        None => Some(x),
        Some(y) => Some(if x < y { x.clone() } else { y.clone() }),
    })
}

fn helper_get_max<I, O>(i: I) -> Option<O>
where
    O: std::cmp::PartialOrd + std::clone::Clone,
    I: IntoIterator<Item = O>,
{
    i.into_iter().fold(None, |min, x| match min {
        None => Some(x),
        Some(y) => Some(if x > y { x.clone() } else { y.clone() }),
    })
}
