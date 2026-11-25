//! Lines and curves at huge, infinite or NaN coordinates must not hang the PNG
//! or terminal backends. Each used to step once per pixel (or braille dot)
//! between the endpoints, a dash pattern finer than f32 can resolve never
//! advanced, and curve tessellation recursed without end on non-finite points.

#![cfg(feature = "png")]

mod hostile;

use hostile::bounded;
use kuva::backend::png::PngBackend;
use kuva::backend::terminal::TerminalBackend;
use kuva::render::render::{PathData, Primitive, Scene};

const FAR: [f64; 6] = [1e9, -1e9, 1e30, f64::MAX, f64::INFINITY, f64::NAN];

fn line(x1: f64, y1: f64, x2: f64, y2: f64, width: f64, dash: Option<&str>) -> Primitive {
    Primitive::Line {
        x1,
        y1,
        x2,
        y2,
        stroke: "black".into(),
        stroke_width: width,
        stroke_dasharray: dash.map(str::to_string),
    }
}

fn path(d: String, fill: bool, dash: Option<&str>) -> Primitive {
    Primitive::Path(Box::new(PathData {
        d,
        fill: fill.then(|| "steelblue".into()),
        stroke: "black".into(),
        stroke_width: 1.0,
        opacity: None,
        stroke_dasharray: dash.map(str::to_string),
    }))
}

/// Render `prims` on a 200x150 scene with both backends, on a small stack.
fn render_both(prims: Vec<Primitive>) {
    bounded(1 << 20, move || {
        let mut scene = Scene::new(200.0, 150.0);
        for p in prims {
            scene.add(p);
        }
        let png = PngBackend::new().render_scene(&scene).unwrap();
        assert!(!png.is_empty());
        let term = TerminalBackend::new(80, 24).render_scene(&scene);
        assert!(!term.is_empty());
    });
}

#[test]
fn lines_to_far_off_coordinates_render() {
    for v in FAR {
        for (width, dash) in [
            (1.0, None),
            (3.0, None),
            (1.0, Some("4 2")),
            (3.0, Some("4 2 1")),
        ] {
            render_both(vec![
                line(10.0, 10.0, v, 100.0, width, dash),
                line(10.0, 10.0, 100.0, v, width, dash),
                line(-v, -v, v, v, width, dash),
                line(0.0, 75.0, v, 75.0, width, dash),
                line(100.0, 0.0, 100.0, v, width, dash),
            ]);
        }
    }
}

#[test]
fn dash_patterns_finer_than_f32_can_step_render() {
    for dash in ["1e-30", "1e-30 1e-30", "0.0001", "1e-9 5"] {
        render_both(vec![
            line(10.0, 10.0, 190.0, 140.0, 1.0, Some(dash)),
            line(10.0, 10.0, 190.0, 140.0, 3.0, Some(dash)),
            path("M 10 10 L 190 140 L 10 140".to_string(), false, Some(dash)),
        ]);
    }
}

#[test]
fn curves_with_far_off_or_non_finite_control_points_render() {
    for v in FAR {
        for fill in [false, true] {
            render_both(vec![
                path(format!("M 10 10 C {v} 10 10 {v} 190 140"), fill, None),
                path(format!("M 10 10 C 50 50 100 100 {v} {v} Z"), fill, None),
                path(
                    format!("M {v} 10 C 10 10 10 10 100 100 L 10 140 Z"),
                    fill,
                    None,
                ),
                path(
                    format!("M 10 10 C {v} {v} {v} {v} 190 140"),
                    fill,
                    Some("4 2"),
                ),
            ]);
        }
    }
}

#[test]
fn many_thin_lines_to_far_off_coordinates_render() {
    // Thin lines used to take one step per pixel column out to the far end,
    // about 2e9 steps each here. Forty of them in one scene takes minutes
    // without clipping and milliseconds with it. `PngBackend` draws at scale 2,
    // so a 0.5 wide line is 1 px, which is drawn as a thin (Wu) line.
    let mut prims = Vec::new();
    for i in 0..20 {
        let y = i as f64 * 7.0;
        prims.push(line(10.0, y, 1e9, y + 3.0, 0.5, None));
        prims.push(line(10.0, y, 1e9, y, 0.5, None));
    }
    render_both(prims);
}
