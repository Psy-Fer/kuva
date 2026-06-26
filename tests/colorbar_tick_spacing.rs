//! Colorbar tick spacing: the log/count colorbar appends a tick at the exact data
//! maximum on top of the nearest power-of-ten tick. When the two sit within a label
//! height of each other their labels overprint, so the renderer drops the redundant
//! data-max label while keeping it when it is clearly clear of the decade tick.

use kuva::backend::svg::SvgBackend;
use kuva::plot::hexbin::HexbinPlot;
use kuva::render::{layout::Layout, plots::Plot, render::render_multiple};

fn render_svg(plots: Vec<Plot>, layout: Layout) -> String {
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

/// One dense hex (all points share a coordinate) plus a handful of far-apart
/// singletons. The dense hex sets `v_max`; the singletons pin `v_min` at 1, so the
/// log colorbar labels run 1, 10, 100, … and append `v_max - v_min` at the top.
fn make_hexbin_with_peak(peak_count: usize) -> Vec<Plot> {
    let mut x = vec![2.5_f64; peak_count];
    let mut y = vec![2.5_f64; peak_count];
    for (sx, sy) in [(0.2, 0.2), (4.8, 0.3), (0.3, 4.7), (4.6, 4.8), (4.9, 2.5), (0.1, 2.6)] {
        x.push(sx);
        y.push(sy);
    }
    vec![Plot::Hexbin(
        HexbinPlot::new().with_data(x, y).with_log_color(true),
    )]
}

#[test]
fn test_data_max_label_suppressed_when_adjacent_to_decade() {
    // peak 112 → displayed max = 111, one notch above the 100 decade tick.
    let plots = make_hexbin_with_peak(112);
    let layout = Layout::auto_from_plots(&plots).with_title("Hexbin peak 112");
    let svg = render_svg(plots, layout);
    std::fs::write("test_outputs/colorbar_tick_spacing_suppressed.svg", &svg).unwrap();

    assert!(
        svg.contains(">100</text>"),
        "the 100 decade tick must remain labelled"
    );
    assert!(
        !svg.contains(">111</text>"),
        "the data-max tick (111) overprints the 100 decade and must be dropped"
    );
}

/// Guards against *over*-suppression: a data max that is clearly clear of the decade
/// must not be thinned. (The fix direction itself is covered by the suppressed test —
/// this one passes with or without thinning, but fails if the gap threshold is ever
/// made aggressive enough to drop a well-separated tick.)
#[test]
fn test_data_max_label_kept_when_clear_of_decade() {
    // peak 300 → displayed max = 299, ~half a decade above 100: no overprint.
    let plots = make_hexbin_with_peak(300);
    let layout = Layout::auto_from_plots(&plots).with_title("Hexbin peak 300");
    let svg = render_svg(plots, layout);
    std::fs::write("test_outputs/colorbar_tick_spacing_kept.svg", &svg).unwrap();

    assert!(
        svg.contains(">100</text>"),
        "the 100 decade tick must remain labelled"
    );
    assert!(
        svg.contains(">299</text>"),
        "the data-max tick (299) is clear of the decade and must be kept"
    );
}
