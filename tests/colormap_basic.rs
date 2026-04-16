//! Tests for `ColorMap` — all 38 variants, with emphasis on diverging scales.
//!
//! Tests cover:
//! - `map(t)` produces valid hex color strings at t=0, t=0.5, t=1.
//! - Colors differ across the range (gradient is not degenerate).
//! - `map_rgb(t)` returns `Some` for all non-Custom variants.
//! - Diverging maps are symmetric: the midpoint color (t=0.5) differs from both endpoints.
//! - Integration: `Heatmap` renders valid SVG with each diverging colormap.

use kuva::plot::{ColorMap, Heatmap};
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;

fn outdir() {
    std::fs::create_dir_all("test_outputs").ok();
}

/// A simple 3×3 heatmap with values spanning a range that exercises the full color scale,
/// including negative values to test diverging maps end-to-end.
fn diverging_data() -> Vec<Vec<f64>> {
    vec![
        vec![-3.0, -1.0,  0.0],
        vec![-1.0,  0.0,  1.0],
        vec![ 0.0,  1.0,  3.0],
    ]
}

fn sequential_data() -> Vec<Vec<f64>> {
    vec![
        vec![0.0, 1.0, 2.0],
        vec![3.0, 4.0, 5.0],
        vec![6.0, 7.0, 9.0],
    ]
}

/// Returns the hex color string produced by `map(t)` for the given variant.
fn color_at(cmap: &ColorMap, t: f64) -> String {
    cmap.map(t)
}

/// Assert that a color string looks like a valid #rrggbb hex.
fn assert_hex(color: &str, context: &str) {
    assert!(
        color.starts_with('#') && color.len() == 7,
        "{context}: expected #rrggbb, got {color:?}"
    );
    assert!(
        color[1..].chars().all(|c| c.is_ascii_hexdigit()),
        "{context}: non-hex digit in {color:?}"
    );
}

// ── Diverging variants ────────────────────────────────────────────────────────

/// All 9 diverging ColorMap variants should:
/// 1. Return valid hex strings at t=0, 0.5, 1.
/// 2. Have distinct colors at all three positions (gradient is not flat).
/// 3. Return Some from map_rgb at all positions.
#[test]
fn test_diverging_colormaps_produce_valid_colors() {
    let diverging = [
        ("BrownGreen",    ColorMap::BrownGreen),
        ("PinkGreen",     ColorMap::PinkGreen),
        ("PurpleGreen",   ColorMap::PurpleGreen),
        ("PurpleOrange",  ColorMap::PurpleOrange),
        ("RedBlue",       ColorMap::RedBlue),
        ("RedGrey",       ColorMap::RedGrey),
        ("RedYellowBlue", ColorMap::RedYellowBlue),
        ("RedYellowGreen",ColorMap::RedYellowGreen),
        ("Spectral",      ColorMap::Spectral),
    ];

    for (name, cmap) in &diverging {
        let c0   = color_at(cmap, 0.0);
        let cmid = color_at(cmap, 0.5);
        let c1   = color_at(cmap, 1.0);

        assert_hex(&c0,   &format!("{name} t=0.0"));
        assert_hex(&cmid, &format!("{name} t=0.5"));
        assert_hex(&c1,   &format!("{name} t=1.0"));

        // Gradient must actually change across the range
        assert_ne!(c0, c1,   "{name}: t=0 and t=1 should differ");
        assert_ne!(c0, cmid, "{name}: t=0 and t=0.5 should differ");
        assert_ne!(cmid, c1, "{name}: t=0.5 and t=1 should differ");

        // map_rgb must return Some for non-Custom variants
        assert!(cmap.map_rgb(0.0).is_some(), "{name}: map_rgb(0.0) returned None");
        assert!(cmap.map_rgb(0.5).is_some(), "{name}: map_rgb(0.5) returned None");
        assert!(cmap.map_rgb(1.0).is_some(), "{name}: map_rgb(1.0) returned None");
    }
}

