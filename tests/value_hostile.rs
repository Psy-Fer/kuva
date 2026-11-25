//! Data values that make a total zero, negative, infinite or NaN, and
//! sequence characters missing from a colour template, must not panic.

use std::collections::HashMap;

use kuva::backend::svg::SvgBackend;
use kuva::plot::brick::BrickTemplate;
use kuva::plot::{BrickPlot, ParetoPlot, WafflePlot};
use kuva::render::render::{collect_legend_entries, waffle_largest_remainder};
use kuva::render::{layout::Layout, plots::Plot, render::render_multiple};

fn svg(plot: Plot) -> String {
    let plots = vec![plot];
    let layout = Layout::auto_from_plots(&plots);
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

#[test]
fn pareto_with_a_zero_or_negative_total_renders_bars_without_a_cumulative_line() {
    for values in [
        vec![0.0, 0.0, 0.0],
        vec![5.0, -5.0, 0.0],
        vec![2.0, -7.0, 1.0],
    ] {
        for (labels, horizontal, max) in [(false, false, 0), (true, true, 2), (true, false, 2)] {
            let mut p = ParetoPlot::new()
                .with_cumulative_labels(labels)
                .with_horizontal(horizontal);
            for (i, &v) in values.iter().enumerate() {
                p = p.with_category(format!("c{i}"), v);
            }
            if max > 0 {
                p = p.with_max_categories(max);
            }
            let out = svg(Plot::Pareto(p));
            assert!(out.contains("<svg"));
            assert!(!out.contains("<polyline"), "no cumulative line: {values:?}");
        }
    }
}

#[test]
fn waffle_allocation_ignores_non_finite_and_negative_values() {
    // Shares of the finite positive values, summing to the cell count.
    assert_eq!(waffle_largest_remainder(&[1e308, 1e308], 100), [50, 50]);
    assert_eq!(
        waffle_largest_remainder(&[f64::MAX, f64::MAX, 0.0], 10),
        [5, 5, 0]
    );
    assert_eq!(waffle_largest_remainder(&[3.0, -1.0, 1.0], 4), [3, 0, 1]);
    assert_eq!(
        waffle_largest_remainder(&[f64::NAN, 1.0, f64::INFINITY], 10),
        [0, 10, 0]
    );
    for values in [vec![f64::NAN], vec![f64::INFINITY], vec![-1.0, 0.0], vec![]] {
        let counts = waffle_largest_remainder(&values, 100);
        assert_eq!(counts.len(), values.len());
        assert!(counts.iter().all(|&c| c == 0), "{values:?}");
    }
    // Valid input still sums to the cell count.
    let counts = waffle_largest_remainder(&[1.0, 2.0, 3.3, 0.7], 97);
    assert_eq!(counts.iter().sum::<usize>(), 97);
}

#[test]
fn waffle_with_non_finite_or_overflowing_values_renders() {
    for values in [
        vec![f64::NAN, 1.0],
        vec![f64::INFINITY, 1.0],
        vec![f64::NEG_INFINITY, 1.0],
        vec![1e308, 1e308, 1e308],
        vec![-3.0, 1.0],
    ] {
        for legend in [false, true] {
            let mut w = WafflePlot::new();
            for (i, &v) in values.iter().enumerate() {
                w = w.with_category(format!("c{i}"), v, "steelblue");
            }
            if legend {
                w = w
                    .with_legend("legend")
                    .with_show_percents()
                    .with_show_counts();
            }
            assert!(svg(Plot::Waffle(w)).contains("<svg"), "{values:?}");
        }
    }
}

#[test]
fn brick_sequences_with_characters_missing_from_the_template_render() {
    let seqs = ["ACGTN-acgt", "AAAA", "ACéGT😀", " A C "];
    let dna = BrickPlot::new()
        .with_sequences(seqs)
        .with_template(BrickTemplate::new().dna().template);
    let out = svg(Plot::Brick(dna));
    // Unknown characters get the grey used for unknown flank bases,
    // rgb(180,180,180).
    assert!(out.contains("#b4b4b4"));

    // No template at all (`new()` starts with an empty one, so only the public
    // field can be `None`): the bricks are grey and the legend is empty.
    let none = || {
        let mut bp = BrickPlot::new().with_sequences(seqs);
        bp.template = None;
        bp
    };
    assert!(svg(Plot::Brick(none())).contains("<svg"));
    assert!(collect_legend_entries(&[Plot::Brick(none())]).is_empty());
    let empty = BrickPlot::new()
        .with_sequences(seqs)
        .with_template(HashMap::new());
    assert!(svg(Plot::Brick(empty)).contains("<svg"));
}
