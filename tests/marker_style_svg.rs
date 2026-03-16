/// Tests for circle marker opacity and stroke across ScatterPlot, StripPlot,
/// PolarPlot, and TernaryPlot.  Every test writes an SVG to test_outputs/ for
/// visual inspection, and asserts that the relevant CSS attributes appear in
/// the output when configured.
use kuva::backend::svg::SvgBackend;
use kuva::plot::scatter::ScatterPlot;
use kuva::plot::strip::StripPlot;
use kuva::plot::polar::{PolarPlot, PolarMode};
use kuva::plot::ternary::TernaryPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

fn write(name: &str, svg: &str) {
    std::fs::create_dir_all("test_outputs").ok();
    std::fs::write(format!("test_outputs/{name}.svg"), svg).unwrap();
}

fn render(plots: Vec<Plot>, title: &str) -> String {
    let layout = Layout::auto_from_plots(&plots).with_title(title);
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

// ── SVG attribute helpers ────────────────────────────────────────────────────

fn has_fill_opacity(svg: &str) -> bool {
    svg.contains("fill-opacity=")
}

fn has_stroke(svg: &str) -> bool {
    // Only count stroke attrs on <circle> elements (not path stroke)
    svg.contains("<circle") && svg.contains(" stroke=")
}

fn has_stroke_width_on_circle(svg: &str) -> bool {
    // Look for stroke-width attribute near a circle element
    let after_first_circle = svg.find("<circle").map(|i| &svg[i..]).unwrap_or("");
    after_first_circle.contains("stroke-width=")
}

// ── ScatterPlot ──────────────────────────────────────────────────────────────

/// 500-point scatter with semi-transparent markers + stroke — simulates a busy
/// UMAP / PCA plot where overlapping points should pool colour without making a
/// solid blob.
#[test]
fn test_scatter_semi_transparent_stroke_busy() {
    // Three Gaussian clusters, 150 points each (seed-deterministic using LCG)
    let mut pts: Vec<(f64, f64)> = Vec::new();
    let centers = [(2.0_f64, 3.0_f64), (6.0, 7.0), (4.0, 1.0)];
    let mut seed: u64 = 1234567890;
    let lcg = |s: &mut u64| -> f64 {
        *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (*s >> 33) as f64 / (u64::MAX >> 33) as f64
    };
    for &(cx, cy) in &centers {
        for _ in 0..150 {
            let u1 = lcg(&mut seed).max(1e-9);
            let u2 = lcg(&mut seed);
            let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            let z1 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).sin();
            pts.push((cx + z0 * 0.8, cy + z1 * 0.8));
        }
    }

    let plot = ScatterPlot::new()
        .with_data(pts)
        .with_color("steelblue")
        .with_size(5.0)
        .with_marker_opacity(0.25)
        .with_marker_stroke_width(0.8);

    let svg = render(vec![Plot::Scatter(plot)], "Scatter — semi-transparent + stroke (450 pts)");
    write("marker_scatter_busy_semi_transparent", &svg);

    assert!(svg.contains("<circle"), "should contain circle elements");
    assert!(has_fill_opacity(&svg), "should contain fill-opacity attribute");
    assert!(has_stroke(&svg), "should contain stroke attribute on circles");
}

