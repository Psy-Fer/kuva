//! Waterfall plot documentation examples.
//!
//! Generates canonical SVG outputs used in the visus documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example waterfall
//! ```
//!
//! SVGs are written to `docs/src/assets/waterfall/`.

use visus::plot::WaterfallPlot;
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;

const OUT: &str = "docs/src/assets/waterfall";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/waterfall");

    basic();
    totals();
    connectors_values();
    difference();

    println!("Waterfall SVGs written to {OUT}/");
}

/// Classic delta-only waterfall — a running total that rises and falls.
fn basic() {
    let wf = WaterfallPlot::new()
        .with_delta("Revenue",        850.0)
        .with_delta("Cost of goods", -340.0)
        .with_delta("Personnel",     -180.0)
        .with_delta("Operations",     -90.0)
        .with_delta("Marketing",      -70.0)
        .with_delta("Other income",    55.0)
        .with_delta("Tax",            -85.0);

    let plots = vec![Plot::Waterfall(wf)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Revenue Breakdown")
        .with_y_label("USD (thousands)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Waterfall with intermediate totals — the classic income-statement layout.
///
/// Each `with_total()` bar drops to zero and shows the accumulated running
/// total at that point, making subtotals visible alongside the individual items.
fn totals() {
    let wf = WaterfallPlot::new()
        .with_delta("Revenue",       850.0)
        .with_delta("Cost of goods",-340.0)
        .with_total("Gross profit")
        .with_delta("Personnel",    -180.0)
        .with_delta("Operations",    -90.0)
        .with_delta("Marketing",     -70.0)
        .with_total("EBITDA")
        .with_delta("Depreciation",  -40.0)
        .with_delta("Interest",      -20.0)
        .with_delta("Tax",           -65.0)
        .with_total("Net income");

    let plots = vec![Plot::Waterfall(wf)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Income Statement")
        .with_y_label("USD (thousands)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/totals.svg"), svg).unwrap();
}

/// Connectors and value labels — dashed lines link bar tops, values annotate bars.
fn connectors_values() {
    let wf = WaterfallPlot::new()
        .with_delta("Q1 sales",    420.0)
        .with_delta("Q2 sales",    380.0)
        .with_delta("Returns",    -95.0)
        .with_delta("Discounts",  -60.0)
        .with_total("H1 net")
        .with_delta("Q3 sales",   410.0)
        .with_delta("Q4 sales",   455.0)
        .with_delta("Returns",   -105.0)
        .with_delta("Discounts",  -70.0)
        .with_total("H2 net")
        .with_connectors()
        .with_values();

    let plots = vec![Plot::Waterfall(wf)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Sales – Connectors + Values")
        .with_y_label("USD (thousands)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/connectors_values.svg"), svg).unwrap();
}

/// Difference bars — anchored at explicit [from, to] values, independent of
/// the running total.
///
/// The clearest use is when `from` and `to` match the heights of existing
/// Total bars so the reader can trace the connection directly.
/// Here both period totals are 320 and 730 respectively — the difference bar
/// sits exactly between those two reference levels and shows the +410 gain.
fn difference() {
    let wf = WaterfallPlot::new()
        .with_delta("Revenue",   500.0)   // RT = 500
        .with_delta("Costs",    -180.0)   // RT = 320
        .with_total("Period A")           // total bar: 0 → 320
        .with_delta("Revenue",   600.0)   // RT = 920
        .with_delta("Costs",    -190.0)   // RT = 730
        .with_total("Period B")           // total bar: 0 → 730
        // from = Period A total (320), to = Period B total (730)
        .with_difference("Period A→B", 320.0, 730.0)
        .with_values();

    let plots = vec![Plot::Waterfall(wf)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Period-over-Period Change")
        .with_y_label("USD (thousands)");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/difference.svg"), svg).unwrap();
}
