//! Fonts whose cmap makes metric extraction enumerate far more codes than the
//! 65,536 BMP slots it fills. Measuring a face used to visit every codepoint
//! each Unicode subtable claims and collect them into one list: a single
//! format-12 group spanning all 2^32 codes, or thousands of encoding records
//! pointing at one full-BMP subtable, meant billions of steps or gigabytes.
#![cfg(feature = "custom_fonts")]

#[path = "../src/render/text_metrics/test_font.rs"]
mod font_builder;
mod hostile;

use font_builder::{cmap, font, format12};

use kuva::fonts;
use kuva::plot::BarPlot;
use kuva::render::render::render_multiple;
use kuva::render::{layout::Layout, plots::Plot};

/// Register `bytes` and render a small bar chart in its family, which
/// measures the face.
fn register_and_measure(family: &'static str, bytes: Vec<u8>) {
    hostile::bounded(64 << 20, move || {
        let infos = fonts::register_font_bytes(bytes).expect("register");
        assert_eq!(infos[0].family, family);
        let plots = vec![Plot::Bar(BarPlot::new().with_bar("label", 1.0))];
        let layout = Layout::auto_from_plots(&plots).with_font_family(family);
        let scene = render_multiple(plots, layout);
        assert!(!scene.elements.is_empty());
    });
}

#[test]
fn a_range_over_the_whole_code_space() {
    // All 2^32 codes map to glyph 1 (and past it, which is out of range).
    let bytes = font("Kuva Hostile Wide", cmap(1, format12(&[(0, u32::MAX, 1)])));
    register_and_measure("Kuva Hostile Wide", bytes);
}

#[test]
fn many_overlapping_full_bmp_groups() {
    let groups = vec![(0, 0xFFFF, 1); 20_000];
    let bytes = font("Kuva Hostile Groups", cmap(1, format12(&groups)));
    register_and_measure("Kuva Hostile Groups", bytes);
}

#[test]
fn many_records_for_one_subtable() {
    let bytes = font(
        "Kuva Hostile Records",
        cmap(20_000, format12(&[(0, 0xFFFF, 1)])),
    );
    register_and_measure("Kuva Hostile Records", bytes);
}
