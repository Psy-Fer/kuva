//! Log axes and log colour bars with an infinite or NaN extent must not hang.
//! The tick loops used to run until a power of ten passed the bound, which
//! never happens for an infinite or NaN bound, pushing a tick every pass.

mod hostile;

use hostile::bounded;
use kuva::backend::svg::SvgBackend;
use kuva::plot::{HexbinPlot, ScatterPlot, ZReduce};
use kuva::render::{layout::Layout, plots::Plot, render::render_multiple};

fn render_with(plot: Plot, layout: impl FnOnce(Layout) -> Layout + Send + 'static) -> String {
    bounded(8 << 20, move || {
        let plots = vec![plot];
        let l = layout(Layout::auto_from_plots(&plots));
        SvgBackend.render_scene(&render_multiple(plots, l))
    })
}

fn scatter(points: Vec<(f64, f64)>) -> Plot {
    Plot::Scatter(ScatterPlot::new().with_data(points))
}

#[test]
fn log_axis_with_data_near_f64_max_renders() {
    // The auto range rounds 1e308 up to the next tick, which is infinite.
    let svg = render_with(scatter(vec![(1.0, 1.0), (2.0, 1e308)]), |l| l.with_log_y());
    assert!(svg.contains("<svg"));
}

#[test]
fn log_axis_with_an_infinite_or_nan_bound_renders() {
    for bound in [f64::INFINITY, f64::NAN, f64::MAX] {
        let svg = render_with(scatter(vec![(1.0, 1.0), (10.0, 100.0)]), move |l| {
            l.with_log_x()
                .with_log_y()
                .with_x_axis_max(bound)
                .with_y_axis_max(bound)
        });
        assert!(svg.contains("<svg"), "bound {bound}");
    }
}

fn hexbin() -> HexbinPlot {
    let x: Vec<f64> = (0..50).map(|i| (i % 7) as f64).collect();
    let y: Vec<f64> = (0..50).map(|i| (i % 5) as f64).collect();
    HexbinPlot::new().with_data(x, y).with_log_color(true)
}

#[test]
fn hexbin_log_colorbar_with_an_infinite_or_nan_range_renders() {
    for (lo, hi) in [
        (0.0, f64::INFINITY),
        (f64::NEG_INFINITY, 1.0),
        (0.0, f64::NAN),
        (f64::NAN, f64::NAN),
        (-f64::MAX, f64::MAX),
    ] {
        let plot = Plot::Hexbin(hexbin().with_color_range(lo, hi));
        let svg = render_with(plot, |l| l);
        assert!(svg.contains("<svg"), "range ({lo}, {hi})");
    }
}

#[test]
fn hexbin_log_colorbar_with_overflowing_z_renders() {
    // A sum of z values near f64::MAX overflows to infinity.
    let z: Vec<f64> = (0..50)
        .map(|i| if i % 2 == 0 { f64::MAX } else { -f64::MAX })
        .collect();
    for reduce in [ZReduce::Sum, ZReduce::Max, ZReduce::Mean] {
        let plot = Plot::Hexbin(hexbin().with_z(z.clone(), reduce));
        let svg = render_with(plot, |l| l);
        assert!(svg.contains("<svg"));
    }
}
