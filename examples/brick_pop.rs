//! BrickPopPlot documentation examples.
//!
//! Generates the canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example brick_pop
//! ```
//!
//! SVGs are written to `docs/src/assets/brick_pop/`.

use kuva::backend::svg::SvgBackend;
use kuva::plot::{BrickPlot, ColorMap};
use kuva::render::brick_pop::{BrickPopPlot, MetricColumn};
use std::collections::HashMap;

const OUT: &str = "docs/src/assets/brick_pop";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/brick_pop");
    population();
    values_in_box();
    cohort();
}

fn write(name: &str, svg: String) {
    std::fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
}

fn motif_colors() -> HashMap<String, String> {
    let mut c = HashMap::new();
    c.insert("AATGG".to_string(), "#4c78a8".to_string());
    c.insert("AAGGG".to_string(), "#e45756".to_string());
    c
}

/// The headline view: unique alleles at one STR locus, sorted by frequency, with a
/// frequency bar, two metric heatbox columns, colourbars, and a bottom motif legend.
fn population() {
    // Input is deliberately unsorted; `.sorted_by_frequency()` orders every panel together.
    let strigars = vec![
        ("AATGG:A".to_string(), "12A".to_string()),
        ("AATGG:A".to_string(), "40A".to_string()),
        ("AATGG:A,AAGGG:B".to_string(), "18A6B20A".to_string()),
        ("AATGG:A".to_string(), "8A".to_string()),
        ("AATGG:A".to_string(), "22A".to_string()),
        ("AATGG:A".to_string(), "5A".to_string()),
    ];
    let names = vec![
        "allele_a", "allele_b", "allele_c", "allele_d", "allele_e", "allele_f",
    ];
    let freqs = vec![0.12, 0.30, 0.20, 0.06, 0.25, 0.07];
    let methylation = vec![Some(0.7), Some(0.9), None, Some(0.3), Some(0.6), Some(0.2)];
    let entropy = vec![
        Some(0.10),
        Some(0.05),
        Some(0.80),
        Some(0.20),
        Some(0.10),
        Some(0.30),
    ];

    let brick = BrickPlot::new()
        .with_names(names)
        .with_motif_colors(motif_colors())
        .with_strigars(strigars);

    let scene = BrickPopPlot::new(brick)
        .with_title("RFC1 locus: population alleles")
        .with_frequencies(freqs)
        .with_metric("methylation", methylation, ColorMap::Viridis)
        .with_metric_column(
            MetricColumn::new("motif entropy", entropy, ColorMap::Inferno).with_range(0.0, 1.0),
        )
        .sorted_by_frequency()
        .render(880.0);

    write("population", SvgBackend::new().render_scene(&scene));
}

/// Value-in-box metric mode: the number is written in each cell with the heat on the
/// border. Best with a small number of alleles and tall rows.
fn values_in_box() {
    let strigars = vec![
        ("AATGG:A".to_string(), "24A".to_string()),
        ("AATGG:A".to_string(), "16A".to_string()),
        ("AATGG:A".to_string(), "9A".to_string()),
    ];
    let brick = BrickPlot::new()
        .with_names(vec!["allele_a", "allele_b", "allele_c"])
        .with_motif_colors(motif_colors())
        .with_strigars(strigars);

    let scene = BrickPopPlot::new(brick)
        .with_title("Metric values on the cell border")
        .with_row_height(28.0)
        .with_metric_values(true)
        .with_frequencies(vec![0.5, 0.3, 0.2])
        .with_metric(
            "methylation",
            vec![Some(0.86), Some(0.44), Some(0.12)],
            ColorMap::Viridis,
        )
        .with_metric_column(MetricColumn::new(
            "longest run",
            vec![Some(24.0), Some(16.0), Some(9.0)],
            ColorMap::Turbo,
        ))
        .render(760.0);

    write("values_in_box", SvgBackend::new().render_scene(&scene));
}

/// A larger cohort with long, merged expansions to show the ragged-right allele-size view
/// at scale. Run-length merging keeps very long alleles legible and cheap to draw.
fn cohort() {
    let n = 80usize;
    let mut strigars = Vec::with_capacity(n);
    let mut names = Vec::with_capacity(n);
    let mut freqs = Vec::with_capacity(n);
    let mut methylation = Vec::with_capacity(n);
    let mut entropy = Vec::with_capacity(n);
    let mut longest = Vec::with_capacity(n);

    for i in 0..n {
        let units = 480usize.saturating_sub(i * 5).max(6);
        if i % 8 == 3 {
            let a = units / 2;
            let c = units.saturating_sub(a + 6);
            strigars.push(("AATGG:A,AAGGG:B".to_string(), format!("{a}A6B{c}A")));
        } else {
            strigars.push(("AATGG:A".to_string(), format!("{units}A")));
        }
        names.push(format!("a{:02}", i + 1));
        freqs.push((n - i) as f64);
        methylation.push(if i % 6 == 0 {
            None
        } else {
            Some((i % 10) as f64 / 9.0)
        });
        entropy.push(Some(((i * 17) % 100) as f64 / 100.0));
        longest.push(Some(units as f64));
    }

    let brick = BrickPlot::new()
        .with_names(names)
        .with_motif_colors(motif_colors())
        .with_merge_runs(true)
        .with_strigars(strigars);

    let scene = BrickPopPlot::new(brick)
        .with_title("STR cohort: 80 alleles")
        .with_row_height(7.0)
        .with_frequency_label("Samples")
        .with_frequencies(freqs)
        .with_metric("methylation", methylation, ColorMap::Viridis)
        .with_metric_column(
            MetricColumn::new("motif entropy", entropy, ColorMap::Inferno).with_range(0.0, 1.0),
        )
        .with_metric("longest run", longest, ColorMap::Turbo)
        .render(1000.0);

    write("cohort", SvgBackend::new().render_scene(&scene));
}
