//! The log/count colorbar (hexbin and 2-D histogram with log colouring) must format
//! its tick labels through `Layout::with_colorbar_tick_format`, just like the linear
//! colorbar does. Previously these labels were hard-coded power-of-ten integers and the
//! custom formatter was ignored.

mod common;

use kuva::backend::svg::SvgBackend;
use kuva::plot::hexbin::HexbinPlot;
use kuva::plot::histogram2d::Histogram2D;
use kuva::render::layout::{Layout, TickFormat};
use kuva::render::{plots::Plot, render::render_multiple};
use std::sync::Arc;

fn render_svg(plots: Vec<Plot>, layout: Layout) -> String {
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

/// SI-style integer formatter: 1000 → "1k", 10000 → "10k", 1_000_000 → "1M".
fn si_format() -> TickFormat {
    TickFormat::Custom(Arc::new(|v: f64| {
        let n = v.round() as i64;
        if n != 0 && n % 1_000_000 == 0 {
            format!("{}M", n / 1_000_000)
        } else if n != 0 && n % 1000 == 0 {
            format!("{}k", n / 1000)
        } else {
            format!("{n}")
        }
    }))
}

/// One dense hex (`peak` coincident points) plus far-apart singletons that pin v_min at 1.
fn make_hexbin_with_peak(peak: usize) -> Vec<Plot> {
    let mut x = vec![2.5_f64; peak];
    let mut y = vec![2.5_f64; peak];
    for (sx, sy) in [(0.2, 0.2), (4.8, 0.3), (0.3, 4.7), (4.6, 4.8)] {
        x.push(sx);
        y.push(sy);
    }
    vec![Plot::Hexbin(
        HexbinPlot::new().with_data(x, y).with_log_color(true),
    )]
}

#[test]
fn test_hexbin_log_colorbar_honors_custom_tick_format() {
    // Peak 10001 → displayed counts run 1, 10, 100, 1000, 10000.
    let plots = make_hexbin_with_peak(10001);
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Hexbin SI colorbar")
        .with_colorbar_tick_format(si_format());
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/colorbar_tick_format_hexbin_si.svg", &svg).unwrap();

    assert!(svg.contains(">1k</text>"), "1000 should render as 1k");
    assert!(svg.contains(">10k</text>"), "10000 should render as 10k");
    assert!(
        svg.contains(">100</text>"),
        "100 stays 100 under the SI formatter"
    );
    assert!(
        !svg.contains(">1000</text>"),
        "raw 1000 must not appear once the SI formatter is applied"
    );
    assert!(
        !svg.contains(">10000</text>"),
        "raw 10000 must not appear once the SI formatter is applied"
    );
}

/// Documents that the default (`TickFormat::Auto`) output is unchanged by the routing —
/// integer count labels still render plain. (This passes with or without the fix, since
/// Auto and the old `as u64` cast agree on integers; the fix itself is guarded by the
/// custom-formatter tests above and below.)
#[test]
fn test_hexbin_log_colorbar_default_format_unchanged() {
    let plots = make_hexbin_with_peak(10001);
    let layout = Layout::auto_from_plots(&plots).with_title("Hexbin default colorbar");
    let svg = render_svg(plots, layout);

    assert!(
        svg.contains(">10000</text>"),
        "default Auto format keeps plain-integer count labels"
    );
}

#[test]
fn test_hist2d_log_colorbar_honors_custom_tick_format() {
    // A single dense cell drives the count up so the log colorbar spans several decades.
    let mut data: Vec<(f64, f64)> = vec![(5.0, 5.0); 1001];
    data.extend([(0.5, 0.5), (9.5, 9.5)]);
    let hist = Histogram2D::new()
        .with_data(data, (0.0, 10.0), (0.0, 10.0), 10, 10)
        .with_log_count();
    let plots = vec![Plot::Histogram2d(hist)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("hist2d log sci colorbar")
        .with_colorbar_tick_format(TickFormat::Sci);
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/colorbar_tick_format_hist2d_sci.svg", &svg).unwrap();

    assert!(
        svg.contains(">1e3</text>"),
        "1000 should render in sci notation as 1e3"
    );
    assert!(
        !svg.contains(">1000</text>"),
        "raw 1000 must not appear once the Sci formatter is applied to the log colorbar"
    );
}