/// For true diverging maps the midpoint (t=0.5) is the neutral color (light/white/grey).
/// It should be perceptually lighter than both extremes — i.e. its R+G+B sum > either endpoint.
#[test]
fn test_diverging_midpoint_is_lighter_than_endpoints() {
    fn brightness(cmap: &ColorMap, t: f64) -> u32 {
        let (r, g, b) = cmap.map_rgb(t).unwrap();
        r as u32 + g as u32 + b as u32
    }

    let diverging = [
        ("BrownGreen",    ColorMap::BrownGreen),
        ("PinkGreen",     ColorMap::PinkGreen),
        ("PurpleGreen",   ColorMap::PurpleGreen),
        ("PurpleOrange",  ColorMap::PurpleOrange),
        ("RedBlue",       ColorMap::RedBlue),
        ("RedGrey",       ColorMap::RedGrey),
        ("RedYellowBlue", ColorMap::RedYellowBlue),
        ("RedYellowGreen",ColorMap::RedYellowGreen),
        ("Spectral",      ColorMap::Spectral),
    ];

    for (name, cmap) in &diverging {
        let b0   = brightness(cmap, 0.0);
        let bmid = brightness(cmap, 0.5);
        let b1   = brightness(cmap, 1.0);
        assert!(
            bmid > b0 && bmid > b1,
            "{name}: midpoint brightness ({bmid}) should exceed both endpoints \
             (t=0: {b0}, t=1: {b1})"
        );
    }
}

// ── map_rgb returns Some for every non-Custom variant ────────────────────────

#[test]
fn test_map_rgb_returns_some_for_all_builtin_variants() {
    let all_builtins: Vec<(&str, ColorMap)> = vec![
        // Sequential perceptual
        ("Turbo",            ColorMap::Turbo),
        ("Viridis",          ColorMap::Viridis),
        ("Inferno",          ColorMap::Inferno),
        ("Magma",            ColorMap::Magma),
        ("Plasma",           ColorMap::Plasma),
        ("Cividis",          ColorMap::Cividis),
        ("Warm",             ColorMap::Warm),
        ("Cool",             ColorMap::Cool),
        ("Cubehelix",        ColorMap::Cubehelix),
        // Sequential ColorBrewer
        ("BlueGreen",        ColorMap::BlueGreen),
        ("BluePurple",       ColorMap::BluePurple),
        ("GreenBlue",        ColorMap::GreenBlue),
        ("OrangeRed",        ColorMap::OrangeRed),
        ("PurpleBlueGreen",  ColorMap::PurpleBlueGreen),
        ("PurpleBlue",       ColorMap::PurpleBlue),
        ("PurpleRed",        ColorMap::PurpleRed),
        ("RedPurple",        ColorMap::RedPurple),
        ("YellowGreenBlue",  ColorMap::YellowGreenBlue),
        ("YellowGreen",      ColorMap::YellowGreen),
        ("YellowOrangeBrown",ColorMap::YellowOrangeBrown),
        ("YellowOrangeRed",  ColorMap::YellowOrangeRed),
        // Sequential single-hue
        ("Blues",            ColorMap::Blues),
        ("Greens",           ColorMap::Greens),
        ("Grayscale",        ColorMap::Grayscale),
        ("Oranges",          ColorMap::Oranges),
        ("Purples",          ColorMap::Purples),
        ("Reds",             ColorMap::Reds),
        // Diverging
        ("BrownGreen",       ColorMap::BrownGreen),
        ("PinkGreen",        ColorMap::PinkGreen),
        ("PurpleGreen",      ColorMap::PurpleGreen),
        ("PurpleOrange",     ColorMap::PurpleOrange),
        ("RedBlue",          ColorMap::RedBlue),
        ("RedGrey",          ColorMap::RedGrey),
        ("RedYellowBlue",    ColorMap::RedYellowBlue),
        ("RedYellowGreen",   ColorMap::RedYellowGreen),
        ("Spectral",         ColorMap::Spectral),
        // Cyclical
        ("Rainbow",          ColorMap::Rainbow),
        ("Sinebow",          ColorMap::Sinebow),
    ];

    for (name, cmap) in &all_builtins {
        for &t in &[0.0_f64, 0.25, 0.5, 0.75, 1.0] {
            let rgb = cmap.map_rgb(t);
            assert!(
                rgb.is_some(),
                "{name}: map_rgb({t}) returned None"
            );
            // map_rgb and map should agree on the color
            let (r, g, b) = rgb.unwrap();
            let hex = cmap.map(t);
            assert_hex(&hex, &format!("{name} t={t}"));
            let expected = format!("#{r:02x}{g:02x}{b:02x}");
            assert_eq!(hex, expected, "{name} t={t}: map() and map_rgb() disagree");
        }
    }
}

