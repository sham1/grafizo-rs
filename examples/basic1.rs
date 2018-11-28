extern crate colorbuf;
extern crate grafizo;

use grafizo::path::{Path, Loop};

extern crate png;

use png::HasParameters;

use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

fn main() {
    let background = colorbuf::Color {
        r: 0.0f32,
        g: 0.0f32,
        b: 0.0f32,
        a: 1.0f32,
    };
    let foreground = colorbuf::Color {
        r: 1.0f32,
        g: 1.0f32,
        b: 1.0f32,
        a: 1.0f32,
    };
    let foreground2 = colorbuf::Color {
        r: 1.0f32,
        g: 1.0f32,
        b: 1.0f32,
        a: 1.0f32,
    };
    let foreground3 = colorbuf::Color {
        r: 1.0f32,
        g: 1.0f32,
        b: 1.0f32,
        a: 1.0f32,
    };
    let foreground4 = colorbuf::Color {
        r: 1.0f32,
        g: 1.0f32,
        b: 1.0f32,
        a: 1.0f32,
    };
    let mut canvas = grafizo::Canvas::new(800, 600, background);
    canvas.set_draw_color(foreground);

    let begin = grafizo::vector::Point2::new(100f32, 200f32);
    let end = grafizo::vector::Point2::new(500f32, 300f32);

    let line = grafizo::path::Line::new(begin, end);
    line.stroke(&mut canvas, 10f32);

    canvas.set_draw_color(foreground2);

    let begin = grafizo::vector::Point2::new(100f32, 400f32);
    let control = grafizo::vector::Point2::new(125f32, 300f32);
    let end = grafizo::vector::Point2::new(150f32, 400f32);

    let curve = grafizo::path::QuadBezierCurve::new(begin, control, end);
    curve.stroke(&mut canvas, 10f32);

    canvas.set_draw_color(foreground3);

    let stroked_circle = grafizo::path::Circle::new(grafizo::vector::Point2::new(200f32, 300f32), 10f32);
    stroked_circle.stroke(&mut canvas, 5f32);

    canvas.set_draw_color(foreground4);

    let filled_circle = grafizo::path::Circle::new(grafizo::vector::Point2::new(400f32, 300f32), 10f32);
    filled_circle.fill(&mut canvas);

    let mut buf = [0xFFu8; 800 * 600 * 4];

    let mut stride = 0;
    canvas
        .to_bytebuffer(
            &mut buf[..],
            colorbuf::bitmap::ColorFormat::RGBA,
            colorbuf::bitmap::BitDepth::Eight,
            &mut stride,
        )
        .unwrap();

    // Make the data into a PNG
    let mut curr_path: PathBuf = std::env::current_dir().expect("Couldn't get current directory");
    curr_path.push(r"basic1.png");
    let path = curr_path.as_path();
    let file = File::create(path).unwrap();

    let ref mut file_writer = BufWriter::new(file);

    let mut encoder = png::Encoder::new(file_writer, 800, 600);
    encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
    let mut png_writer = encoder.write_header().unwrap();

    png_writer.write_image_data(&buf).unwrap();
}
