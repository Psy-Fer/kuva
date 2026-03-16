//! Strip plot documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example strip
//! ```
//!
//! SVGs are written to `docs/src/assets/strip/`.

use rand::SeedableRng;
use rand_distr::{Distribution, Normal, Exp};
use kuva::plot::{StripPlot, BoxPlot, LegendEntry, LegendShape, LegendPosition};
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::Palette;

const OUT: &str = "docs/src/assets/strip";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/strip");

    basic();
    swarm();
    center();
    composed();
    palette();
    group_colors();
    marker_density();
    point_colors();

    println!("Strip SVGs written to {OUT}/");
}

fn normal_samples(mean: f64, std: f64, n: usize, seed: u64) -> Vec<f64> {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
    Normal::new(mean, std).unwrap()
        .sample_iter(&mut rng)
        .take(n)
        .collect()
}

fn bimodal_samples(mean1: f64, mean2: f64, std: f64, n: usize, seed: u64) -> Vec<f64> {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
    let d1 = Normal::new(mean1, std).unwrap();
    let d2 = Normal::new(mean2, std).unwrap();
    let half = n / 2;
    d1.sample_iter(&mut rng.clone()).take(half)
        .chain(d2.sample_iter(&mut rng).take(n - half))
        .collect()
}

fn skewed_samples(scale: f64, shift: f64, n: usize, seed: u64) -> Vec<f64> {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
    Exp::new(1.0 / scale).unwrap()
        .sample_iter(&mut rng)
        .take(n)
        .map(|x| x + shift)
        .collect()
}

/// Jittered strip — 300 points per group showing varied distribution shapes.
///
/// With this many points the jitter cloud fills out visibly, making it easy
/// to compare spread and location across groups.
fn basic() {
    let strip = StripPlot::new()
        .with_group("Control",   normal_samples(5.0, 0.8, 300, 1))
        .with_group("Low dose",  normal_samples(6.5, 1.1, 300, 2))
        .with_group("High dose", normal_samples(8.0, 1.4, 300, 3))
        .with_group("Washout",   normal_samples(5.8, 0.9, 300, 4))
        .with_color("steelblue")
        .with_point_size(2.5)
        .with_jitter(0.35);

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Jittered Strip Plot")
        .with_y_label("Measurement");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/basic.svg"), svg).unwrap();
}

/// Beeswarm — 150 points per group, non-overlapping horizontal spread.
///
/// Swarm works best for N < ~200 per group; with 150 points the structure of
/// each distribution is clearly revealed without points running off the edges.
fn swarm() {
    let strip = StripPlot::new()
        .with_group("Control",     normal_samples(0.0, 1.0, 150, 10))
        .with_group("Bimodal",     bimodal_samples(-1.5, 1.5, 0.55, 150, 11))
        .with_group("Right-skewed", skewed_samples(1.2, -0.5, 150, 12))
        .with_color("steelblue")
        .with_point_size(3.0)
        .with_swarm();

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Beeswarm")
        .with_y_label("Value");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/swarm.svg"), svg).unwrap();
}

/// Center stack — 400 points per group, all at the group midpoint.
///
/// Points stack directly on top of each other at x = group centre. With 400
/// points the vertical column makes local density immediately obvious.
fn center() {
    let strip = StripPlot::new()
        .with_group("Normal",  normal_samples(5.0, 1.0, 400, 20))
        .with_group("Bimodal", bimodal_samples(2.0, 8.0, 0.8, 400, 21))
        .with_group("Skewed",  skewed_samples(1.5, 1.0, 400, 22))
        .with_color("steelblue")
        .with_point_size(2.0)
        .with_center();

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Center Stack")
        .with_y_label("Value");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/center.svg"), svg).unwrap();
}

