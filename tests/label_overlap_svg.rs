mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::{BarPlot, ScatterPlot};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::{render_multiple, Primitive, Scene};
use kuva::AxisLabelOverlap;

fn text_element_count(scene: &Scene) -> usize {
    scene
        .elements
        .iter()
        .filter(|el| matches!(el, Primitive::Text { .. }))
        .count()
}

fn label_ys(scene: &Scene, labels: &[&str]) -> Vec<f64> {
    let set: std::collections::HashSet<&str> = labels.iter().copied().collect();
    scene
        .elements
        .iter()
        .filter_map(|el| match el {
            Primitive::Text { y, content, .. } if set.contains(content.as_str()) => Some(*y),
            _ => None,
        })
        .collect()
}

#[test]
fn test_thin_reduces_dense_numeric_labels() {
    // 51 ticks (step=1 on [0,50]) at 300px width: every label would overlap its
    // neighbour. Thin should skip the majority; Allow draws every one.
    let data = vec![(0.0f64, 0.0f64), (50.0, 1.0)];

    let make_layout = |plots: &[Plot]| {
        Layout::auto_from_plots(plots)
            .with_x_axis_min(0.0)
            .with_x_axis_max(50.0)
            .with_x_tick_step(1.0)
            .with_width(300.0)
    };

    let plots = vec![Plot::Scatter(ScatterPlot::new().with_data(data.clone()))];
    let layout_allow = make_layout(&plots);
    let scene_allow = render_multiple(plots, layout_allow);

    let plots = vec![Plot::Scatter(ScatterPlot::new().with_data(data))];
    let layout_thin = make_layout(&plots).with_x_label_overlap(AxisLabelOverlap::Thin);
    let scene_thin = render_multiple(plots, layout_thin);

    common::write_test_output(
        "test_outputs/label_overlap_thin_allow.svg",
        SvgBackend.render_scene(&scene_allow),
    )
    .unwrap();
    common::write_test_output(
        "test_outputs/label_overlap_thin.svg",
        SvgBackend.render_scene(&scene_thin),
    )
    .unwrap();

    let n_allow = text_element_count(&scene_allow);
    let n_thin = text_element_count(&scene_thin);

    assert!(
        n_thin < n_allow,
        "Thin should produce fewer Text elements than Allow (allow={n_allow}, thin={n_thin})"
    );
    // Must still draw at least a few labels (not completely empty).
    assert!(n_thin > 0, "Thin should draw at least some tick labels");
}

#[test]
fn test_stagger_splits_category_labels_into_two_rows() {
    // 8 categories with 5-char names at 300px: adjacent labels overlap, so
    // stagger should place them in alternating rows.  Allow leaves them all at
    // the same y.
    let cats: Vec<&str> = vec![
        "Grp01", "Grp02", "Grp03", "Grp04", "Grp05", "Grp06", "Grp07", "Grp08",
    ];

    let make_bar = || {
        cats.iter()
            .fold(BarPlot::new(), |b, &cat| b.with_bar(cat, 1.0))
    };

    let plots_allow = vec![Plot::Bar(make_bar())];
    let layout_allow = Layout::auto_from_plots(&plots_allow).with_width(300.0);
    let scene_allow = render_multiple(plots_allow, layout_allow);

    let plots_stagger = vec![Plot::Bar(make_bar())];
    let layout_stagger = Layout::auto_from_plots(&plots_stagger)
        .with_width(300.0)
        .with_x_label_overlap(AxisLabelOverlap::Stagger);
    let scene_stagger = render_multiple(plots_stagger, layout_stagger);

    common::write_test_output(
        "test_outputs/label_overlap_stagger_allow.svg",
        SvgBackend.render_scene(&scene_allow),
    )
    .unwrap();
    common::write_test_output(
        "test_outputs/label_overlap_stagger.svg",
        SvgBackend.render_scene(&scene_stagger),
    )
    .unwrap();

    let ys_allow = label_ys(&scene_allow, &cats);
    assert_eq!(ys_allow.len(), 8, "Allow should draw all 8 category labels");
    let allow_spread = ys_allow.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
        - ys_allow.iter().cloned().fold(f64::INFINITY, f64::min);
    assert!(
        allow_spread < 2.0,
        "Allow should place all labels at the same y (spread={allow_spread:.1}px)"
    );

    let ys_stagger = label_ys(&scene_stagger, &cats);
    assert_eq!(
        ys_stagger.len(),
        8,
        "Stagger should still draw all 8 labels (never drops)"
    );

    // Collect distinct y values (within 2px of each other count as one row).
    let mut sorted = ys_stagger.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut distinct: Vec<f64> = Vec::new();
    for y in sorted {
        if distinct.iter().all(|&d: &f64| (d - y).abs() >= 2.0) {
            distinct.push(y);
        }
    }
    assert_eq!(
        distinct.len(),
        2,
        "Stagger should produce exactly 2 y rows for these 8 categories, got {:?}",
        distinct
    );
}
