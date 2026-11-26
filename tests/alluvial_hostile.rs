//! Alluvial Sankey data with tiny, huge, zero, negative or non-finite weights,
//! alluvia of different lengths, or extra columns from `with_link` must render
//! with crossing reduction on. The ordering objective dropped y values within
//! 1e-9 of each other and then looked them up exactly, and it assumed every
//! alluvium had a node on every column. Weights far apart in size also made
//! the 2-opt tour search loop forever.

mod hostile;

use kuva::backend::svg::SvgBackend;
use kuva::plot::{SankeyNodeOrder, SankeyPlot};
use kuva::render::{layout::Layout, plots::Plot, render::render_multiple};

fn svg(plot: SankeyPlot) -> String {
    let plots = vec![Plot::Sankey(plot)];
    let layout = Layout::auto_from_plots(&plots);
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

const MODES: [SankeyNodeOrder; 2] = [
    SankeyNodeOrder::CrossingReduction,
    SankeyNodeOrder::Neighbornet,
];

fn rows() -> [[&'static str; 3]; 4] {
    [
        ["A", "X", "P"],
        ["B", "Y", "Q"],
        ["A", "Y", "P"],
        ["B", "X", "Q"],
    ]
}

#[test]
fn alluvia_with_extreme_weights_render() {
    let weight_sets: [[f64; 4]; 7] = [
        [1e-12, 2e-12, 1e-12, 3e-12],
        [5e-324, 5e-324, 1e-320, 5e-324],
        [1e300, 1e-300, 1.0, 1e-12],
        [0.0, 0.0, 1.0, 0.0],
        [-1.0, 2.0, -3.0, 1.0],
        [f64::NAN, 1.0, 2.0, 1.0],
        [f64::INFINITY, 1.0, f64::NEG_INFINITY, 1.0],
    ];
    for weights in weight_sets {
        for mode in MODES {
            hostile::bounded(16 << 20, move || {
                let mut p = SankeyPlot::new().with_node_order(mode);
                for (row, w) in rows().into_iter().zip(weights) {
                    p = p.with_alluvium(row, w);
                }
                assert!(svg(p).contains("<svg"), "{weights:?}");
            });
        }
    }
}

#[test]
fn tiny_weights_get_the_same_order_as_scaled_up_weights() {
    // The crossing objective scales with the weights, so the chosen order (and
    // with it the order nodes are drawn in) must not depend on the scale.
    let order = |scale: f64, mode: SankeyNodeOrder| {
        let mut p = SankeyPlot::new().with_node_order(mode);
        for (row, w) in rows().into_iter().zip([3.0, 1.0, 2.0, 5.0]) {
            p = p.with_alluvium(row, w * scale);
        }
        let out = svg(p);
        let labels: Vec<String> = out
            .split("<text")
            .filter_map(|t| t.split('>').nth(1)?.split('<').next().map(String::from))
            .filter(|l| ["A", "B", "X", "Y", "P", "Q"].contains(&l.as_str()))
            .collect();
        labels
    };
    for mode in MODES {
        let want = order(1.0, mode);
        assert_eq!(want.len(), 6);
        assert_eq!(order(1e-12, mode), want);
    }
}

#[test]
fn ragged_and_empty_alluvia_and_extra_link_columns_render() {
    for mode in MODES {
        let ragged = SankeyPlot::new()
            .with_node_order(mode)
            .with_alluvium(["A", "X", "P"], 2.0)
            .with_alluvium(["B", "Y"], 1.0)
            .with_alluvium(["A"], 1.0)
            .with_alluvium(Vec::<&str>::new(), 1.0)
            .with_alluvium(["B", "X", "Q", "Z"], 3.0);
        assert!(svg(ragged).contains("<svg"));

        // Links add nodes in columns past the alluvium axes.
        let extra = SankeyPlot::new()
            .with_node_order(mode)
            .with_alluvium(["A", "X"], 2.0)
            .with_alluvium(["B", "Y"], 1.0)
            .with_alluvium(["A", "Y"], 1.0)
            .with_link("1~~X", "Out1", 2.0)
            .with_link("Out1", "Out2", 2.0)
            .with_link("Lone", "Other", 1.0);
        assert!(svg(extra).contains("<svg"));
    }
}
