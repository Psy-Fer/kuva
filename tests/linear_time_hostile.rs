//! Renderers that used to be quadratic or worse in the input size must finish
//! quickly on large inputs. Each test runs under the hostile-input harness, so
//! a regression fails after 10 seconds (or on the 2 GB memory cap) instead of
//! hanging the test run.

mod hostile;

use kuva::backend::svg::SvgBackend;
use kuva::plot::{MosaicPlot, RocGroup, RocPlot, SurvivalPlot};
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

#[test]
fn survival_with_many_censored_rows_and_event_times() {
    // Censoring levels scanned the Kaplan-Meier steps once per censored row,
    // and the log-rank test rescanned every row twice per event time: about
    // 10^10 steps each here.
    let out = hostile::bounded(64 << 20, || {
        let group = |offset: f64| {
            let n = 100_000;
            // First half censored early, second half distinct event times.
            let times = (0..n)
                .map(|i| {
                    if i < n / 2 {
                        0.5 + i as f64 * 1e-6
                    } else {
                        offset + i as f64
                    }
                })
                .collect();
            let events = (0..n).map(|i| i >= n / 2).collect();
            (times, events)
        };
        let (ta, ea) = group(0.0);
        let (tb, eb) = group(0.25);
        let sp = SurvivalPlot::new()
            .with_group("a", ta, ea)
            .with_group("b", tb, eb)
            .with_logrank_pvalue(true);
        svg(Plot::Survival(sp))
    });
    assert!(out.contains("log-rank p"));
}

#[test]
fn roc_with_many_raw_predictions() {
    // DeLong compared every positive score with every negative one: 10^10
    // comparisons for 200,000 predictions.
    let out = hostile::bounded(64 << 20, || {
        let preds = (0..200_000).map(|i| (((i * 7919) % 100_003) as f64, i % 2 == 0));
        // The legend computes each group again, so this exercises both calls.
        let roc = RocPlot::new().with_legend("models").with_group(
            RocGroup::new("model")
                .with_raw(preds)
                .with_ci(true)
                .with_auc_label(true),
        );
        svg(Plot::Roc(roc))
    });
    assert!(out.contains("AUC"));
}
