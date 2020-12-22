use minifb::*;
use raqote::*;

use hexgrid::*;

const WIDTH: usize = 500;
const HEIGHT: usize = 500;

const HEXSCALE: f32 = 0.1;

fn draw_hex(
    hex: &HexCoord<i16>,
    dt: &mut DrawTarget,
    (width, height): (usize, usize),
    color: &Source,
    style: &StrokeStyle,
    draw_options: &DrawOptions
) {
    let dim = 0.5 * usize::min(width, height) as f32;
    let scale_point = |(x, y): (f32, f32)| (
        (1.0 + x * HEXSCALE) * dim,
        (1.0 + y * HEXSCALE) * dim,
    );

    let coords = hex.cartesian_corners();

    let mut pb = PathBuilder::new();
    let move_to = |pb: &mut PathBuilder, (x, y)| pb.move_to(x, y);
    let line_to = |pb: &mut PathBuilder, (x, y)| pb.line_to(x, y);

    move_to(&mut pb, scale_point(coords[0]));
    for &xy in &coords[1..] {
        line_to(&mut pb, scale_point(xy));
    }
    pb.close();
    let path = pb.finish();
    dt.stroke(&path, color, style, &draw_options);
}

fn main() {
    let mut window = Window::new(
        "Hex Grid",
        WIDTH,
        HEIGHT,
        WindowOptions {..WindowOptions::default()},
    ).unwrap();
    let size = window.get_size();
    let mut dt = DrawTarget::new(WIDTH as i32, HEIGHT as i32);
    let black = Source::Solid(SolidSource::from_unpremultiplied_argb(0xff, 0, 0, 0));
    let white = SolidSource::from_unpremultiplied_argb(0xff, 0xff, 0xff, 0xff);
    let draw_options = DrawOptions::new();
    let stroke_style = StrokeStyle {
        width: 2.0,
        ..StrokeStyle::default()
    };
    
    let hex: HexCoord<i16> = HexCoord::new(0, 0);
    while window.is_open() && !window.is_key_down(Key::Escape) {
        dt.clear(white);
        draw_hex(&hex, &mut dt, size, &black, &stroke_style, &draw_options);
        window.update_with_buffer(dt.get_data(), size.0, size.1).unwrap();
    }
}