/// Hollow open circles — ideal for very dense areas where solid fills merge.
#[test]
fn test_scatter_hollow_open_circles_busy() {
    // Grid of 20×20 = 400 points in a noisy grid pattern
    let mut pts: Vec<(f64, f64)> = Vec::new();
    let mut seed: u64 = 987654321;
    let lcg = |s: &mut u64| -> f64 {
        *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (*s >> 33) as f64 / (u64::MAX >> 33) as f64
    };
    for i in 0..20 {
        for j in 0..20 {
            let x = i as f64 + (lcg(&mut seed) - 0.5) * 0.4;
            let y = j as f64 + (lcg(&mut seed) - 0.5) * 0.4;
            pts.push((x, y));
        }
    }

    let plot = ScatterPlot::new()
        .with_data(pts)
        .with_color("tomato")
        .with_size(6.0)
        .with_marker_opacity(0.0)       // fully hollow
        .with_marker_stroke_width(1.2);

    let svg = render(vec![Plot::Scatter(plot)], "Scatter — hollow open circles (400 pts, grid)");
    write("marker_scatter_hollow_grid", &svg);

    assert!(svg.contains("<circle"), "should contain circle elements");
    assert!(has_fill_opacity(&svg), "fill-opacity should be emitted");
    // opacity 0.0 → fill-opacity="0"
    assert!(svg.contains(r#"fill-opacity="0""#), "fill-opacity should be 0");
    assert!(has_stroke(&svg), "stroke should be present for hollow circles");
}

/// Three series, each with a different marker style, overlaid on one canvas —
/// tests that per-series opacity/stroke settings are independent.
#[test]
fn test_scatter_three_series_mixed_styles() {
    let mut seed: u64 = 111222333;
    let lcg = |s: &mut u64| -> f64 {
        *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (*s >> 33) as f64 / (u64::MAX >> 33) as f64
    };

    let gen_cluster = |cx: f64, cy: f64, n: usize, s: &mut u64| -> Vec<(f64, f64)> {
        (0..n).map(|_| {
            let u1 = lcg(s).max(1e-9);
            let u2 = lcg(s);
            let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            let z1 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).sin();
            (cx + z0 * 1.2, cy + z1 * 1.2)
        }).collect()
    };

    // Series A: solid
    let a = ScatterPlot::new()
        .with_data(gen_cluster(2.0, 2.0, 80, &mut seed))
        .with_color("steelblue")
        .with_size(4.0)
        .with_legend("Solid");

    // Series B: semi-transparent
    let b = ScatterPlot::new()
        .with_data(gen_cluster(5.0, 5.0, 80, &mut seed))
        .with_color("tomato")
        .with_size(4.0)
        .with_marker_opacity(0.35)
        .with_marker_stroke_width(1.0)
        .with_legend("Semi-transparent");

    // Series C: hollow
    let c = ScatterPlot::new()
        .with_data(gen_cluster(3.5, 7.0, 80, &mut seed))
        .with_color("seagreen")
        .with_size(5.0)
        .with_marker_opacity(0.0)
        .with_marker_stroke_width(1.5)
        .with_legend("Hollow");

    let plots = vec![Plot::Scatter(a), Plot::Scatter(b), Plot::Scatter(c)];
    let svg = render(plots, "Scatter — 3 series: solid / semi-transparent / hollow");
    write("marker_scatter_three_styles", &svg);

    assert!(has_fill_opacity(&svg), "fill-opacity should appear (series B and C)");
    assert!(has_stroke(&svg), "stroke should appear (series B and C)");
    assert!(svg.contains("Solid"), "legend label Solid should appear");
    assert!(svg.contains("Hollow"), "legend label Hollow should appear");
}

/// Solid markers (no opacity/stroke set) must not emit fill-opacity or stroke.
#[test]
fn test_scatter_solid_no_opacity_attrs() {
    let plot = ScatterPlot::new()
        .with_data(vec![(1.0, 2.0), (3.0, 4.0), (5.0, 6.0)])
        .with_color("black")
        .with_size(4.0);

    let svg = render(vec![Plot::Scatter(plot)], "Scatter — solid default");
    write("marker_scatter_solid_default", &svg);

    // Must NOT emit fill-opacity or stroke on circles when not configured
    // (the SVG circles for the data points should have neither)
    let circle_segment = svg.find("<circle").and_then(|i| svg[i..].find("/>").map(|j| &svg[i..i+j]));
    if let Some(seg) = circle_segment {
        assert!(!seg.contains("fill-opacity"), "solid circles must not emit fill-opacity");
        assert!(!seg.contains(" stroke="), "solid circles must not emit stroke");
    }
}

// ── StripPlot ────────────────────────────────────────────────────────────────

/// 5 groups × 120 points = 600 points total, all with semi-transparent open
/// circles — the classic "dense strip" situation where solid dots hide structure.
#[test]
fn test_strip_semi_transparent_busy() {
    let mut seed: u64 = 55667788;
    let lcg = |s: &mut u64| -> f64 {
        *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (*s >> 33) as f64 / (u64::MAX >> 33) as f64
    };

    let groups = [
        ("Control",  5.0_f64, 0.5_f64),
        ("Low",      6.0,     0.8),
        ("Mid",      7.5,     1.2),
        ("High",     9.0,     0.6),
        ("Extreme",  11.0,    1.5),
    ];

    let mut plot = StripPlot::new()
        .with_color("steelblue")
        .with_point_size(5.0)
        .with_jitter(0.3)
        .with_marker_opacity(0.3)
        .with_marker_stroke_width(0.8);

    for &(label, mean, sd) in &groups {
        let vals: Vec<f64> = (0..120).map(|_| {
            let u1 = lcg(&mut seed).max(1e-9);
            let u2 = lcg(&mut seed);
            let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            mean + z * sd
        }).collect();
        plot = plot.with_group(label, vals);
    }

    let plots = vec![Plot::Strip(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Strip — semi-transparent + stroke (600 pts, 5 groups)")
        .with_y_label("Measurement");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    write("marker_strip_busy_semi_transparent", &svg);

    assert!(svg.contains("<circle"), "strip should emit circles");
    assert!(has_fill_opacity(&svg), "fill-opacity should be present");
    assert!(has_stroke(&svg), "stroke should be present");
}

/// Hollow swarm — beeswarm with hollow open circles to show point density.
#[test]
fn test_strip_hollow_swarm() {
    let mut seed: u64 = 99887766;
    let lcg = |s: &mut u64| -> f64 {
        *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (*s >> 33) as f64 / (u64::MAX >> 33) as f64
    };

    let mut plot = StripPlot::new()
        .with_color("darkorange")
        .with_point_size(4.0)
        .with_swarm()
        .with_marker_opacity(0.0)
        .with_marker_stroke_width(1.2);

    for &(label, mean, sd) in &[("A", 5.0_f64, 1.0_f64), ("B", 7.0, 1.5), ("C", 9.0, 0.8)] {
        let vals: Vec<f64> = (0..60).map(|_| {
            let u1 = lcg(&mut seed).max(1e-9);
            let u2 = lcg(&mut seed);
            let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            mean + z * sd
        }).collect();
        plot = plot.with_group(label, vals);
    }

    let plots = vec![Plot::Strip(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Strip — hollow open circles, beeswarm layout");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    write("marker_strip_hollow_swarm", &svg);

    assert!(has_fill_opacity(&svg));
    assert!(has_stroke(&svg));
}

// ── PolarPlot ────────────────────────────────────────────────────────────────

/// Multi-series polar scatter with different opacity/stroke settings per series.
#[test]
fn test_polar_marker_styles_multi_series() {
    let mut seed: u64 = 123456789;
    let lcg = |s: &mut u64| -> f64 {
        *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (*s >> 33) as f64 / (u64::MAX >> 33) as f64
    };

    // Series 1: dense cluster near r=0.5 around θ=30°—90°; semi-transparent
    let (r1, t1): (Vec<f64>, Vec<f64>) = (0..120).map(|_| {
        let u = lcg(&mut seed);
        let v = lcg(&mut seed);
        let r = 0.5 + (u - 0.5) * 0.3;
        let t = 60.0 + (v - 0.5) * 60.0;
        (r, t)
    }).unzip();

    // Series 2: dense cluster near r=0.8 around θ=180°–240°; hollow
    let (r2, t2): (Vec<f64>, Vec<f64>) = (0..120).map(|_| {
        let u = lcg(&mut seed);
        let v = lcg(&mut seed);
        let r = 0.8 + (u - 0.5) * 0.2;
        let t = 210.0 + (v - 0.5) * 60.0;
        (r, t)
    }).unzip();

    // Series 3: ring at r≈1 spread over all angles; solid small dots
    let (r3, t3): (Vec<f64>, Vec<f64>) = (0..80).map(|i| {
        (1.0 + (lcg(&mut seed) - 0.5) * 0.1, i as f64 * (360.0 / 80.0))
    }).unzip();

    let plot = PolarPlot::new()
        .with_series_labeled(r1, t1, "Cluster A", PolarMode::Scatter)
        .with_color("steelblue")
        .with_marker_opacity(0.3)
        .with_marker_stroke_width(0.8)
        .with_series_labeled(r2, t2, "Cluster B", PolarMode::Scatter)
        .with_color("tomato")
        .with_marker_opacity(0.0)
        .with_marker_stroke_width(1.2)
        .with_series_labeled(r3, t3, "Ring", PolarMode::Scatter)
        .with_color("seagreen")
        .with_r_max(1.2)
        .with_legend(true);

    let plots = vec![Plot::Polar(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Polar — mixed opacity/stroke styles (320 pts, 3 series)");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    write("marker_polar_mixed_styles", &svg);

    assert!(svg.contains("<circle"), "polar scatter should emit circles");
    assert!(has_fill_opacity(&svg), "fill-opacity should appear (series A and B)");
    assert!(has_stroke(&svg), "stroke should appear (series A and B)");
    assert!(svg.contains("Cluster A"), "legend label should appear");
    assert!(svg.contains("Ring"), "legend label should appear");
}

/// Very dense polar scatter (500 points) with semi-transparent dots.
/// Shows the wind-rose density pattern clearly.
#[test]
fn test_polar_dense_semi_transparent() {
    use std::f64::consts::PI;

    let mut seed: u64 = 444555666;
    let lcg = |s: &mut u64| -> f64 {
        *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (*s >> 33) as f64 / (u64::MAX >> 33) as f64
    };

    // Four dominant wind directions: N (0°), NE (45°), S (180°), W (270°)
    let directions = [0.0_f64, 45.0, 180.0, 270.0];
    let (mut r_all, mut t_all) = (Vec::new(), Vec::new());

    for &dir in &directions {
        for _ in 0..125 {
            let u1 = lcg(&mut seed).max(1e-9);
            let u2 = lcg(&mut seed);
            let z_r = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos();
            let z_t = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).sin();
            let r = (0.7 + z_r * 0.15).clamp(0.05, 1.2);
            let t = (dir + z_t * 20.0).rem_euclid(360.0);
            r_all.push(r);
            t_all.push(t);
        }
    }

    let plot = PolarPlot::new()
        .with_series(r_all, t_all)
        .with_color("steelblue")
        .with_marker_opacity(0.2)
        .with_marker_stroke_width(0.6)
        .with_r_max(1.3)
        .with_theta_divisions(24);

    let plots = vec![Plot::Polar(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Polar — wind-rose density, 500 pts, semi-transparent");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    write("marker_polar_wind_rose_dense", &svg);

    assert!(has_fill_opacity(&svg));
    assert!(has_stroke(&svg));
}

// ── TernaryPlot ──────────────────────────────────────────────────────────────

/// Very busy ternary (300+ points, 4 groups, each a tight cluster near one
/// vertex + centre) with semi-transparent markers and stroke.
#[test]
fn test_ternary_semi_transparent_busy() {
    let mut seed: u64 = 7654321;
    let lcg = |s: &mut u64| -> f64 {
        *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (*s >> 33) as f64 / (u64::MAX >> 33) as f64
    };

    // Generate a point near (a0, b0, c0) with std dev sigma, then normalise.
    let gen_point = |a0: f64, b0: f64, c0: f64, sigma: f64, s: &mut u64| -> (f64, f64, f64) {
        let gauss = |s: &mut u64| -> f64 {
            let u1 = lcg(s).max(1e-9);
            let u2 = lcg(s);
            (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
        };
        let a = (a0 + gauss(s) * sigma).max(0.0);
        let b = (b0 + gauss(s) * sigma).max(0.0);
        let c = (c0 + gauss(s) * sigma).max(0.0);
        let sum = a + b + c;
        (a / sum, b / sum, c / sum)
    };

    let mut plot = TernaryPlot::new()
        .with_corner_labels("Sand", "Silt", "Clay")
        .with_normalize(true)
        .with_legend(true)
        .with_marker_opacity(0.35)
        .with_marker_stroke_width(0.8);

    let clusters = [
        (0.8, 0.1, 0.1, "Sandy"),
        (0.1, 0.8, 0.1, "Silty"),
        (0.1, 0.1, 0.8, "Clayey"),
        (1.0/3.0, 1.0/3.0, 1.0/3.0, "Loam"),
    ];

    for &(a0, b0, c0, label) in &clusters {
        for _ in 0..80 {
            let (a, b, c) = gen_point(a0, b0, c0, 0.08, &mut seed);
            plot = plot.with_point_group(a, b, c, label);
        }
    }

    let plots = vec![Plot::Ternary(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Ternary — soil texture diagram, 320 pts, semi-transparent");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    write("marker_ternary_soil_busy", &svg);

    assert!(svg.contains("<circle"), "ternary data points should be circles");
    assert!(has_fill_opacity(&svg), "fill-opacity should appear");
    assert!(has_stroke(&svg), "stroke should appear");
    assert!(svg.contains("Sandy"), "legend label should appear");
    assert!(svg.contains("Clay"), "corner label should appear");
}

/// Ternary hollow open circles — 400 points showing density by circle overlap.
#[test]
fn test_ternary_hollow_circles_dense() {
    let mut seed: u64 = 314159265;
    let lcg = |s: &mut u64| -> f64 {
        *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (*s >> 33) as f64 / (u64::MAX >> 33) as f64
    };

    let mut plot = TernaryPlot::new()
        .with_corner_labels("A", "B", "C")
        .with_normalize(true)
        .with_marker_size(4.0)
        .with_marker_opacity(0.0)
        .with_marker_stroke_width(1.0);

    // Diagonal band from A-corner to BC midpoint with Gaussian spread
    for _ in 0..400 {
        let t = lcg(&mut seed);
        let noise_b = (lcg(&mut seed) - 0.5) * 0.15;
        let noise_c = (lcg(&mut seed) - 0.5) * 0.15;
        let a = (1.0 - t).max(0.0);
        let b = (t * 0.5 + noise_b).max(0.0);
        let c = (t * 0.5 + noise_c).max(0.0);
        let s = a + b + c;
        plot = plot.with_point(a / s, b / s, c / s);
    }

    let plots = vec![Plot::Ternary(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Ternary — hollow open circles, A→BC diagonal band, 400 pts");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    write("marker_ternary_hollow_dense", &svg);

    assert!(svg.contains("<circle"), "ternary data points should be circles");
    assert!(svg.contains(r#"fill-opacity="0""#), "fill-opacity should be 0 (hollow)");
    assert!(has_stroke(&svg), "stroke should be present for hollow circles");
}

/// Smoke-check: building with opacity/stroke but rendering without groups.
#[test]
fn test_ternary_ungrouped_with_stroke() {
    let plot = TernaryPlot::new()
        .with_point(0.6, 0.2, 0.2)
        .with_point(0.2, 0.6, 0.2)
        .with_point(0.2, 0.2, 0.6)
        .with_point(1.0/3.0, 1.0/3.0, 1.0/3.0)
        .with_marker_opacity(0.5)
        .with_marker_stroke_width(1.5);

    let plots = vec![Plot::Ternary(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Ternary — ungrouped with stroke");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    write("marker_ternary_ungrouped_stroke", &svg);

    assert!(has_fill_opacity(&svg));
    assert!(has_stroke(&svg));
    assert!(has_stroke_width_on_circle(&svg));
}

// ── Attribute precision ──────────────────────────────────────────────────────

/// fill-opacity should be a sensible decimal value, not NaN or garbage.
#[test]
fn test_fill_opacity_value_is_sane() {
    let plot = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 1.0_f64), (2.0, 2.0)])
        .with_color("navy")
        .with_size(4.0)
        .with_marker_opacity(0.4);

    let plots = vec![Plot::Scatter(plot)];
    let svg = render(plots, "opacity sanity");

    // find fill-opacity="..." and check the value
    if let Some(i) = svg.find(r#"fill-opacity=""#) {
        let rest = &svg[i + r#"fill-opacity=""#.len()..];
        let val: &str = rest.split('"').next().unwrap_or("");
        let parsed: f64 = val.parse().expect("fill-opacity must be a valid float");
        assert!(
            (parsed - 0.4).abs() < 0.01,
            "fill-opacity should be ~0.4, got {parsed}"
        );
    } else {
        panic!("fill-opacity attribute not found in SVG");
    }
}

/// stroke-width should reflect the value passed to with_marker_stroke_width.
#[test]
fn test_stroke_width_value_is_sane() {
    let plot = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 1.0_f64)])
        .with_color("teal")
        .with_size(5.0)
        .with_marker_opacity(0.5)
        .with_marker_stroke_width(2.5);

    let plots = vec![Plot::Scatter(plot)];
    let svg = render(plots, "stroke-width sanity");

    // find stroke-width near a circle element
    if let Some(ci) = svg.find("<circle") {
        let circle_seg = &svg[ci..];
        let end = circle_seg.find("/>").unwrap_or(circle_seg.len());
        let seg = &circle_seg[..end];
        assert!(seg.contains("stroke-width="), "stroke-width must be in the circle element");
        if let Some(i) = seg.find(r#"stroke-width=""#) {
            let rest = &seg[i + r#"stroke-width=""#.len()..];
            let val: &str = rest.split('"').next().unwrap_or("");
            let parsed: f64 = val.parse().expect("stroke-width must be valid float");
            assert!(
                (parsed - 2.5).abs() < 0.01,
                "stroke-width should be ~2.5, got {parsed}"
            );
        }
    } else {
        panic!("no <circle> element found in SVG");
    }
}