/// Custom colormap must return None from map_rgb.
#[test]
fn test_custom_map_rgb_returns_none() {
    use std::sync::Arc;
    let cmap = ColorMap::Custom(Arc::new(|t: f64| format!("#{:02x}0000", (t * 255.0) as u8)));
    assert!(cmap.map_rgb(0.5).is_none());
    assert_hex(&cmap.map(0.5), "Custom t=0.5");
}

// ── Clamping: values outside [0, 1] must not panic ───────────────────────────

#[test]
fn test_map_clamps_out_of_range_values() {
    let diverging_cmaps = [
        ColorMap::BrownGreen, ColorMap::Spectral, ColorMap::RedBlue,
    ];
    for cmap in &diverging_cmaps {
        // Should not panic
        let _ = cmap.map(-0.5);
        let _ = cmap.map(1.5);
        let _ = cmap.map(f64::NAN);
        let _ = cmap.map_rgb(-1.0);
        let _ = cmap.map_rgb(2.0);
    }
}

// ── Heatmap integration: each diverging map renders valid SVG ─────────────────

macro_rules! heatmap_diverging_test {
    ($fn_name:ident, $variant:expr, $filename:literal) => {
        #[test]
        fn $fn_name() {
            outdir();
            let heatmap = Heatmap::new()
                .with_data(diverging_data())
                .with_color_map($variant);
            let plots = vec![Plot::Heatmap(heatmap)];
            let layout = Layout::auto_from_plots(&plots)
                .with_title(concat!("Heatmap — ", $filename))
                .with_x_categories(vec!["A".into(), "B".into(), "C".into()])
                .with_y_categories(vec!["X".into(), "Y".into(), "Z".into()]);
            let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
            std::fs::write(concat!("test_outputs/colormap_", $filename, ".svg"), &svg).unwrap();
            assert!(svg.starts_with("<svg"), "{} SVG must start with <svg", $filename);
            assert!(svg.contains("<rect"), "{} SVG must contain rect elements", $filename);
        }
    };
}

heatmap_diverging_test!(test_heatmap_brown_green,     ColorMap::BrownGreen,     "brown_green");
heatmap_diverging_test!(test_heatmap_pink_green,      ColorMap::PinkGreen,      "pink_green");
heatmap_diverging_test!(test_heatmap_purple_green,    ColorMap::PurpleGreen,    "purple_green");
heatmap_diverging_test!(test_heatmap_purple_orange,   ColorMap::PurpleOrange,   "purple_orange");
heatmap_diverging_test!(test_heatmap_red_blue,        ColorMap::RedBlue,        "red_blue");
heatmap_diverging_test!(test_heatmap_red_grey,        ColorMap::RedGrey,        "red_grey");
heatmap_diverging_test!(test_heatmap_red_yellow_blue, ColorMap::RedYellowBlue,  "red_yellow_blue");
heatmap_diverging_test!(test_heatmap_red_yellow_green,ColorMap::RedYellowGreen, "red_yellow_green");
heatmap_diverging_test!(test_heatmap_spectral,        ColorMap::Spectral,       "spectral");

// ── A few new sequential variants render correctly ────────────────────────────

macro_rules! heatmap_sequential_test {
    ($fn_name:ident, $variant:expr, $filename:literal) => {
        #[test]
        fn $fn_name() {
            outdir();
            let heatmap = Heatmap::new()
                .with_data(sequential_data())
                .with_color_map($variant);
            let plots = vec![Plot::Heatmap(heatmap)];
            let layout = Layout::auto_from_plots(&plots)
                .with_title(concat!("Heatmap — ", $filename));
            let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
            std::fs::write(concat!("test_outputs/colormap_", $filename, ".svg"), &svg).unwrap();
            assert!(svg.starts_with("<svg"), "{} SVG must start with <svg", $filename);
        }
    };
}

heatmap_sequential_test!(test_heatmap_magma,               ColorMap::Magma,            "magma");
heatmap_sequential_test!(test_heatmap_plasma,              ColorMap::Plasma,           "plasma");
heatmap_sequential_test!(test_heatmap_turbo,               ColorMap::Turbo,            "turbo");
heatmap_sequential_test!(test_heatmap_cubehelix,           ColorMap::Cubehelix,        "cubehelix");
heatmap_sequential_test!(test_heatmap_yellow_orange_red,   ColorMap::YellowOrangeRed,  "yellow_orange_red");
heatmap_sequential_test!(test_heatmap_rainbow,             ColorMap::Rainbow,          "rainbow");