/// BoxPlot + StripPlot composed on the same axes.
///
/// 80 points per group — enough to make the jitter cloud visible while
/// keeping individual points distinguishable behind the box summary.
fn composed() {
    let data_a = normal_samples(4.0, 1.0, 80, 30);
    let data_b = bimodal_samples(2.5, 6.5, 0.7, 80, 31);
    let data_c = normal_samples(5.5, 1.6, 80, 32);

    let boxplot = BoxPlot::new()
        .with_group("Control",     data_a.clone())
        .with_group("Bimodal",     data_b.clone())
        .with_group("High-spread", data_c.clone())
        .with_color("steelblue");

    let strip = StripPlot::new()
        .with_group("Control",     data_a)
        .with_group("Bimodal",     data_b)
        .with_group("High-spread", data_c)
        .with_color("rgba(0,0,0,0.3)")
        .with_point_size(2.5)
        .with_jitter(0.2);

    let plots = vec![Plot::Box(boxplot), Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Box + Strip")
        .with_y_label("Value");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/composed.svg"), svg).unwrap();
}

/// Per-group colors — each group gets its own color via `.with_group_colors()`.
fn group_colors() {
    let strip = StripPlot::new()
        .with_group("Control",   vec![4.1, 5.0, 5.3, 5.8, 6.2, 4.7])
        .with_group("Treatment", vec![5.5, 6.1, 6.4, 7.2, 7.8, 6.9])
        .with_group("Placebo",   vec![3.9, 4.5, 4.8, 5.1, 5.6, 4.3])
        .with_group_colors(vec!["steelblue", "crimson", "seagreen"])
        .with_point_size(4.0)
        .with_jitter(0.3);

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Per-Group Colors")
        .with_y_label("Measurement");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/group_colors.svg"), svg).unwrap();
}

/// 500 points per group with semi-transparent fill + stroke.
///
/// At this density, solid markers pile into an opaque block and the
/// distribution shape is lost. Reducing opacity lets the darker bands reveal
/// where points accumulate, while the stroke keeps individual points legible.
fn marker_density() {
    let strip = StripPlot::new()
        .with_group("Control",   normal_samples(5.0, 0.8, 500, 50))
        .with_group("Low dose",  normal_samples(6.0, 1.2, 500, 51))
        .with_group("High dose", bimodal_samples(4.5, 8.5, 0.6, 500, 52))
        .with_group("Washout",   skewed_samples(1.2, 3.5, 500, 53))
        .with_color("steelblue")
        .with_point_size(4.0)
        .with_jitter(0.3)
        .with_marker_opacity(0.25)
        .with_marker_stroke_width(0.7);

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Dense strip — semi-transparent markers (500 pts/group)")
        .with_y_label("Measurement");

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/marker_density.svg"), svg).unwrap();
}

/// Per-point colors — each point carries its own color via `.with_colored_group()`.
///
/// Simulates a STR genotyping view where each point is a single read and the color
/// identifies the primary repeat motif. A manual legend maps motif labels to colors.
fn point_colors() {
    // Motif categories with representative repeat-count distributions
    let motifs: &[(&str, &str, f64, f64)] = &[
        ("ATTC",  "tomato",       6.0, 1.2),
        ("GCGC",  "seagreen",     9.0, 1.5),
        ("ATAT",  "goldenrod",    4.5, 0.9),
        ("CGCG",  "mediumpurple", 11.0, 1.8),
        ("TTAGG", "steelblue",    7.5, 1.3),
    ];

    let mut rng = rand::rngs::SmallRng::seed_from_u64(99);

    // Build flat (value, color) list — ~25 reads per motif
    let mut points: Vec<(f64, &str)> = Vec::new();
    for &(_, color, mean, std) in motifs {
        let dist = Normal::new(mean, std).unwrap();
        for v in dist.sample_iter(&mut rng).take(25) {
            points.push((v.max(1.0), color));
        }
    }
    // Shuffle deterministically so motifs interleave in the column
    use rand::seq::SliceRandom;
    points.shuffle(&mut rng);

    let strip = StripPlot::new()
        .with_colored_group("Sample", points)
        .with_swarm()
        .with_point_size(4.5);

    // Manual legend — one circle swatch per motif
    let legend_entries: Vec<LegendEntry> = motifs.iter().map(|&(label, color, _, _)| {
        LegendEntry { label: label.to_string(), color: color.to_string(), shape: LegendShape::Circle, dasharray: None }
    }).collect();

    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("STR Repeat Counts — Per-point Motif Colors")
        .with_x_label("Sample")
        .with_y_label("Repeat count")
        .with_legend_title("Motif")
        .with_legend_entries(legend_entries)
        .with_legend_position(LegendPosition::OutsideRightTop);

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/point_colors.svg"), svg).unwrap();
}

/// Multiple StripPlots with a palette — each plot gets a distinct color.
fn palette() {
    let wt  = StripPlot::new()
        .with_group("WT",  normal_samples(5.0, 0.9, 200, 40))
        .with_group("HET", normal_samples(6.2, 1.0, 200, 41))
        .with_group("KO",  normal_samples(7.8, 1.3, 200, 42))
        .with_jitter(0.3)
        .with_point_size(2.5)
        .with_legend("Line A");

    let ko = StripPlot::new()
        .with_group("WT",  normal_samples(5.3, 0.9, 200, 43))
        .with_group("HET", normal_samples(6.8, 1.0, 200, 44))
        .with_group("KO",  normal_samples(8.4, 1.1, 200, 45))
        .with_jitter(0.3)
        .with_point_size(2.5)
        .with_legend("Line B");

    let plots = vec![Plot::Strip(wt), Plot::Strip(ko)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Two Lines – Palette")
        .with_y_label("Expression")
        .with_palette(Palette::wong());

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/palette.svg"), svg).unwrap();
}
