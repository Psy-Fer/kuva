//! Fonts whose metrics or outlines are thousands of em, and PNG canvases far
//! larger than any figure. Layout used the font's advances and line heights as
//! given, so a crafted font made the auto-sized canvas, and with it the PNG
//! buffer, many gigabytes; a huge outline made fontdue allocate a bitmap of
//! that size for every glyph drawn.
#![cfg(all(feature = "custom_fonts", feature = "png"))]

#[path = "../src/render/text_metrics/test_font.rs"]
mod font_builder;
mod hostile;

use font_builder::{cmap, font_spec, format12, Spec};
use kuva::backend::raster::{RasterBackend, MAX_RASTER_PIXELS};
use kuva::fonts;
use kuva::plot::BarPlot;
use kuva::render::render::{render_multiple, Scene};
use kuva::render::{layout::Layout, plots::Plot};

/// Printable ASCII maps to glyph 1 (one group per code, since a group maps
/// consecutive codes to consecutive glyphs).
fn ascii_cmap() -> Vec<u8> {
    let groups: Vec<(u32, u32, u32)> = (0x20..=0x7E).map(|c| (c, c, 1)).collect();
    cmap(1, format12(&groups))
}

fn bar_scene(family: &str) -> Scene {
    let plots = vec![Plot::Bar(
        BarPlot::new().with_bar("alpha", 3.0).with_bar("beta", 5.0),
    )];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("A title")
        .with_y_label("Rotated label")
        .with_font_family(family);
    render_multiple(plots, layout)
}

#[test]
fn huge_font_metrics_give_an_ordinary_canvas() {
    // 16 units per em, every advance and vertical metric at its maximum:
    // thousands of em.
    let bytes = font_spec(
        "Kuva Huge Metrics",
        ascii_cmap(),
        &Spec {
            units_per_em: 16,
            vmetrics: (i16::MAX, i16::MIN, i16::MAX),
            advances: vec![u16::MAX, u16::MAX],
            ..Spec::default()
        },
    );
    fonts::register_font_bytes(bytes).unwrap();
    let (w, h, png) = hostile::bounded(64 << 20, || {
        let scene = bar_scene("Kuva Huge Metrics");
        let png = RasterBackend::new().render_scene(&scene);
        (scene.width, scene.height, png)
    });
    // Each character is at most 4 em wide and each line at most 4 em tall.
    assert!(w < 4_000.0 && h < 4_000.0, "{w} x {h}");
    assert!(png.unwrap().starts_with(b"\x89PNG"));
}

#[test]
fn huge_glyph_outlines_are_not_drawn() {
    // Glyph 1 is a square 30,000 units on a side at 16 units per em: about
    // 45,000 px at 24 px, a 2 GB bitmap per size drawn.
    let bytes = font_spec(
        "Kuva Huge Glyph",
        ascii_cmap(),
        &Spec {
            units_per_em: 16,
            advances: vec![8, 8],
            square: Some(30_000),
            ..Spec::default()
        },
    );
    fonts::register_font_bytes(bytes).unwrap();
    let png = hostile::bounded(64 << 20, || {
        RasterBackend::new().render_scene(&bar_scene("Kuva Huge Glyph"))
    });
    assert!(png.unwrap().starts_with(b"\x89PNG"));
}

#[test]
fn ordinary_glyph_outlines_are_drawn() {
    // A 700-unit square glyph draws: the PNG differs from the same font with
    // an empty glyph 1, which has the same metrics and so the same layout.
    let spec = |square| Spec {
        units_per_em: 1000,
        advances: vec![600, 600],
        square,
        ..Spec::default()
    };
    fonts::register_font_bytes(font_spec(
        "Kuva Square Glyph",
        ascii_cmap(),
        &spec(Some(700)),
    ))
    .unwrap();
    fonts::register_font_bytes(font_spec("Kuva Empty Glyph", ascii_cmap(), &spec(None))).unwrap();
    let square = bar_scene("Kuva Square Glyph");
    let empty = bar_scene("Kuva Empty Glyph");
    assert_eq!((square.width, square.height), (empty.width, empty.height));
    let backend = RasterBackend::new();
    assert_ne!(
        backend.render_scene(&square).unwrap(),
        backend.render_scene(&empty).unwrap()
    );
}

#[test]
fn rotated_text_far_longer_than_the_canvas_is_not_drawn() {
    // A y-axis label of a million characters: its unrotated offscreen buffer
    // would be about 2 x 10^7 by 40 px, over 3 GB of RGBA.
    let png = hostile::bounded(64 << 20, || {
        let plots = vec![Plot::Bar(BarPlot::new().with_bar("a", 1.0))];
        let layout = Layout::auto_from_plots(&plots)
            .with_width(600.0)
            .with_height(400.0)
            .with_y_label("W".repeat(1_000_000));
        RasterBackend::new().render_scene(&render_multiple(plots, layout))
    });
    assert!(png.unwrap().starts_with(b"\x89PNG"));
}

#[test]
fn an_oversized_canvas_is_an_error() {
    let err = hostile::bounded(64 << 20, || {
        // 40,000 x 40,000 px at scale 2: 6.4 GB of RGBA.
        RasterBackend::new().render_scene(&Scene::new(20_000.0, 20_000.0))
    })
    .unwrap_err();
    assert!(err.contains(&MAX_RASTER_PIXELS.to_string()), "{err}");

    // One pixel over the limit, and non-finite sizes.
    let side = 1u64 << 14;
    let scene = Scene::new(side as f64 + 1.0, side as f64);
    assert!(RasterBackend::new()
        .with_scale(1.0)
        .render_scene(&scene)
        .is_err());
    for (w, h) in [(f64::INFINITY, 10.0), (10.0, f64::NAN)] {
        assert!(RasterBackend::new()
            .render_scene(&Scene::new(w, h))
            .is_err());
    }
}
