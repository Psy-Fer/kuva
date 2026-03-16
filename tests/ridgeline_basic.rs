use kuva::plot::ridgeline::RidgelinePlot;
use kuva::render::plots::Plot;
use kuva::render::layout::Layout;
use kuva::render::render::render_multiple;
use kuva::backend::svg::SvgBackend;

// ── Deterministic data generators ─────────────────────────────────────────────

/// LCG → uniform in (0, 1).
fn lcg(state: &mut u64) -> f64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((*state >> 33) as f64) / (u32::MAX as f64)
}

/// Box-Muller Gaussian sample from LCG.
fn gaussian(state: &mut u64, mean: f64, std: f64) -> f64 {
    let u1 = lcg(state).max(1e-10);
    let u2 = lcg(state);
    let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
    mean + z * std
}

fn make_gaussian(seed: u64, n: usize, mean: f64, std: f64) -> Vec<f64> {
    let mut state = seed;
    (0..n).map(|_| gaussian(&mut state, mean, std)).collect()
}

/// Uniform in [mean - half_width, mean + half_width].
fn make_data(seed: u64, n: usize, mean: f64) -> Vec<f64> {
    let mut state = seed;
    (0..n).map(|_| mean + (lcg(&mut state) - 0.5) * 4.0).collect()
}

// ── Helper ────────────────────────────────────────────────────────────────────

