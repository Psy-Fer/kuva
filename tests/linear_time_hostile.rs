//! Renderers that used to be quadratic or worse in the input size must finish
//! quickly on large inputs. Each test runs under the hostile-input harness, so
//! a regression fails after 10 seconds (or on the 2 GB memory cap) instead of
//! hanging the test run.

mod hostile;

use kuva::backend::svg::SvgBackend;
use kuva::plot::logo::{Alphabet, SequenceLogo};
use kuva::plot::roc::compute_group;
use kuva::plot::{BoxPlot, NetworkLayout, NetworkPlot, ScatterPlot, StripPlot, ViolinPlot};
use kuva::plot::{Clustermap, GanttPlot, MosaicPlot, RocGroup, SurvivalPlot};
use kuva::render::color::Color;
use kuva::render::render::{Primitive, Scene};
use kuva::render::render_utils::{repel_labels, RepelItem, MAX_REPEL_LABELS};
use kuva::render::track_stack::{Interval, IntervalTrack, TrackStack};
use kuva::render::{layout::Layout, plots::Plot, render::render_multiple};

fn svg(plot: Plot) -> String {
    let plots = vec![plot];
    let layout = Layout::auto_from_plots(&plots);
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

#[test]
fn mosaic_with_a_distinct_column_and_row_per_cell() {
    // The renderer scanned every cell for each (column, row) pair, cubic in
    // the cell count, and reserved columns x rows primitives up front (8e8
    // here, over 100 GB).
    let mosaic = || {
        let mut m = MosaicPlot::new();
        for i in 0..20_000 {
            m = m.with_cell(format!("c{i}"), format!("r{i}"), 1.0 + (i % 7) as f64);
        }
        m
    };
    // The capacity estimate follows the cells, not columns x rows.
    assert!(Plot::Mosaic(mosaic()).estimated_primitives() <= 20_000 * 8);
    let out = hostile::bounded(64 << 20, move || svg(Plot::Mosaic(mosaic())));
    assert!(out.contains("c19999"));
}

#[test]
fn survival_with_many_censored_rows_and_event_times() {
    // Censoring levels scanned the Kaplan-Meier steps once per censored row,
    // and the log-rank test rescanned every row twice per event time: about
    // 10^9 steps each here.
    let out = hostile::bounded(64 << 20, || {
        let group = |offset: f64| {
            let n = 30_000;
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
    // DeLong compared every positive score with every negative one: 4 x 10^10
    // comparisons for 200,000 predictions, run again for the legend. This
    // calls the computation directly; drawing a 200,000-point curve in a
    // debug build is slow for unrelated reasons.
    let auc = hostile::bounded(64 << 20, || {
        let preds = (0..200_000).map(|i| (((i * 7919) % 100_003) as f64, i % 2 == 0));
        let group = RocGroup::new("model").with_raw(preds).with_ci(true);
        compute_group(&group).auc
    });
    assert!(auc > 0.4 && auc < 0.6, "{auc}");
}

#[test]
fn gantt_with_a_group_per_task() {
    // Group order used Vec::contains per task and display rows rescanned every
    // task per group: about 10^10 steps for 100,000 tasks, repeated in several
    // layout and render passes. This calls the ordering directly; a full debug
    // render of that many bars is slow for unrelated reasons (SVG writing).
    let labels = hostile::bounded(64 << 20, || {
        let mut g = GanttPlot::new();
        for i in 0..100_000 {
            g = g.with_task_group(format!("g{i}"), format!("t{i}"), i as f64, i as f64 + 2.0);
        }
        assert_eq!(g.effective_group_order().len(), 100_000);
        g.row_labels()
    });
    // A header row and a task row per group.
    assert_eq!(labels.len(), 200_000);
    assert_eq!(labels[199_998..], ["g99999", "t99999"]);
}

#[test]
fn categorical_clustermap_with_every_cell_distinct() {
    // Category resolution de-duplicated with list searches, and each cell's
    // colour was a search of the resolved categories: N x D for N cells and
    // D distinct values, several times per render.
    let out = hostile::bounded(64 << 20, || {
        let n = 200;
        let cats: Vec<Vec<String>> = (0..n)
            .map(|r| (0..n).map(|c| format!("v{}", r * n + c)).collect())
            .collect();
        let cm = Clustermap::new()
            .with_categorical_data(cats)
            .with_cluster_rows(false)
            .with_cluster_cols(false);
        svg(Plot::Clustermap(cm))
    });
    assert!(out.contains("<svg"));
}

fn rect(x: f64, y: f64, w: f64, h: f64, fill: &str) -> Primitive {
    Primitive::Rect {
        x,
        y,
        width: w,
        height: h,
        fill: Color::from(fill),
        stroke: None,
        stroke_width: None,
        opacity: None,
    }
}

#[test]
fn rect_batching_with_a_huge_rect_among_small_ones() {
    // The batching overlap check bucketed every rect into a grid sized from
    // the median rect, cell by cell, so one rect 10^12 px wide meant 10^11
    // bucket inserts. Such rects are now checked directly.
    let svg = hostile::bounded(64 << 20, || {
        let mut scene = Scene::new(400.0, 400.0);
        let colors = ["#112233", "#445566", "#778899"];
        for i in 0..5_000 {
            let (x, y) = ((i % 100) as f64 * 4.0, (i / 100) as f64 * 4.0);
            scene.add(rect(x, y, 4.0, 4.0, colors[i % 3]));
        }
        scene.add(rect(-5e11, 10.0, 1e12, 3.0, "#112233"));
        for i in 0..5_000 {
            scene.add(rect((i % 100) as f64 * 4.0, 300.0, 4.0, 4.0, colors[i % 3]));
        }
        SvgBackend.render_scene(&scene)
    });
    // Every rect is drawn: the small ones as subpaths, the huge one included.
    let subpaths = svg.matches('z').count() + svg.matches("<rect").count();
    assert!(subpaths >= 10_001, "{subpaths}");
}

#[test]
fn interval_track_with_every_interval_overlapping() {
    // Row packing scanned every open row for each interval, and the stack
    // asked for the track height several times per render, each time
    // packing again: about 5 x 1.8 x 10^9 row checks here.
    let svg = hostile::bounded(64 << 20, || {
        let intervals = (0..60_000)
            .map(|i| Interval::new(i as f64 * 1e-3, 100.0 + i as f64 * 1e-3))
            .collect();
        let scene = TrackStack::new()
            .x_range(0.0, 200.0)
            .track(IntervalTrack::new(intervals))
            .render(600.0);
        SvgBackend.render_scene(&scene)
    });
    assert!(svg.contains("<svg"));
}

#[test]
fn network_with_many_components() {
    // The component-wise layout scanned every edge once per component: 20,000
    // disjoint pairs meant 4 x 10^8 edge lookups.
    let positions = hostile::bounded(64 << 20, || {
        let mut net = NetworkPlot::new().with_layout(NetworkLayout::ForceDirected);
        for i in 0..20_000 {
            net = net.with_edge(format!("a{i}"), format!("b{i}"), 1.0);
        }
        net.compute_positions()
    });
    assert_eq!(positions.len(), 40_000);
    assert!(positions.iter().all(|p| p.0.is_finite() && p.1.is_finite()));
}

#[test]
fn wrapped_logo_with_many_positions_and_sequences() {
    // Every wrapped row cloned the whole logo, all sequences included, and
    // re-walked each sequence from its start to cut out the row's slice:
    // 400 rows x 2,500 sequences x 20,000 positions here.
    let scene = hostile::bounded(64 << 20, || {
        // The matrix is built directly and the sequences attached through the
        // public field, so the test time goes on wrapping, not on counting
        // 5 x 10^7 letters in a debug build.
        let seq = "ACGT".repeat(5_000);
        let mut logo = SequenceLogo::from_matrix(
            (0..20_000)
                .map(|p| {
                    let mut row = vec![1.0; 4];
                    row[p % 4] = 2_497.0;
                    row
                })
                .collect(),
            Alphabet::Dna,
        );
        logo.sample_size = Some(2_500);
        logo.source_sequences = Some(vec![seq; 2_500]);
        let plots = vec![Plot::SequenceLogo(logo)];
        let layout = Layout::auto_from_plots(&plots);
        render_multiple(plots, layout)
    });
    // 400 rows of 50 positions, stacked.
    assert!(scene.height > 400.0 * 200.0, "height {}", scene.height);
}

fn swarm_scene(plot: Plot) -> Scene {
    let plots = vec![plot];
    let layout = Layout::auto_from_plots(&plots);
    render_multiple(plots, layout)
}

fn budget_warned(scene: &Scene) -> bool {
    scene.warnings.iter().any(|w| w.code == "swarm.budget")
}

#[test]
fn swarm_with_many_equal_values() {
    // Every candidate offset was checked against every point already placed:
    // 2,001 candidates x 10^5 points x 10^5 points here. Equal values now
    // reuse the previous answer once the columns are full, so no budget is
    // needed.
    for horizontal in [false, true] {
        let scene = hostile::bounded(64 << 20, move || {
            swarm_scene(Plot::Strip(
                StripPlot::new()
                    .with_group("a", vec![5.0; 100_000])
                    .with_swarm()
                    .with_horizontal(horizontal),
            ))
        });
        assert!(!budget_warned(&scene), "horizontal {horizontal}");
    }
}

#[test]
fn swarm_with_many_nearly_equal_values() {
    // Distinct but nearly equal values fill the columns and then try every
    // candidate for each point. The candidate budget stops that and reports
    // it, for the strip plot and the box and violin overlays alike.
    let values: Vec<f64> = (0..100_000).map(|i| 5.0 + i as f64 * 1e-12).collect();
    let plots: Vec<Box<dyn Fn(Vec<f64>) -> Plot + Send>> = vec![
        Box::new(|v| Plot::Strip(StripPlot::new().with_group("a", v).with_swarm())),
        Box::new(|v| {
            Plot::Strip(
                StripPlot::new()
                    .with_group("a", v)
                    .with_swarm()
                    .with_horizontal(true),
            )
        }),
        Box::new(|v| Plot::Box(BoxPlot::new().with_group("a", v).with_swarm_overlay())),
        Box::new(|v| {
            Plot::Violin(
                ViolinPlot::new()
                    .with_group("a", v)
                    .with_swarm_overlay()
                    .with_horizontal(true),
            )
        }),
    ];
    for (i, make) in plots.into_iter().enumerate() {
        let v = values.clone();
        let scene = hostile::bounded(64 << 20, move || swarm_scene(make(v)));
        assert!(budget_warned(&scene), "plot {i}");
    }
}

#[test]
fn ordinary_swarm_has_no_budget_warning() {
    // 10,000 points spread over a normal-looking range, all in one group.
    let values: Vec<f64> = (0..10_000)
        .map(|i| ((i * 7919) % 10_000) as f64 / 100.0)
        .collect();
    let scene = hostile::bounded(64 << 20, move || {
        swarm_scene(Plot::Strip(
            StripPlot::new().with_group("a", values).with_swarm(),
        ))
    });
    assert!(!budget_warned(&scene));
}

fn repel_warned(scene: &Scene) -> bool {
    scene
        .warnings
        .iter()
        .any(|w| w.code == "labels.repel_skipped")
}

#[test]
fn repel_with_every_point_labelled() {
    // Repulsion compared every label with every other label and every anchor,
    // 150 times: 50,000 labelled points meant about 5 x 10^11 checks.
    let scene = hostile::bounded(64 << 20, || {
        let pts: Vec<(f64, f64)> = (0..50_000)
            .map(|i| ((i % 100) as f64, (i / 100) as f64))
            .collect();
        let labels: Vec<String> = (0..50_000).map(|i| format!("p{i}")).collect();
        let plots = vec![Plot::Scatter(
            ScatterPlot::new()
                .with_data(pts)
                .with_labels(labels)
                .with_repel_labels(),
        )];
        let layout = Layout::auto_from_plots(&plots);
        render_multiple(plots, layout)
    });
    assert!(repel_warned(&scene));
}

#[test]
fn network_label_repel_with_many_nodes() {
    let scene = hostile::bounded(64 << 20, || {
        let mut net = NetworkPlot::new()
            .with_layout(NetworkLayout::Circle)
            .with_labels()
            .with_repel_labels();
        for i in 0..5_000 {
            net = net.with_node(format!("node{i}"));
        }
        let plots = vec![Plot::Network(net)];
        let layout = Layout::auto_from_plots(&plots);
        render_multiple(plots, layout)
    });
    assert!(repel_warned(&scene));
}

#[test]
fn repel_labels_leaves_too_many_items_alone() {
    let item = |i: usize| RepelItem {
        anchor: (50.0, 50.0),
        half_w: 10.0,
        half_h: 5.0,
        pos: (50.0 + i as f64 * 1e-3, 50.0),
    };
    let bounds = (0.0, 0.0, 400.0, 400.0);
    // At the limit, stacked labels are pushed apart.
    let mut at_limit: Vec<RepelItem> = (0..MAX_REPEL_LABELS).map(item).collect();
    hostile::bounded(64 << 20, move || {
        repel_labels(&mut at_limit, bounds, 1);
        assert!(at_limit
            .iter()
            .enumerate()
            .any(|(i, it)| it.pos != item(i).pos));
    });
    // One more and they are left where they are.
    let mut over: Vec<RepelItem> = (0..=MAX_REPEL_LABELS).map(item).collect();
    hostile::bounded(64 << 20, move || {
        repel_labels(&mut over, bounds, 150);
        assert!(over.iter().enumerate().all(|(i, it)| it.pos == item(i).pos));
    });
}

#[test]
fn a_few_repelled_labels_give_no_warning() {
    let pts: Vec<(f64, f64)> = (0..20).map(|i| (i as f64 * 0.01, 1.0)).collect();
    let labels: Vec<String> = (0..20).map(|i| format!("p{i}")).collect();
    let plots = vec![Plot::Scatter(
        ScatterPlot::new()
            .with_data(pts)
            .with_labels(labels)
            .with_repel_labels(),
    )];
    let layout = Layout::auto_from_plots(&plots);
    assert!(!repel_warned(&render_multiple(plots, layout)));
}
