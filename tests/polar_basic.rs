use std::f64::consts::PI;
use kuva::plot::polar::{PolarMode, PolarPlot};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use kuva::backend::svg::SvgBackend;

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
