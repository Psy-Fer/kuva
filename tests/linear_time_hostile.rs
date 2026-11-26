//! Renderers that used to be quadratic or worse in the input size must finish
//! quickly on large inputs. Each test runs under the hostile-input harness, so
//! a regression fails after 10 seconds (or on the 2 GB memory cap) instead of
//! hanging the test run.

mod hostile;

use kuva::backend::svg::SvgBackend;
use kuva::plot::MosaicPlot;
use kuva::render::{layout::Layout, plots::Plot, render::render_multiple};

fn svg(plot: Plot) -> String {
    let plots = vec![plot];
    let layout = Layout::auto_from_plots(&plots);
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

#[test]
fn mosaic_with_a_distinct_column_and_row_per_cell() {
    // The renderer scanned every cell for each (column, row) pair, cubic in
    // the cell count, and reserved columns x rows primitives up front (5e9
    // here, an allocation failure).
    let mosaic = || {
        let mut m = MosaicPlot::new();
        for i in 0..50_000 {
            m = m.with_cell(format!("c{i}"), format!("r{i}"), 1.0 + (i % 7) as f64);
        }
        m
    };
    // The capacity estimate follows the cells, not columns x rows.
    assert!(Plot::Mosaic(mosaic()).estimated_primitives() <= 50_000 * 8);
    let out = hostile::bounded(64 << 20, move || svg(Plot::Mosaic(mosaic())));
    assert!(out.contains("c49999"));
}
