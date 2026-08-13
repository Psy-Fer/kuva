//! Kaplan-Meier survival plot documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example survival
//! ```
//!
//! SVGs are written to `docs/src/assets/survival/`.

use kuva::backend::svg::SvgBackend;
use kuva::plot::SurvivalPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use std::fs;

const OUT: &str = "docs/src/assets/survival";

fn write(name: &str, plots: Vec<Plot>, layout: Layout) {
    fs::create_dir_all(OUT).unwrap();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
}

fn main() {
    // ── Basic single-group KM ─────────────────────────────────────────────
    let sp = SurvivalPlot::new().with_group(
        "All patients",
        vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 16.0, 18.0,
            20.0, 22.0, 24.0, 26.0,
        ],
        vec![
            true, true, false, true, true, false, true, false, true, false, true, false, true,
            false, false, true, false, true, false, false,
        ],
    );
    let plots = vec![Plot::Survival(sp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Kaplan-Meier Survival Curve")
        .with_x_label("Time (months)")
        .with_y_label("Survival probability");
    write("basic", plots, layout);

    // ── Two groups with CI ────────────────────────────────────────────────
    let ctrl_times = vec![
        1.0, 2.0, 3.0, 4.0, 5.0, 7.0, 8.0, 10.0, 12.0, 14.0, 15.0, 16.0, 18.0, 20.0, 22.0, 24.0,
        26.0, 28.0, 30.0, 32.0,
    ];
    let ctrl_events = vec![
        true, true, true, false, true, true, false, true, false, true, true, false, true, false,
        false, true, false, true, false, false,
    ];
    let trt_times = vec![
        2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0, 26.0, 28.0, 32.0, 34.0,
        36.0, 40.0, 42.0, 44.0,
    ];
    let trt_events = vec![
        false, true, false, true, false, false, true, false, false, true, false, false, true,
        false, false, true, false, false, false, false,
    ];
    let sp = SurvivalPlot::new()
        .with_group("Control", ctrl_times, ctrl_events)
        .with_group("Treatment", trt_times, trt_events)
        .with_ci(true)
        .with_pvalue_text("log-rank p = 0.038")
        .with_legend("Arm");
    let plots = vec![Plot::Survival(sp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Overall Survival by Treatment Arm")
        .with_x_label("Time (months)")
        .with_y_label("Probability of survival")
        .with_width(520.0)
        .with_height(400.0);
    write("two_groups_ci", plots, layout);

    // ── Three groups (disease stage) ──────────────────────────────────────
    let sp = SurvivalPlot::new()
        .with_group(
            "Stage I",
            vec![10.0, 14.0, 20.0, 26.0, 32.0, 36.0, 40.0, 44.0, 48.0, 52.0],
            vec![
                false, true, false, false, true, false, false, true, false, false,
            ],
        )
        .with_group(
            "Stage II",
            vec![5.0, 8.0, 12.0, 16.0, 20.0, 24.0, 28.0, 32.0, 36.0, 40.0],
            vec![
                true, true, false, true, false, true, false, false, true, false,
            ],
        )
        .with_group(
            "Stage III",
            vec![2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0],
            vec![
                true, true, true, false, true, true, false, true, false, true,
            ],
        )
        .with_pvalue_text("log-rank p < 0.001")
        .with_legend("Stage");
    let plots = vec![Plot::Survival(sp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Survival by Disease Stage")
        .with_x_label("Time (months)")
        .with_y_label("Overall survival")
        .with_width(520.0)
        .with_height(400.0);
    write("three_groups", plots, layout);

    // ── Clinical figure: risk table + median lines + log-rank p-value ─────────
    let sp = SurvivalPlot::new()
        .with_group(
            "Arm A",
            vec![
                2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0,
            ],
            vec![
                true, true, false, true, true, false, true, true, false, true, true, false,
            ],
        )
        .with_group(
            "Arm B",
            vec![
                6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0, 26.0, 28.0,
            ],
            vec![
                true, false, true, true, false, true, false, true, false, true, false, false,
            ],
        )
        .with_legend("Treatment")
        .with_risk_table(true)
        .with_median_lines(true)
        .with_logrank_pvalue(true);
    let plots = vec![Plot::Survival(sp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Overall Survival")
        .with_x_label("Time (months)")
        .with_y_label("Survival probability")
        .with_width(640.0)
        .with_height(560.0);
    write("clinical", plots, layout);
}
