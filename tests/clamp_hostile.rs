//! Labels wider than the plot, plots smaller than their margins and spans
//! outside the plot must not panic. These positions were limited with
//! `f64::clamp`, which panics when its minimum is above its maximum or either
//! bound is NaN.

use kuva::backend::svg::SvgBackend;
use kuva::plot::{
    GenomeBuild, LabelStyle, ManhattanPlot, NetworkPlot, ScatterPlot, SyntenyPlot, VennPlot,
    VolcanoPlot,
};
use kuva::render::render_utils::{repel_labels, RepelItem};
use kuva::render::{layout::Layout, plots::Plot, render::render_multiple};

fn svg_sized(plot: Plot, size: Option<(f64, f64)>) -> String {
    let plots = vec![plot];
    let mut layout = Layout::auto_from_plots(&plots);
    if let Some((w, h)) = size {
        layout = layout.with_width(w).with_height(h);
    }
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

/// Default size, then a small one.
const SIZES: [Option<(f64, f64)>; 2] = [None, Some((120.0, 90.0))];

fn long(n: usize) -> String {
    "a very long label ".repeat(n)
}

#[test]
fn repel_centres_a_label_larger_than_its_bounds() {
    let item = |half_w, half_h| RepelItem {
        anchor: (50.0, 50.0),
        half_w,
        half_h,
        pos: (50.0, 50.0),
    };
    // Wider and taller than the 100 x 80 bounds: centred.
    let mut items = [item(500.0, 200.0), item(5.0, 5.0)];
    repel_labels(&mut items, (0.0, 0.0, 100.0, 80.0), 50);
    assert_eq!(items[0].pos, (50.0, 40.0));
    // Zero-size and NaN bounds.
    for bounds in [(10.0, 10.0, 10.0, 10.0), (f64::NAN, 0.0, f64::NAN, 80.0)] {
        let mut items = [item(5.0, 5.0), item(500.0, 5.0)];
        repel_labels(&mut items, bounds, 50);
    }
}

#[test]
fn repel_labels_longer_than_the_plot_render() {
    for size in SIZES {
        let scatter = ScatterPlot::new()
            .with_data([(1.0, 2.0), (2.0, 3.0), (3.0, 1.0)])
            .with_labels([long(20), "b".into(), long(5)])
            .with_repel_labels();
        assert!(svg_sized(Plot::Scatter(scatter), size).contains("<svg"));

        let volcano = VolcanoPlot::new()
            .with_point(long(20), 3.0, 1e-8)
            .with_point("g2", -2.5, 1e-6)
            .with_point("g3", 0.1, 0.5)
            .with_label_top(3)
            .with_label_style(LabelStyle::Repel);
        assert!(svg_sized(Plot::Volcano(volcano), size).contains("<svg"));

        let manhattan = ManhattanPlot::new()
            .with_data_bp(
                [
                    (long(20), 1_000_000.0, 1e-9),
                    ("1".to_string(), 2_000_000.0, 1e-3),
                ],
                GenomeBuild::Hg38,
            )
            .with_label_top(2)
            .with_label_style(LabelStyle::Repel);
        assert!(svg_sized(Plot::Manhattan(manhattan), size).contains("<svg"));
    }
}

#[test]
fn network_labels_wider_than_the_plot_render() {
    for size in SIZES {
        for font in [12, 400] {
            let net = NetworkPlot::new()
                .with_edge(long(30), "b".to_string(), 1.0)
                .with_edge("b".to_string(), "c".to_string(), 1.0)
                .with_edge("c".to_string(), long(10), 1.0)
                .with_labels()
                .with_repel_labels()
                .with_label_size(font);
            assert!(svg_sized(Plot::Network(net), size).contains("<svg"));
        }
    }
}

#[test]
fn synteny_with_long_labels_on_a_narrow_plot_renders() {
    for size in [Some((80.0, 200.0)), Some((20.0, 60.0))] {
        let syn = SyntenyPlot::new()
            .with_sequences([(long(10), 1000.0), (long(12), 800.0)])
            .with_block(0, 0.0, 500.0, 1, 100.0, 600.0);
        assert!(svg_sized(Plot::Synteny(syn), size).contains("<svg"));
    }
}

#[test]
fn venn_leader_labels_on_a_tiny_plot_render() {
    let venn = VennPlot::new()
        .with_set("A", vec!["1", "2", "3"])
        .with_set("B", vec!["2", "3", "4"])
        .with_set("C", vec!["3", "5"])
        .with_leader_lines(true);
    assert!(svg_sized(Plot::Venn(venn), Some((30.0, 30.0))).contains("<svg"));
}

#[test]
fn manhattan_with_chromosomes_outside_a_fixed_x_range_renders() {
    let plot = ManhattanPlot::new().with_data_bp(
        [
            ("1".to_string(), 1_000_000.0, 1e-9),
            ("2".to_string(), 5_000_000.0, 1e-4),
            ("3".to_string(), 9_000_000.0, 1e-2),
        ],
        GenomeBuild::Hg38,
    );
    let plots = vec![Plot::Manhattan(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_x_axis_min(0.0)
        .with_x_axis_max(1e6);
    let out = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert!(out.contains("<svg"));
}
