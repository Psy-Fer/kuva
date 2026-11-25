//! Sankey links that form cycles must not hang the column assignment.

mod hostile;

use hostile::{bounded, render};
use kuva::plot::sankey::{SankeyNodeOrder, SankeyPlot};
use kuva::render::plots::Plot;

fn render_sankey(sankey: SankeyPlot) -> String {
    bounded(8 << 20, move || render(Plot::Sankey(sankey)))
}

/// The x of the `<text>` element whose content is exactly `label`.
fn label_x(svg: &str, label: &str) -> f64 {
    let end = svg
        .find(&format!(">{label}</text>"))
        .unwrap_or_else(|| panic!("no label {label:?}"));
    let start = svg[..end].rfind("<text").unwrap();
    let tag = &svg[start..end];
    let x = tag.find(" x=\"").unwrap() + 4;
    tag[x..][..tag[x..].find('"').unwrap()].parse().unwrap()
}

#[test]
fn cycle_reachable_from_a_source_renders() {
    // S -> A -> B -> A, then B -> C -> D. Column propagation used to push A
    // and B right forever.
    let svg = render_sankey(
        SankeyPlot::new()
            .with_link("S", "A", 5.0)
            .with_link("A", "B", 4.0)
            .with_link("B", "A", 2.0)
            .with_link("B", "C", 2.0)
            .with_link("C", "D", 2.0),
    );
    // The back link B -> A is left out of column assignment, so every node
    // gets its own column in order from the source.
    let xs: Vec<f64> = ["S", "A", "B", "C", "D"]
        .iter()
        .map(|l| label_x(&svg, l))
        .collect();
    assert!(
        xs.windows(2).all(|w| w[0] < w[1]),
        "columns out of order: {xs:?}"
    );
}

#[test]
fn self_link_after_a_source_renders() {
    let svg = render_sankey(
        SankeyPlot::new()
            .with_link("S", "A", 5.0)
            .with_link("A", "A", 1.0)
            .with_link("A", "T", 5.0)
            .with_link("T", "U", 5.0),
    );
    let xs: Vec<f64> = ["S", "A", "T", "U"]
        .iter()
        .map(|l| label_x(&svg, l))
        .collect();
    assert!(
        xs.windows(2).all(|w| w[0] < w[1]),
        "columns out of order: {xs:?}"
    );
}

#[test]
fn cycles_render_with_every_ordering_and_link_style() {
    let base = || {
        SankeyPlot::new()
            .with_link("S", "A", 5.0)
            .with_link("A", "B", 4.0)
            .with_link("B", "C", 4.0)
            .with_link("C", "A", 1.0)
            .with_link("C", "C", 1.0)
            .with_link("X", "Y", 1.0)
            .with_link("Y", "X", 1.0)
            .with_node_column("B", 1)
    };
    for order in [
        SankeyNodeOrder::Input,
        SankeyNodeOrder::CrossingReduction,
        SankeyNodeOrder::Neighbornet,
    ] {
        for sankey in [
            base().with_node_order(order),
            base().with_node_order(order).with_gradient_links(),
            base().with_node_order(order).with_per_link_colors(),
        ] {
            let svg = render_sankey(sankey);
            for label in ["S", "A", "B", "C", "X", "Y"] {
                assert!(svg.contains(&format!(">{label}</text>")), "{label} missing");
            }
        }
    }
}

#[test]
fn long_cycle_through_many_nodes_renders() {
    // A 300-node ring fed by one source.
    let n = 300;
    let mut sankey = SankeyPlot::new().with_link("src".to_string(), "n0".to_string(), 1.0);
    for i in 0..n {
        sankey = sankey.with_link(format!("n{i}"), format!("n{}", (i + 1) % n), 1.0);
    }
    let svg = render_sankey(sankey);
    // Only the link back to n0 closes the ring, so n0..n299 run left to right.
    let xs: Vec<f64> = [0, 1, 2, n / 2, n - 1]
        .iter()
        .map(|i| label_x(&svg, &format!("n{i}")))
        .collect();
    assert!(
        xs.windows(2).all(|w| w[0] < w[1]),
        "columns out of order: {xs:?}"
    );
}
