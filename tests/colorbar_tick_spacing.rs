//! Colorbar tick spacing: the log/count colorbar appends a tick at the exact data
//! maximum on top of the nearest power-of-ten tick. When the two sit within a label
//! height of each other their labels overprint, so the renderer drops the redundant
//! data-max label while keeping it when it is clearly clear of the decade tick.

mod common;

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
    common::write_test_output("test_outputs/colorbar_tick_spacing_suppressed.svg", &svg).unwrap();

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
    common::write_test_output("test_outputs/colorbar_tick_spacing_kept.svg", &svg).unwrap();

    assert!(
        svg.contains(">100</text>"),
        "the 100 decade tick must remain labelled"
    );
    assert!(
        svg.contains(">299</text>"),
        "the data-max tick (299) is clear of the decade and must be kept"
    );
}

// ── Bug: gap-guard `continue` skips the tick mark as well as the label ───────

#[test]
fn test_suppressed_tick_label_still_draws_tick_mark() {
    // peak=112: the "111" data-max label is suppressed (within tick_size of "100").
    // The tick mark at that position must still be drawn — only the label is redundant.
    // peak=300: "299" is clearly away from "100"; all 5 tick marks are drawn.
    // Both plots share identical x/y data ranges so their axis <line> counts are
    // equal. The only structural difference is the colorbar tick marks.
    // After the fix both SVGs should contain the same number of <line> elements.
    // With the bug, peak=112 has one fewer (the suppressed tick's mark is also gone).
    let plots_112 = make_hexbin_with_peak(112);
    let layout_112 = Layout::auto_from_plots(&plots_112);
    let svg_112 = render_svg(plots_112, layout_112);

    let plots_300 = make_hexbin_with_peak(300);
    let layout_300 = Layout::auto_from_plots(&plots_300);
    let svg_300 = render_svg(plots_300, layout_300);

    common::write_test_output("test_outputs/colorbar_tick_mark_suppressed_112.svg", &svg_112).unwrap();
    common::write_test_output("test_outputs/colorbar_tick_mark_full_300.svg", &svg_300).unwrap();

    let line_count_112 = svg_112.matches("<line").count();
    let line_count_300 = svg_300.matches("<line").count();

    assert_eq!(
        line_count_112, line_count_300,
        "a suppressed tick label must not suppress its tick mark: \
         peak=112 has {line_count_112} <line> elements, peak=300 has {line_count_300}"
    );
}

// ── Bug: log colorbar "1" label suppressed by proximity to the "0" origin tick

fn make_hexbin_large_range(peak_count: usize) -> Vec<Plot> {
    // Use x/y in [10, 50] so axis ticks fall on multiples of 10 and never
    // produce a bare ">1</text>" that would confuse the colorbar label check.
    let mut x = vec![30.0_f64; peak_count];
    let mut y = vec![30.0_f64; peak_count];
    for (sx, sy) in [(11.0, 11.0), (49.0, 12.0), (11.0, 48.0), (48.0, 49.0), (49.0, 30.0), (11.0, 29.0)] {
        x.push(sx);
        y.push(sy);
    }
    vec![Plot::Hexbin(
        HexbinPlot::new().with_data(x, y).with_log_color(true),
    )]
}

#[test]
fn test_log_colorbar_decade_1_not_suppressed_on_short_canvas() {
    // The hexbin log-color tick list starts with (0.0, "0") then (log10(2)≈0.301, "1").
    // At large log ranges on a short canvas the pixel gap between these two entries
    // drops below tick_size, causing the "1" label to be dropped by the gap guard.
    //
    // peak=2000  → log_max = log10(2000) ≈ 3.30
    // with_height(180) → bar_height ≈ 80px
    // gap = 80 × 0.301/3.30 ≈ 7.3px  < tick_size=12  → "1" suppressed (bug)
    let plots = make_hexbin_large_range(2000);
    let layout = Layout::auto_from_plots(&plots).with_height(180.0);
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/colorbar_tick_decade_1.svg", &svg).unwrap();

    assert!(
        svg.contains(">1</text>"),
        "the '1' decade label must remain visible on a large-range log colorbar \
         even on a short canvas (currently dropped by proximity to the '0' origin tick)"
    );
}