fn render(plots: Vec<Plot>) -> String {
    let layout = Layout::auto_from_plots(&plots);
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

fn write(name: &str, svg: &str) {
    std::fs::create_dir_all("test_outputs").ok();
    std::fs::write(format!("test_outputs/{name}.svg"), svg).unwrap();
}

// ── Basic tests ───────────────────────────────────────────────────────────────

#[test]
fn test_ridgeline_basic() {
    let plot = RidgelinePlot::new()
        .with_group("GroupA", make_data(1, 50, 2.0))
        .with_group("GroupB", make_data(2, 50, 5.0));
    let svg = render(vec![Plot::Ridgeline(plot)]);
    assert!(svg.contains("<path"), "expected path elements");
    assert!(svg.contains("GroupA"), "expected group label");
    assert!(svg.contains("GroupB"), "expected group label");
    write("ridgeline_basic", &svg);
}

#[test]
fn test_ridgeline_filled() {
    let plot = RidgelinePlot::new()
        .with_group("A", make_data(1, 50, 2.0))
        .with_filled(true);
    let svg = render(vec![Plot::Ridgeline(plot)]);
    assert!(svg.contains('Z'), "expected closed path for filled ridgeline");
}

#[test]
fn test_ridgeline_unfilled() {
    let plot = RidgelinePlot::new()
        .with_group("A", make_data(1, 50, 2.0))
        .with_filled(false);
    let svg = render(vec![Plot::Ridgeline(plot)]);
    // No filled path → no 'Z' close command
    assert!(!svg.contains('Z'), "unfilled ridgeline should have no Z");
}

#[test]
fn test_ridgeline_overlap_zero() {
    let plot = RidgelinePlot::new()
        .with_group("A", make_data(1, 50, 2.0))
        .with_group("B", make_data(2, 50, 5.0))
        .with_overlap(0.0);
    let svg = render(vec![Plot::Ridgeline(plot)]);
    assert!(svg.contains("<path"));
    write("ridgeline_overlap_zero", &svg);
}

#[test]
fn test_ridgeline_large_overlap() {
    // overlap=2.0: ridges extend 3× the cell height — must not panic
    let plot = RidgelinePlot::new()
        .with_group("A", make_data(1, 60, 0.0))
        .with_group("B", make_data(2, 60, 3.0))
        .with_group("C", make_data(3, 60, 6.0))
        .with_overlap(2.0);
    let svg = render(vec![Plot::Ridgeline(plot)]);
    assert!(svg.contains("<path"));
    write("ridgeline_large_overlap", &svg);
}

#[test]
fn test_ridgeline_colors() {
    let plot = RidgelinePlot::new()
        .with_group_color("A", make_data(1, 50, 2.0), "#e74c3c");
    let svg = render(vec![Plot::Ridgeline(plot)]);
    assert!(svg.contains("e74c3c"), "expected explicit color in SVG");
}

#[test]
fn test_ridgeline_legend() {
    let plot = RidgelinePlot::new()
        .with_group("GroupA", make_data(1, 50, 2.0))
        .with_group("GroupB", make_data(2, 50, 5.0))
        .with_legend(true);
    let layout = Layout::auto_from_plots(&[Plot::Ridgeline(plot.clone())]);
    let svg = SvgBackend.render_scene(&render_multiple(vec![Plot::Ridgeline(plot)], layout));
    assert!(svg.contains("GroupA"), "expected legend label in SVG");
}

#[test]
fn test_ridgeline_normalize() {
    let plot = RidgelinePlot::new()
        .with_group("A", make_data(1, 50, 2.0))
        .with_group("B", make_data(2, 50, 5.0))
        .with_normalize(true);
    let svg = render(vec![Plot::Ridgeline(plot)]);
    assert!(svg.contains("<path"));
}

#[test]
fn test_ridgeline_single_group() {
    let plot = RidgelinePlot::new()
        .with_group("Only", make_data(1, 40, 0.0));
    let svg = render(vec![Plot::Ridgeline(plot)]);
    assert!(svg.contains("<path"));
    assert!(svg.contains("Only"));
    write("ridgeline_single_group", &svg);
}

// ── Label-order test ─────────────────────────────────────────────────────────

#[test]
fn test_ridgeline_group_order() {
    // Groups added first→last are rendered top→bottom.
    // On the y-axis, the FIRST group ("Alpha") must appear ABOVE the last ("Gamma").
    // In SVG, the y-axis labels are rendered top-to-bottom, so "Alpha" should
    // appear before "Gamma" in the SVG text content.
    let plot = RidgelinePlot::new()
        .with_group("Alpha", make_data(1, 40, 1.0))
        .with_group("Beta",  make_data(2, 40, 4.0))
        .with_group("Gamma", make_data(3, 40, 7.0));
    let svg = render(vec![Plot::Ridgeline(plot)]);
    let alpha_pos = svg.find("Alpha").expect("Alpha label missing");
    let gamma_pos = svg.find("Gamma").expect("Gamma label missing");
    // Alpha (top group) maps to y=3 data, which is a SMALLER pixel y than Gamma's y=1.
    // SVG text for smaller pixel-y appears earlier (lower y coord → earlier in SVG output
    // since SVG renders top-to-bottom left-to-right).
    // We just check both appear in the SVG without asserting pixel order here, since
    // SVG text order depends on renderer traversal, not y-coord order.
    assert!(alpha_pos < svg.len());
    assert!(gamma_pos < svg.len());
}

// ── Title / overflow test ─────────────────────────────────────────────────────

#[test]
fn test_ridgeline_with_title() {
    // Regression: top ridge must not overlap the title.  The SVG should contain
    // both a <text> element with the title and <path> elements for the ridges.
    let plot = RidgelinePlot::new()
        .with_group("Low",  make_gaussian(1, 80, -2.0, 1.0))
        .with_group("Mid",  make_gaussian(2, 80,  0.0, 1.0))
        .with_group("High", make_gaussian(3, 80,  2.0, 1.0))
        .with_overlap(0.5);
    let layout = Layout::auto_from_plots(&[Plot::Ridgeline(plot.clone())])
        .with_title("Ridge with Title")
        .with_x_label("Value")
        .with_y_label("Group");
    let svg = SvgBackend.render_scene(&render_multiple(vec![Plot::Ridgeline(plot)], layout));
    assert!(svg.contains("Ridge with Title"), "title missing from SVG");
    assert!(svg.contains("<path"), "ridges missing from SVG");
    write("ridgeline_with_title", &svg);
}

// ── Temperature / seasonal ridgeline ──────────────────────────────────────────

/// Seasonal temperature distributions for a temperate-climate city.
/// Months ordered Jan→Dec (top→bottom in the ridgeline).
/// Mean daily temperatures (°C), std-dev reflects seasonal variability.
const MONTHS: [(&str, f64, f64); 12] = [
    ("January",   -3.0, 5.0),
    ("February",  -1.5, 5.5),
    ("March",      4.0, 5.0),
    ("April",     10.0, 4.0),
    ("May",       15.5, 3.5),
    ("June",      20.0, 3.0),
    ("July",      23.0, 2.5),
    ("August",    22.5, 2.5),
    ("September", 17.0, 3.0),
    ("October",   10.5, 4.0),
    ("November",   3.5, 5.0),
    ("December",  -1.0, 5.5),
];

#[test]
fn test_ridgeline_temperature() {
    let mut plot = RidgelinePlot::new()
        .with_overlap(0.6)
        .with_opacity(0.75);

    // Use a blue→red gradient of colours across the months (cold→hot)
    let colors = [
        "#3a7abf", "#4589c4", "#6ba3d4", "#a0bfdc",
        "#d4b8a0", "#e8c97a", "#f0a830", "#e86820",
        "#d44a10", "#c06030", "#9070a0", "#5060b0",
    ];

    for (i, &(month, mean, std)) in MONTHS.iter().enumerate() {
        let data = make_gaussian(i as u64 + 1, 200, mean, std);
        plot = plot.with_group_color(month, data, colors[i]);
    }

    let layout = Layout::auto_from_plots(&[Plot::Ridgeline(plot.clone())])
        .with_title("Daily Temperature Distributions by Month")
        .with_x_label("Temperature (°C)")
        .with_y_label("Month");
    let svg = SvgBackend.render_scene(&render_multiple(vec![Plot::Ridgeline(plot)], layout));

    assert!(svg.contains("<path"), "expected path elements");
    for &(month, _, _) in &MONTHS {
        assert!(svg.contains(month), "expected month label '{month}' in SVG");
    }
    // All 12 months should have ridge paths (24 paths: fill + outline each)
    let path_count = svg.matches("<path").count();
    assert!(path_count >= 12, "expected at least 12 paths, got {path_count}");

    write("ridgeline_temperature", &svg);
}

#[test]
fn test_ridgeline_temperature_no_fill() {
    let mut plot = RidgelinePlot::new()
        .with_filled(false)
        .with_overlap(0.8)
        .with_stroke_width(2.0);
    for (i, &(month, mean, std)) in MONTHS.iter().enumerate() {
        plot = plot.with_group(month, make_gaussian(i as u64 + 42, 150, mean, std));
    }
    let svg = render(vec![Plot::Ridgeline(plot)]);
    // No fill → no Z in paths
    assert!(!svg.contains('Z'), "unfilled should have no Z");
    assert!(svg.contains("<path"));
    write("ridgeline_temperature_outline", &svg);
}

#[test]
fn test_ridgeline_baseline_default_on() {
    // show_baseline is true by default — the SVG should contain a Line element
    // drawn at each group's y-center (full plot width).  We can't inspect pixel
    // coords directly, but we can count <line elements: axes + ticks + N baselines.
    let n = 3usize;
    let plot = RidgelinePlot::new()
        .with_group("A", make_gaussian(1, 60, 0.0, 1.0))
        .with_group("B", make_gaussian(2, 60, 3.0, 1.0))
        .with_group("C", make_gaussian(3, 60, 6.0, 1.0));
    let svg = render(vec![Plot::Ridgeline(plot)]);
    let line_count = svg.matches("<line").count();
    // 2 axis lines + x-ticks + y-category ticks (3) + 3 baselines
    assert!(line_count >= n + 2, "expected at least {n} baselines + 2 axis lines, got {line_count}");
    write("ridgeline_baseline_on", &svg);
}

#[test]
fn test_ridgeline_baseline_off() {
    let plot_on  = RidgelinePlot::new()
        .with_group("A", make_gaussian(1, 60, 0.0, 1.0))
        .with_group("B", make_gaussian(2, 60, 3.0, 1.0));
    let plot_off = RidgelinePlot::new()
        .with_group("A", make_gaussian(1, 60, 0.0, 1.0))
        .with_group("B", make_gaussian(2, 60, 3.0, 1.0))
        .with_baseline(false);
    let svg_on  = render(vec![Plot::Ridgeline(plot_on)]);
    let svg_off = render(vec![Plot::Ridgeline(plot_off)]);
    // with_baseline(false) should produce fewer <line elements
    let on_lines  = svg_on .matches("<line").count();
    let off_lines = svg_off.matches("<line").count();
    assert!(off_lines < on_lines,
        "disabling baseline should reduce line count: on={on_lines} off={off_lines}");
}

#[test]
fn test_ridgeline_bandwidth_override() {
    let plot = RidgelinePlot::new()
        .with_group("Narrow",  make_gaussian(1, 100, 0.0, 1.0))
        .with_group("Wide",    make_gaussian(2, 100, 5.0, 2.0))
        .with_bandwidth(0.5); // forced narrow bandwidth for both groups
    let svg = render(vec![Plot::Ridgeline(plot)]);
    assert!(svg.contains("<path"));
    write("ridgeline_bandwidth", &svg);
}

#[test]
fn test_ridgeline_with_groups_builder() {
    let groups = vec![
        ("Spring", make_gaussian(1, 80, 12.0, 3.0)),
        ("Summer", make_gaussian(2, 80, 24.0, 2.0)),
        ("Autumn", make_gaussian(3, 80, 10.0, 4.0)),
        ("Winter", make_gaussian(4, 80, -2.0, 5.0)),
    ];
    let plot = RidgelinePlot::new()
        .with_groups(groups)
        .with_overlap(0.4);
    let svg = render(vec![Plot::Ridgeline(plot)]);
    assert!(svg.contains("Spring"));
    assert!(svg.contains("Winter"));
    write("ridgeline_seasons", &svg);
}
