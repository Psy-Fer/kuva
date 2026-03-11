use kuva::plot::ternary::TernaryPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use kuva::backend::svg::SvgBackend;

fn render(plot: TernaryPlot) -> String {
    let plots = vec![Plot::Ternary(plot)];
    let layout = Layout::auto_from_plots(&plots);
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

fn render_titled(plot: TernaryPlot, title: &str) -> String {
    let plots = vec![Plot::Ternary(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title(title);
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

fn write(name: &str, svg: &str) {
    std::fs::create_dir_all("test_outputs").ok();
    std::fs::write(format!("test_outputs/{name}.svg"), svg).unwrap();
}

// ── basic ──────────────────────────────────────────────────────────────────────

#[test]
fn test_ternary_basic() {
    let plot = TernaryPlot::new()
        .with_point(0.7, 0.2, 0.1)
        .with_point(0.1, 0.7, 0.2)
        .with_point(0.2, 0.1, 0.7);
    let svg = render_titled(plot, "Ternary Basic");
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path"));
    write("ternary_basic", &svg);
}

// ── groups / legend ────────────────────────────────────────────────────────────

#[test]
fn test_ternary_groups() {
    let plot = TernaryPlot::new()
        .with_point_group(0.7, 0.2, 0.1, "A-rich")
        .with_point_group(0.1, 0.7, 0.2, "B-rich")
        .with_point_group(0.2, 0.1, 0.7, "C-rich")
        .with_legend(true);
    let plots = vec![Plot::Ternary(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Ternary Groups");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert!(svg.contains("<svg"));
    assert!(svg.contains("A-rich"));
    assert!(svg.contains("B-rich"));
    assert!(svg.contains("C-rich"));
    write("ternary_groups", &svg);
}

// ── normalize ─────────────────────────────────────────────────────────────────

#[test]
fn test_ternary_normalize() {
    let plot = TernaryPlot::new()
        .with_point(70.0, 20.0, 10.0)
        .with_point(10.0, 70.0, 20.0)
        .with_normalize(true);
    let svg = render_titled(plot, "Ternary Normalize");
    assert!(svg.contains("<svg"));
    write("ternary_normalize", &svg);
}

// ── grid ──────────────────────────────────────────────────────────────────────

#[test]
fn test_ternary_grid() {
    let plot_with_grid = TernaryPlot::new()
        .with_point(0.33, 0.33, 0.34)
        .with_grid(true)
        .with_grid_lines(5);
    let plot_no_grid = TernaryPlot::new()
        .with_point(0.33, 0.33, 0.34)
        .with_grid(false);

    let svg_with = render(plot_with_grid);
    let svg_without = render(plot_no_grid);

    let count_with = svg_with.matches("<path").count();
    let count_without = svg_without.matches("<path").count();
    assert!(count_with > count_without, "Grid should add more paths");
    write("ternary_grid", &svg_with);
    write("ternary_no_grid", &svg_without);
}

#[test]
fn test_ternary_fine_grid() {
    // Dense 10-line grid — stress test rendering with many lines
    let plot = TernaryPlot::new()
        .with_point(0.5, 0.3, 0.2)
        .with_grid(true)
        .with_grid_lines(10);
    let svg = render_titled(plot, "Ternary Fine Grid");
    assert!(svg.contains("<svg"));
    // 10 lines per axis = 30 grid paths; coarse check: more than basic outline
    let fine_count = svg.matches("<path").count();
    let coarse = TernaryPlot::new()
        .with_point(0.5, 0.3, 0.2)
        .with_grid(true)
        .with_grid_lines(4);
    let coarse_count = render(coarse).matches("<path").count();
    assert!(fine_count > coarse_count, "10-line grid should have more paths than 4-line grid");
    write("ternary_fine_grid", &svg);
}

// ── corner labels ─────────────────────────────────────────────────────────────

#[test]
fn test_ternary_corner_labels() {
    let plot = TernaryPlot::new()
        .with_point(0.5, 0.3, 0.2)
        .with_corner_labels("Silicon", "Oxygen", "Carbon");
    let svg = render_titled(plot, "Ternary Corner Labels");
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Silicon"));
    assert!(svg.contains("Oxygen"));
    assert!(svg.contains("Carbon"));
    write("ternary_corner_labels", &svg);
}

// ── percentages ───────────────────────────────────────────────────────────────

#[test]
fn test_ternary_no_percentages() {
    let plot_with_pct = TernaryPlot::new()
        .with_point(0.5, 0.3, 0.2)
        .with_percentages(true);
    let plot_no_pct = TernaryPlot::new()
        .with_point(0.5, 0.3, 0.2)
        .with_percentages(false);

    let svg_with = render(plot_with_pct);
    let svg_without = render(plot_no_pct);

    let count_with = svg_with.matches("<text").count();
    let count_without = svg_without.matches("<text").count();
    assert!(count_with > count_without, "show_percentages=true should add more text elements");
    write("ternary_percentages", &svg_with);
    write("ternary_no_percentages", &svg_without);
}

// ── axis corner correctness ────────────────────────────────────────────────────
// A point at exactly one corner should produce a single marker at that vertex.
// We don't test exact pixels, but the SVG should render without panic and contain
// marker elements (circle or path) for all three pure-component extremes.
#[test]
fn test_ternary_vertices() {
    let plot = TernaryPlot::new()
        .with_point(1.0, 0.0, 0.0) // should land at top (A vertex)
        .with_point(0.0, 1.0, 0.0) // should land at bottom-left (B vertex)
        .with_point(0.0, 0.0, 1.0) // should land at bottom-right (C vertex)
        .with_grid(false)
        .with_percentages(false);
    let svg = render_titled(plot, "Ternary Vertices");
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path") || svg.contains("<circle"));
    write("ternary_vertices", &svg);
}

// ── soil texture diagram ───────────────────────────────────────────────────────
// Classic three-group dataset (Sand / Silt / Clay fractions).
// Raw counts are normalized so a+b+c=1 doesn't need to hold in input.
#[test]
fn test_ternary_soil_texture() {
    // sand, silt, clay — real-ish soil texture classes
    let samples: &[(f64, f64, f64, &str)] = &[
        // Sandy loam cluster
        (70.0, 20.0, 10.0, "Sandy Loam"),
        (65.0, 25.0, 10.0, "Sandy Loam"),
        (72.0, 18.0, 10.0, "Sandy Loam"),
        (68.0, 22.0, 10.0, "Sandy Loam"),
        (74.0, 16.0, 10.0, "Sandy Loam"),
        // Clay loam cluster
        (30.0, 35.0, 35.0, "Clay Loam"),
        (25.0, 38.0, 37.0, "Clay Loam"),
        (33.0, 32.0, 35.0, "Clay Loam"),
        (28.0, 36.0, 36.0, "Clay Loam"),
        (32.0, 33.0, 35.0, "Clay Loam"),
        // Silty loam cluster
        (20.0, 60.0, 20.0, "Silty Loam"),
        (18.0, 63.0, 19.0, "Silty Loam"),
        (22.0, 58.0, 20.0, "Silty Loam"),
        (19.0, 62.0, 19.0, "Silty Loam"),
        (21.0, 60.0, 19.0, "Silty Loam"),
    ];
    let mut plot = TernaryPlot::new()
        .with_corner_labels("Sand", "Silt", "Clay")
        .with_normalize(true)
        .with_grid_lines(5)
        .with_legend(true);
    for &(a, b, c, grp) in samples {
        plot = plot.with_point_group(a, b, c, grp);
    }
    let plots = vec![Plot::Ternary(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Soil Texture Diagram");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Sandy Loam"));
    assert!(svg.contains("Clay Loam"));
    assert!(svg.contains("Silty Loam"));
    assert!(svg.contains("Sand"));
    assert!(svg.contains("Silt"));
    assert!(svg.contains("Clay"));
    write("ternary_soil_texture", &svg);
}

// ── geochemistry / mineral composition ────────────────────────────────────────
// Feldspar ternary: Or (orthoclase), Ab (albite), An (anorthite)
#[test]
fn test_ternary_feldspar() {
    // Deterministic pseudo-random points using simple LCG
    let mut state: u64 = 12345;
    let mut lcg = || -> f64 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (state >> 33) as f64 / (u64::MAX >> 33) as f64
    };

    let groups = ["Alkali Feldspar", "Plagioclase", "Intermediate"];
    // Alkali: high Or (A)
    // Plagioclase: high An (C)
    // Intermediate: mixed
    let biases: &[(f64, f64, f64)] = &[
        (0.70, 0.20, 0.10), // Or-rich
        (0.10, 0.20, 0.70), // An-rich
        (0.33, 0.34, 0.33), // mixed
    ];

    let mut plot = TernaryPlot::new()
        .with_corner_labels("Or", "Ab", "An")
        .with_grid_lines(5)
        .with_legend(true);

    for (i, &(ba, bb, bc)) in biases.iter().enumerate() {
        for _ in 0..12 {
            let da = (lcg() - 0.5) * 0.12;
            let db = (lcg() - 0.5) * 0.12;
            let a = (ba + da).clamp(0.01, 0.98);
            let b = (bb + db).clamp(0.01, 0.98);
            let c = (bc - da - db).clamp(0.01, 0.98);
            plot = plot.with_point_group(a, b, c, groups[i]);
        }
    }

    let plots = vec![Plot::Ternary(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Feldspar Ternary");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Alkali Feldspar"));
    assert!(svg.contains("Plagioclase"));
    assert!(svg.contains("Or"));
    assert!(svg.contains("Ab"));
    assert!(svg.contains("An"));
    write("ternary_feldspar", &svg);
}

// ── four groups with custom marker size ───────────────────────────────────────
#[test]
fn test_ternary_four_groups_large_markers() {
    let samples: &[(f64, f64, f64, &str)] = &[
        (0.80, 0.10, 0.10, "Alpha"),
        (0.75, 0.15, 0.10, "Alpha"),
        (0.10, 0.80, 0.10, "Beta"),
        (0.12, 0.75, 0.13, "Beta"),
        (0.10, 0.10, 0.80, "Gamma"),
        (0.13, 0.12, 0.75, "Gamma"),
        (0.33, 0.33, 0.34, "Delta"),
        (0.30, 0.35, 0.35, "Delta"),
    ];
    let mut plot = TernaryPlot::new()
        .with_marker_size(8.0)
        .with_legend(true)
        .with_grid_lines(4);
    for &(a, b, c, grp) in samples {
        plot = plot.with_point_group(a, b, c, grp);
    }
    let svg = render_titled(plot, "Four Groups Large Markers");
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Alpha"));
    assert!(svg.contains("Beta"));
    assert!(svg.contains("Gamma"));
    assert!(svg.contains("Delta"));
    write("ternary_four_groups", &svg);
}

// ── normalize with raw count data ─────────────────────────────────────────────
#[test]
fn test_ternary_normalize_counts() {
    // Raw read counts — wildly different scales, normalize=true projects to simplex
    let samples: &[(f64, f64, f64, &str)] = &[
        (1200.0, 300.0, 100.0, "High-A"),
        (1500.0, 250.0, 80.0,  "High-A"),
        (900.0,  400.0, 120.0, "High-A"),
        (80.0,   1400.0, 200.0, "High-B"),
        (100.0,  1600.0, 180.0, "High-B"),
        (90.0,   1200.0, 250.0, "High-B"),
        (150.0,  200.0,  2000.0, "High-C"),
        (120.0,  180.0,  1800.0, "High-C"),
        (130.0,  220.0,  2200.0, "High-C"),
    ];
    let mut plot = TernaryPlot::new()
        .with_normalize(true)
        .with_legend(true)
        .with_grid_lines(5)
        .with_corner_labels("GeneA", "GeneB", "GeneC");
    for &(a, b, c, grp) in samples {
        plot = plot.with_point_group(a, b, c, grp);
    }
    let plots = vec![Plot::Ternary(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Normalized Count Data");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert!(svg.contains("<svg"));
    assert!(svg.contains("High-A"));
    assert!(svg.contains("GeneA"));
    write("ternary_normalize_counts", &svg);
}

// ── no percentages with dense dataset ────────────────────────────────────────
#[test]
fn test_ternary_dense_no_pct() {
    let mut state: u64 = 99991;
    let mut lcg = || -> f64 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (state >> 33) as f64 / (u64::MAX >> 33) as f64
    };

    let mut plot = TernaryPlot::new()
        .with_percentages(false)
        .with_grid_lines(5)
        .with_marker_size(4.0);

    // 50 uniformly scattered points — no groups, no percentages
    for _ in 0..50 {
        let a = lcg();
        let b = lcg() * (1.0 - a);
        let c = 1.0 - a - b;
        plot = plot.with_point(a, b, c);
    }
    let svg = render_titled(plot, "Dense No Percentages");
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path") || svg.contains("<circle"));
    write("ternary_dense_no_pct", &svg);
}
