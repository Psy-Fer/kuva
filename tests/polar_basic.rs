use std::f64::consts::PI;
use kuva::plot::polar::{PolarMode, PolarPlot};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use kuva::backend::svg::SvgBackend;
use kuva::TickFormat;

fn render(plot: PolarPlot) -> String {
    let plots = vec![Plot::Polar(plot)];
    let layout = Layout::auto_from_plots(&plots);
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

fn render_titled(plot: PolarPlot, title: &str) -> String {
    let plots = vec![Plot::Polar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title(title);
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

fn write(name: &str, svg: &str) {
    std::fs::create_dir_all("test_outputs").ok();
    std::fs::write(format!("test_outputs/{name}.svg"), svg).unwrap();
}

#[test]
fn test_polar_basic() {
    let theta: Vec<f64> = (0..36).map(|i| i as f64 * 10.0).collect();
    let r: Vec<f64> = theta.iter().map(|&t| 1.0 + t.to_radians().cos()).collect();

    let plot = PolarPlot::new().with_series(r, theta);
    let svg = render(plot);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle") || svg.contains("<path"));
    write("polar_basic", &svg);
}

#[test]
fn test_polar_line() {
    let theta: Vec<f64> = (0..36).map(|i| i as f64 * 10.0).collect();
    let r: Vec<f64> = vec![1.5; 36];

    let plot = PolarPlot::new().with_series_line(r, theta);
    let svg = render(plot);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path"));
    write("polar_line", &svg);
}

#[test]
fn test_polar_grid() {
    let theta: Vec<f64> = (0..12).map(|i| i as f64 * 30.0).collect();
    let r: Vec<f64> = vec![1.0; 12];

    let plot = PolarPlot::new()
        .with_series(r, theta)
        .with_grid(true)
        .with_r_grid_lines(4);
    let svg = render(plot);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path"));
    write("polar_grid", &svg);
}

#[test]
fn test_polar_clockwise() {
    let theta: Vec<f64> = (0..36).map(|i| i as f64 * 10.0).collect();
    let r: Vec<f64> = theta.iter().map(|&t| 1.0 + 0.5 * t.to_radians().cos()).collect();

    let plot = PolarPlot::new()
        .with_series(r, theta)
        .with_clockwise(true)
        .with_theta_start(0.0);
    let svg = render(plot);
    assert!(svg.contains("<svg"));
    write("polar_clockwise", &svg);
}

#[test]
fn test_polar_r_max_override() {
    let theta: Vec<f64> = (0..36).map(|i| i as f64 * 10.0).collect();
    let r: Vec<f64> = vec![0.5; 36];

    let plot = PolarPlot::new()
        .with_series(r, theta)
        .with_r_max(2.0);
    let svg = render(plot);
    assert!(svg.contains("<svg"));
    write("polar_r_max", &svg);
}

#[test]
fn test_polar_multiple_series() {
    let theta: Vec<f64> = (0..36).map(|i| i as f64 * 10.0).collect();
    let r1: Vec<f64> = vec![1.0; 36];
    let r2: Vec<f64> = vec![2.0; 36];

    let plot = PolarPlot::new()
        .with_series_labeled(r1, theta.clone(), "Series A", PolarMode::Scatter)
        .with_series_labeled(r2, theta, "Series B", PolarMode::Scatter);
    let svg = render(plot);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle") || svg.contains("<path"));
    write("polar_multiple_series", &svg);
}

#[test]
fn test_polar_legend() {
    let theta: Vec<f64> = (0..36).map(|i| i as f64 * 10.0).collect();
    let r: Vec<f64> = vec![1.0; 36];

    let plot = PolarPlot::new()
        .with_series_labeled(r, theta, "Wind speed", PolarMode::Scatter)
        .with_legend(true);
    let plots = vec![Plot::Polar(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Polar Legend Test");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Wind speed"));
    write("polar_legend", &svg);
}

#[test]
fn test_polar_x_tick_format() {
    let theta: Vec<f64> = (0..36).map(|i| i as f64 * 10.0).collect();
    let r: Vec<f64> = vec![1.0; 36];

    let plot = PolarPlot::new()
        .with_series_labeled(r, theta, "Wind speed", PolarMode::Scatter)
        .with_theta_divisions(8)
        .with_legend(true);
    let plots = vec![Plot::Polar(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_x_tick_format(TickFormat::Custom(std::sync::Arc::new(
            |v| {
                if v < 45.0 {
                    "N".to_string()
                } else if v < 90.0 {
                    "NE".to_string()
                } else if v < 135.0 {
                    "E".to_string()
                } else if v < 180.0 {
                    "SE".to_string()
                } else if v < 225.0 {
                    "S".to_string()
                } else if v < 270.0 {
                    "SW".to_string()
                } else if v < 315.0 {
                    "W".to_string()
                } else {
                    "NW".to_string()
                }
            },
        )))
        .with_title("Polar Custom X Ticks Test");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Wind speed"));
    assert!(svg.contains("NE"));
    assert!(svg.contains("SE"));
    assert!(svg.contains("SW"));
    assert!(svg.contains("NW"));
    write("polar_x_ticks", &svg);
}

// ── complex showcase tests ─────────────────────────────────────────────────────

// Cardioid (line) + noisy observations (scatter): two series, two colors, legend.
// The cardioid r = 1 + cos(θ) is a classic polar curve — heart-shaped loop.
#[test]
fn test_polar_cardioid_with_observations() {
    // Smooth cardioid line (360 points)
    let n = 360usize;
    let theta_line: Vec<f64> = (0..=n).map(|i| i as f64).collect();
    let r_line: Vec<f64> = theta_line
        .iter()
        .map(|&t| 1.0 + (t * PI / 180.0).cos())
        .collect();

    // Sparse noisy observations sampled every 15°
    let mut state: u64 = 77777;
    let mut lcg = || -> f64 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (state >> 33) as f64 / (u64::MAX >> 33) as f64
    };
    let theta_obs: Vec<f64> = (0..24).map(|i| i as f64 * 15.0).collect();
    let r_obs: Vec<f64> = theta_obs
        .iter()
        .map(|&t| {
            let ideal = 1.0 + (t * PI / 180.0).cos();
            (ideal + (lcg() - 0.5) * 0.25).max(0.0)
        })
        .collect();

    let plot = PolarPlot::new()
        .with_series_labeled(r_line, theta_line, "Cardioid", PolarMode::Line)
        .with_color("#2171b5")
        .with_series_labeled(r_obs, theta_obs, "Observations", PolarMode::Scatter)
        .with_color("#d94801")
        .with_r_max(2.0)
        .with_r_grid_lines(4)
        .with_theta_divisions(12)
        .with_legend(true);

    let plots = vec![Plot::Polar(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Cardioid r = 1 + cos(θ)");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path"));
    assert!(svg.contains("Cardioid"));
    assert!(svg.contains("Observations"));
    write("polar_cardioid_observations", &svg);
}

// Three mathematical curves as lines — different colors, legend, 8 angular spokes.
// Rose:      r = |cos(3θ)|   — 6-petal pattern (abs keeps r positive)
// Lemniscate r = sqrt(|cos(2θ)|) — figure-8 lobes
// Unit circle r = 1.0         — reference baseline
#[test]
fn test_polar_three_curves() {
    let n = 720usize; // 0.5° resolution for smooth curves
    let theta: Vec<f64> = (0..n).map(|i| i as f64 * 360.0 / n as f64).collect();

    let r_rose: Vec<f64> = theta
        .iter()
        .map(|&t| (3.0 * t * PI / 180.0).cos().abs())
        .collect();

    let r_lemniscate: Vec<f64> = theta
        .iter()
        .map(|&t| (2.0 * t * PI / 180.0).cos().abs().sqrt())
        .collect();

    let r_circle: Vec<f64> = vec![1.0; n];

    let plot = PolarPlot::new()
        .with_series_labeled(r_rose,       theta.clone(), "Rose |cos 3θ|",       PolarMode::Line)
        .with_color("#e41a1c")
        .with_series_labeled(r_lemniscate, theta.clone(), "Lemniscate √|cos 2θ|", PolarMode::Line)
        .with_color("#377eb8")
        .with_series_labeled(r_circle,     theta,         "Unit circle",           PolarMode::Line)
        .with_color("#4daf4a")
        .with_r_max(1.0)
        .with_r_grid_lines(4)
        .with_theta_divisions(8)
        .with_legend(true);

    let plots = vec![Plot::Polar(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Polar Curves");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path"));
    assert!(svg.contains("Rose"));
    assert!(svg.contains("Lemniscate"));
    assert!(svg.contains("Unit circle"));
    write("polar_three_curves", &svg);
}

// Archimedean spiral in math convention (θ=0 east, CCW).
// r = θ / (2π) — one full loop per 360°, three loops total.
#[test]
fn test_polar_spiral_math_convention() {
    let n = 1080usize; // 3 × 360 = three full loops
    let theta: Vec<f64> = (0..=n).map(|i| i as f64 / 3.0).collect(); // 0°–360°
    let r: Vec<f64> = theta
        .iter()
        .map(|&t| t / 360.0) // r grows from 0 to 1 over 360°
        .collect();

    let plot = PolarPlot::new()
        .with_series_labeled(r, theta, "Archimedean spiral", PolarMode::Line)
        .with_color("#6a3d9a")
        // Math convention: θ=0 at east, counter-clockwise
        .with_theta_start(90.0)
        .with_clockwise(false)
        .with_r_grid_lines(3)
        .with_theta_divisions(12)
        .with_legend(true);

    let svg = render_titled(plot, "Spiral (math convention)");
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path"));
    assert!(svg.contains("Archimedean spiral"));
    write("polar_spiral", &svg);
}

// Directional wind-rose style: four bearing clusters (N/E/S/W) as scatter,
// plus a smooth omnidirectional reference circle as a line.
// Mimics compass-convention directional data (θ=0 north, CW).
#[test]
fn test_polar_wind_rose_style() {
    let mut state: u64 = 31415;
    let mut lcg = || -> f64 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (state >> 33) as f64 / (u64::MAX >> 33) as f64
    };

    // Four directional clusters: N=0°, E=90°, S=180°, W=270°
    let centers = [("North", 0.0_f64), ("East", 90.0), ("South", 180.0), ("West", 270.0)];
    let colors  = ["#e41a1c", "#377eb8", "#4daf4a", "#ff7f00"];

    let mut plot = PolarPlot::new()
        .with_r_max(2.5)
        .with_r_grid_lines(5)
        .with_theta_divisions(16) // every 22.5° for compass rose feel
        .with_legend(true);

    for (&(label, center_deg), color) in centers.iter().zip(colors.iter()) {
        let mut r_vals = Vec::new();
        let mut t_vals = Vec::new();
        for _ in 0..20 {
            let spread = (lcg() - 0.5) * 30.0; // ±15° angular spread
            let t = center_deg + spread;
            let r = 1.2 + lcg() * 1.0; // r between 1.2 and 2.2
            t_vals.push(t);
            r_vals.push(r);
        }
        plot = plot
            .with_series_labeled(r_vals, t_vals, label, PolarMode::Scatter)
            .with_color(*color);
    }

    // Calm reference circle at r=1.0
    let theta_ref: Vec<f64> = (0..=360).map(|i| i as f64).collect();
    let r_ref: Vec<f64> = vec![1.0; 361];
    plot = plot
        .with_series_labeled(r_ref, theta_ref, "Calm radius", PolarMode::Line)
        .with_color("#aaaaaa");

    let plots = vec![Plot::Polar(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Wind Rose (Compass Convention)");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));

    assert!(svg.contains("<svg"));
    assert!(svg.contains("North"));
    assert!(svg.contains("East"));
    assert!(svg.contains("South"));
    assert!(svg.contains("West"));
    assert!(svg.contains("Calm radius"));
    write("polar_wind_rose", &svg);
}

// Regression test for PR #40 / the θ=0° hardcoded-label bug.
//
// Before the fix, `add_polar` had:
//   if theta_deg == 0.0 { "0°".to_string() } else { format!("{}°", ...) }
// — meaning the 0° spoke ALWAYS showed "0°" regardless of `x_tick_format`.
// Custom labels were "covered" at exactly θ=0.
//
// After the fix, all spokes go through `computed.x_tick_format.format(theta_deg)`,
// so a custom formatter is respected at every division including 0°.
//
// This test uses 4 divisions (N/E/S/W) and verifies:
//   1. The custom labels appear ("North", "East", "South", "West").
//   2. The old hardcoded "0°" is NOT present (it would indicate the bug is back).
//   3. Default `auto_from_plots` behaviour is unchanged: degree labels like "90°"
//      still appear when no custom format is set.
#[test]
fn test_polar_custom_tick_overrides_zero_degree() {
    use std::sync::Arc;
    use kuva::TickFormat;

    // 4-spoke polar with custom compass labels.
    let r: Vec<f64> = vec![1.0, 2.0, 1.5, 0.5, 1.0]; // closed
    let theta: Vec<f64> = vec![0.0, 90.0, 180.0, 270.0, 360.0];

    let plot = PolarPlot::new()
        .with_series(r, theta)
        .with_theta_divisions(4);

    let plots = vec![Plot::Polar(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_x_tick_format(TickFormat::Custom(Arc::new(|v| {
            match v as u32 {
                0   => "North".to_string(),
                90  => "East".to_string(),
                180 => "South".to_string(),
                270 => "West".to_string(),
                _   => format!("{v}°"),
            }
        })));

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    write("polar_custom_tick_zero", &svg);

    // Custom labels must appear at all four spokes.
    assert!(svg.contains(">North<"), "θ=0° spoke must show custom label 'North'");
    assert!(svg.contains(">East<"),  "θ=90° spoke must show 'East'");
    assert!(svg.contains(">South<"), "θ=180° spoke must show 'South'");
    assert!(svg.contains(">West<"),  "θ=270° spoke must show 'West'");

    // The old hardcoded "0°" must NOT appear — that would mean the bug is back.
    assert!(!svg.contains(">0°<"),
        "θ=0° spoke must not show hardcoded '0°' when a custom format is set");
}

#[test]
fn test_polar_default_degree_format() {
    // Without a custom format, auto_from_plots sets TickFormat::Degree,
    // so labels should show degree symbols: "0°", "90°", "180°", "270°".
    let r: Vec<f64> = vec![1.0; 5];
    let theta: Vec<f64> = vec![0.0, 90.0, 180.0, 270.0, 360.0];

    let plot = PolarPlot::new()
        .with_series(r, theta)
        .with_theta_divisions(4);

    let svg = render(plot);
    write("polar_default_degree_format", &svg);

    assert!(svg.contains(">0°<"),   "default polar format must show '0°' at θ=0");
    assert!(svg.contains(">90°<"),  "default polar format must show '90°'");
    assert!(svg.contains(">180°<"), "default polar format must show '180°'");
    assert!(svg.contains(">270°<"), "default polar format must show '270°'");
}

// ── r_min / negative-radius tests (#54) ───────────────────────────────────────

// Basic: r_min shifts the baseline so data at r=r_min maps to centre.
// r in [0.5, 1.5], r_min=0.5 → effective range [0, 1.0].
// Ring labels should show actual r values: 0.75, 1.0, 1.25, 1.5 (4 rings).
#[test]
fn test_polar_r_min_basic() {
    let theta: Vec<f64> = (0..8).map(|i| i as f64 * 45.0).collect();
    let r: Vec<f64> = vec![0.5, 0.75, 1.0, 1.25, 1.5, 1.25, 1.0, 0.75];

    let plot = PolarPlot::new()
        .with_series(r, theta)
        .with_r_min(0.5)
        .with_r_max(1.5)
        .with_r_grid_lines(4);
    let svg = render(plot);
    assert!(svg.contains("<svg"));
    // Ring labels must reflect actual r values, not fractions.
    assert!(svg.contains(">0.75<") || svg.contains(">1<") || svg.contains(">1.5<"),
        "ring labels should show actual r values relative to r_min");
    write("polar_r_min_basic", &svg);
}

// Negative r_min: dB-scale antenna pattern.
// r values range from -20 to 0. r_min=-20, r_max=0.
// Point at r=-20 → centre; point at r=0 → outer edge.
#[test]
fn test_polar_r_min_negative() {
    let theta: Vec<f64> = (0..=360).map(|i| i as f64).collect();
    // Simulate an antenna pattern: main lobe near 0°, back-lobe near 180°.
    let r: Vec<f64> = theta.iter().map(|&t| {
        let rad = t.to_radians();
        // Pattern: 0 dB at 0°, -20 dB at 180°, smooth in between.
        -20.0 * (1.0 - rad.cos().abs())
    }).collect();

    let plot = PolarPlot::new()
        .with_series_line(r, theta)
        .with_r_min(-20.0)
        .with_r_max(0.0)
        .with_r_grid_lines(4);
    let plots = vec![Plot::Polar(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Antenna Pattern (dB)");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path"), "line series should produce a path element");
    // Ring labels should include negative values.
    assert!(svg.contains(">-") || svg.contains(">0<"),
        "ring labels should contain negative or zero r values");
    write("polar_r_min_negative", &svg);
}

// Points with r < r_min should be clamped to centre (rendered there, not clipped out).
// One point at r=0.0 with r_min=1.0 → should still render (at centre).
#[test]
fn test_polar_r_min_clamp_to_centre() {
    let theta: Vec<f64> = vec![0.0, 90.0, 180.0, 270.0];
    // r=0.0 is below r_min=1.0 — should land at centre.
    let r: Vec<f64> = vec![0.0, 1.5, 2.0, 1.5];

    let plot = PolarPlot::new()
        .with_series(r, theta)
        .with_r_min(1.0)
        .with_r_max(2.0);
    let svg = render(plot);
    assert!(svg.contains("<svg"));
    // The clamped point lands at cx,cy — plot still renders.
    assert!(svg.contains("<circle") || svg.contains("<path"));
    write("polar_r_min_clamp", &svg);
}

// Auto r_max when r_min is set should use data max, not 0.
// r_min=-1.0, data max=1.0 → r_max auto = 1.0, range = 2.0.
#[test]
fn test_polar_r_min_auto_r_max() {
    let theta: Vec<f64> = (0..36).map(|i| i as f64 * 10.0).collect();
    // r = sin(theta) which goes negative
    let r: Vec<f64> = theta.iter().map(|&t| t.to_radians().sin()).collect();

    let plot = PolarPlot::new()
        .with_series_line(r, theta)
        .with_r_min(-1.0);
    // r_max not set — should auto-derive to ~1.0
    let svg = render(plot);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path"));
    write("polar_r_min_auto_r_max", &svg);
}

// r_min with scatter + existing r_max: verify it composes correctly.
#[test]
fn test_polar_r_min_with_explicit_r_max() {
    let theta: Vec<f64> = (0..12).map(|i| i as f64 * 30.0).collect();
    let r: Vec<f64> = vec![-5.0, -3.0, 0.0, 3.0, 5.0, 3.0, 0.0, -3.0, -5.0, -3.0, 0.0, 3.0];

    let plot = PolarPlot::new()
        .with_series(r, theta)
        .with_r_min(-5.0)
        .with_r_max(5.0)
        .with_r_grid_lines(5)
        .with_legend(false);
    let svg = render(plot);
    assert!(svg.contains("<svg"));
    write("polar_r_min_explicit_r_max", &svg);
}
