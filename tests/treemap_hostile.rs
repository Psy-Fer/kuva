//! Treemaps with many tiles must lay out without deep recursion.

mod hostile;

use hostile::{bounded, render};
use kuva::plot::treemap::{TreemapLayout, TreemapNode, TreemapPlot};
use kuva::render::plots::Plot;

fn one_big_many_small(n: usize) -> TreemapPlot {
    let mut tm = TreemapPlot::new().with_node(TreemapNode::leaf("big", 1e7));
    for i in 0..n {
        tm = tm.with_node(TreemapNode::leaf(format!("t{i}"), 1.0));
    }
    tm.with_show_labels(false)
}

#[test]
fn many_small_tiles_beside_a_big_one_lay_out_on_a_small_stack() {
    // The big tile leaves a thin strip, so each small tile takes a strip of its
    // own. Squarify used to recurse once per strip.
    let svg = bounded(256 * 1024, || {
        render(Plot::Treemap(one_big_many_small(50_000)))
    });
    assert!(svg.contains("<svg"));
}

#[test]
fn many_equal_tiles_lay_out_with_every_algorithm() {
    for algo in [
        TreemapLayout::Squarify,
        TreemapLayout::SliceDice,
        TreemapLayout::Binary,
    ] {
        let svg = bounded(8 << 20, move || {
            let mut tm = TreemapPlot::new().with_layout(algo);
            for i in 0..20_000 {
                tm = tm.with_node(TreemapNode::leaf(format!("t{i}"), 1.0));
            }
            render(Plot::Treemap(tm.with_show_labels(false)))
        });
        assert!(svg.contains("<svg"));
    }
}
