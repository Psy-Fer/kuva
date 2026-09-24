//! Legend reference documentation examples.
//!
//! Generates canonical SVG outputs used in docs/src/reference/legends.md.
//! Run with:
//!
//! ```bash
//! cargo run --example legends
//! ```
//!
//! SVGs are written to `docs/src/assets/legends/`.

use kuva::backend::svg::SvgBackend;
use kuva::plot::legend::{LegendEntry, LegendPosition, LegendShape};
use kuva::plot::scatter::ScatterPlot;
use kuva::plot::stacked_area::StackedAreaPlot;
use kuva::render::figure::Figure;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

const OUT: &str = "docs/src/assets/legends";

// ── Shared dataset ──────────────────────────────────────────────────────────
// Monthly variant counts for 4 types; used across most examples so every
// position/style screenshot shows the same underlying data.

fn months() -> Vec<f64> {
    (1..=12).map(|m| m as f64).collect()
}

fn variant_sa(pos: LegendPosition) -> (Vec<Plot>, Layout) {
    let sa = StackedAreaPlot::new()
        .with_x(months())
        .with_series([
            420.0, 445.0, 398.0, 510.0, 488.0, 501.0, 467.0, 523.0, 495.0, 540.0, 518.0, 555.0,
        ])
        .with_color("steelblue")
        .with_legend("SNVs")
        .with_series([
            95.0, 102.0, 88.0, 115.0, 108.0, 112.0, 98.0, 125.0, 118.0, 130.0, 122.0, 140.0,
        ])
        .with_color("orange")
        .with_legend("Indels")
        .with_series([
            22.0, 25.0, 20.0, 28.0, 26.0, 27.0, 24.0, 31.0, 28.0, 33.0, 30.0, 35.0,
        ])
        .with_color("mediumseagreen")
        .with_legend("SVs")
        .with_series([
            15.0, 17.0, 14.0, 19.0, 18.0, 18.0, 16.0, 21.0, 19.0, 23.0, 21.0, 24.0,
        ])
        .with_color("tomato")
        .with_legend("CNVs");

    let plots = vec![Plot::StackedArea(sa)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Monthly Variant Counts")
        .with_x_label("Month")
        .with_y_label("Variant count")
        .with_legend_position(pos);
    (plots, layout)
}

fn write(name: &str, plots: Vec<Plot>, layout: Layout) {
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
    println!("  wrote {OUT}/{name}.svg");
}

// ── main ────────────────────────────────────────────────────────────────────

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/legends");

    basic_auto();
    positions();
    no_box();
    legend_title();
    legend_groups();
    custom_pos();
    data_coords();
    manual_entries();
    size_override();
    figure_shared();

    println!("Legend SVGs written to {OUT}/");
}

// ── 1. Basic auto-collected legend ──────────────────────────────────────────

fn basic_auto() {
    // Scatter with three colour groups — legend is collected automatically.
    let groups = [
        (
            "Cluster A",
            "steelblue",
            vec![(1.1, 2.3), (1.9, 3.1), (2.4, 2.7), (3.0, 3.8), (3.6, 3.2)],
        ),
        (
            "Cluster B",
            "orange",
            vec![(4.0, 1.2), (4.8, 1.8), (5.3, 1.4), (6.0, 2.0), (6.5, 1.6)],
        ),
        (
            "Cluster C",
            "mediumseagreen",
            vec![(2.0, 5.5), (2.8, 6.1), (3.5, 5.8), (4.3, 6.5), (5.0, 6.0)],
        ),
    ];

    let mut plots: Vec<Plot> = Vec::new();
    for (label, color, pts) in &groups {
        plots.push(Plot::Scatter(
            ScatterPlot::new()
                .with_data(pts.iter().copied())
                .with_color(*color)
                .with_legend(*label)
                .with_size(6.0),
        ));
    }

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Auto-Collected Legend")
        .with_x_label("X")
        .with_y_label("Y");

    write("basic_auto", plots, layout);
}

// ── 2. Position variants ─────────────────────────────────────────────────────

fn positions() {
    // OutsideRightTop — default; no .with_legend_position() call needed
    let (plots, layout) = variant_sa(LegendPosition::OutsideRightTop);
    write("pos_outside_right_top", plots, layout);

    // InsideTopRight
    let (plots, layout) = variant_sa(LegendPosition::InsideTopRight);
    write("pos_inside_top_right", plots, layout);

    // InsideBottomLeft
    let (plots, layout) = variant_sa(LegendPosition::InsideBottomLeft);
    write("pos_inside_bottom_left", plots, layout);

    // OutsideLeftTop
    let (plots, layout) = variant_sa(LegendPosition::OutsideLeftTop);
    write("pos_outside_left_top", plots, layout);

    // OutsideBottomCenter
    let (plots, layout) = variant_sa(LegendPosition::OutsideBottomCenter);
    write("pos_outside_bottom_center", plots, layout);
}

// ── 3. Suppress the legend box ───────────────────────────────────────────────

fn no_box() {
    let sa = StackedAreaPlot::new()
        .with_x(months())
        .with_series([
            420.0, 445.0, 398.0, 510.0, 488.0, 501.0, 467.0, 523.0, 495.0, 540.0, 518.0, 555.0,
        ])
        .with_color("steelblue")
        .with_legend("SNVs")
        .with_series([
            95.0, 102.0, 88.0, 115.0, 108.0, 112.0, 98.0, 125.0, 118.0, 130.0, 122.0, 140.0,
        ])
        .with_color("orange")
        .with_legend("Indels")
        .with_series([
            22.0, 25.0, 20.0, 28.0, 26.0, 27.0, 24.0, 31.0, 28.0, 33.0, 30.0, 35.0,
        ])
        .with_color("mediumseagreen")
        .with_legend("SVs")
        .with_series([
            15.0, 17.0, 14.0, 19.0, 18.0, 18.0, 16.0, 21.0, 19.0, 23.0, 21.0, 24.0,
        ])
        .with_color("tomato")
        .with_legend("CNVs");

    let plots = vec![Plot::StackedArea(sa)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Legend Without Box")
        .with_x_label("Month")
        .with_y_label("Variant count")
        .with_legend_position(LegendPosition::InsideTopRight)
        .with_legend_box(false);

    write("no_box", plots, layout);
}

// ── 4. Legend title ───────────────────────────────────────────────────────────

fn legend_title() {
    let (plots, layout) = variant_sa(LegendPosition::OutsideRightTop);
    let layout = layout.with_legend_title("Variant type");
    write("legend_title", plots, layout);
}

// ── 5. Grouped legend ─────────────────────────────────────────────────────────

fn legend_groups() {
    let groups = [
        (
            "Control-A",
            "steelblue",
            vec![(1.0, 2.1), (2.0, 3.4), (3.0, 2.9), (4.0, 4.2)],
        ),
        (
            "Control-B",
            "#4e9fd4",
            vec![(1.0, 1.8), (2.0, 2.9), (3.0, 2.5), (4.0, 3.6)],
        ),
        (
            "Treatment-A",
            "tomato",
            vec![(1.0, 2.5), (2.0, 4.1), (3.0, 5.3), (4.0, 6.8)],
        ),
        (
            "Treatment-B",
            "#e06060",
            vec![(1.0, 2.2), (2.0, 3.8), (3.0, 5.0), (4.0, 6.2)],
        ),
    ];

    let mut plots: Vec<Plot> = Vec::new();
    for (_, color, pts) in &groups {
        plots.push(Plot::Scatter(
            ScatterPlot::new()
                .with_data(pts.iter().copied())
                .with_color(*color)
                .with_size(6.0),
        ));
    }

    let ctrl_entries = vec![
        LegendEntry {
            label: "Control-A".into(),
            color: "steelblue".into(),
            shape: LegendShape::Circle,
            dasharray: None,
        },
        LegendEntry {
            label: "Control-B".into(),
            color: "#4e9fd4".into(),
            shape: LegendShape::Circle,
            dasharray: None,
        },
    ];
    let trt_entries = vec![
        LegendEntry {
            label: "Treatment-A".into(),
            color: "tomato".into(),
            shape: LegendShape::Circle,
            dasharray: None,
        },
        LegendEntry {
            label: "Treatment-B".into(),
            color: "#e06060".into(),
            shape: LegendShape::Circle,
            dasharray: None,
        },
    ];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Grouped Legend")
        .with_x_label("Time (h)")
        .with_y_label("Expression")
        .with_legend_group("Controls", ctrl_entries)
        .with_legend_group("Treatments", trt_entries);

    write("legend_groups", plots, layout);
}

// ── 6. Custom pixel position ──────────────────────────────────────────────────

fn custom_pos() {
    let (plots, layout) = variant_sa(LegendPosition::OutsideRightTop);
    // Place the legend at an explicit pixel coordinate inside the plot area.
    // with_legend_at sets Custom(x, y) — no extra margin is reserved.
    let layout = layout.with_legend_at(30.0, 30.0);
    write("custom_pos", plots, layout);
}

// ── 7. Data-space position ────────────────────────────────────────────────────

fn data_coords() {
    let (plots, layout) = variant_sa(LegendPosition::OutsideRightTop);
    // Place the legend at month=7, count=450 — inside the data area.
    let layout = layout.with_legend_at_data(7.0, 450.0);
    write("data_coords", plots, layout);
}

/// `(label, colour, marker shape, points)` for a manually-built legend entry.
type ShapedSeries = (
    &'static str,
    &'static str,
    LegendShape,
    &'static [(f64, f64)],
);

/// `(label, colour, points)` for a series that takes the default marker.
type PlainSeries = (&'static str, &'static str, &'static [(f64, f64)]);

// ── 8. Manual legend entries ──────────────────────────────────────────────────

fn manual_entries() {
    // Three scatter series with different marker shapes.
    let series: &[ShapedSeries] = &[
        (
            "Healthy",
            "steelblue",
            LegendShape::Circle,
            &[(1.2, 4.1), (2.0, 3.8), (2.8, 4.5), (3.7, 4.0), (4.5, 4.3)],
        ),
        (
            "At risk",
            "orange",
            LegendShape::Rect,
            &[(1.5, 2.5), (2.3, 2.1), (3.1, 2.8), (4.0, 2.4), (4.8, 2.7)],
        ),
        (
            "Diseased",
            "crimson",
            LegendShape::Line,
            &[(1.1, 0.9), (1.9, 1.3), (2.7, 0.7), (3.5, 1.1), (4.3, 0.8)],
        ),
    ];

    let mut plots: Vec<Plot> = Vec::new();
    for (_, color, _, pts) in series {
        plots.push(Plot::Scatter(
            ScatterPlot::new()
                .with_data(pts.iter().copied())
                .with_color(*color)
                .with_size(6.0),
        ));
    }

    let entries: Vec<LegendEntry> = series
        .iter()
        .map(|(label, color, shape, _)| LegendEntry {
            label: (*label).into(),
            color: (*color).into(),
            shape: *shape,
            dasharray: None,
        })
        .collect();

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Manual Legend Entries")
        .with_x_label("Biomarker A")
        .with_y_label("Biomarker B")
        .with_legend_entries(entries);

    write("manual_entries", plots, layout);
}

// ── 9. Legend width override ──────────────────────────────────────────────────

fn size_override() {
    // Long labels that would overflow the default auto-sized box.
    let series: &[PlainSeries] = &[
        (
            "Homo sapiens (reference)",
            "steelblue",
            &[(0.0, 1.0), (1.0, 1.8), (2.0, 2.4), (3.0, 2.9)],
        ),
        (
            "Mus musculus (knockout)",
            "orange",
            &[(0.0, 0.9), (1.0, 1.5), (2.0, 1.8), (3.0, 2.0)],
        ),
        (
            "Rattus norvegicus (ctrl)",
            "crimson",
            &[(0.0, 0.8), (1.0, 1.2), (2.0, 1.4), (3.0, 1.5)],
        ),
    ];

    let mut plots: Vec<Plot> = Vec::new();
    for (label, color, pts) in series {
        plots.push(Plot::Scatter(
            ScatterPlot::new()
                .with_data(pts.iter().copied())
                .with_color(*color)
                .with_legend(*label)
                .with_size(6.0),
        ));
    }

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Long Labels — Width Override")
        .with_x_label("Week")
        .with_y_label("Fold change")
        .with_legend_width(230.0);

    write("size_override", plots, layout);
}

// ── 10. Figure shared legend ──────────────────────────────────────────────────

fn figure_shared() {
    let colors = [
        ("steelblue", "SNVs"),
        ("orange", "Indels"),
        ("mediumseagreen", "SVs"),
        ("tomato", "CNVs"),
    ];

    let counts: [[f64; 6]; 4] = [
        [420.0, 467.0, 510.0, 523.0, 540.0, 555.0],
        [95.0, 98.0, 115.0, 125.0, 130.0, 140.0],
        [22.0, 24.0, 28.0, 31.0, 33.0, 35.0],
        [15.0, 16.0, 19.0, 21.0, 23.0, 24.0],
    ];

    // Panel A — H1 cohort (first 6 months)
    let mut panel_a: Vec<Plot> = Vec::new();
    for (i, (color, label)) in colors.iter().enumerate() {
        let pts: Vec<(f64, f64)> = counts[i]
            .iter()
            .enumerate()
            .map(|(m, &v)| (m as f64 + 1.0, v))
            .collect();
        panel_a.push(Plot::Scatter(
            ScatterPlot::new()
                .with_data(pts)
                .with_color(*color)
                .with_legend(*label)
                .with_size(6.0),
        ));
    }

    // Panel B — H2 cohort (same types, slightly different values)
    let counts_b: [[f64; 6]; 4] = [
        [390.0, 440.0, 480.0, 505.0, 520.0, 535.0],
        [88.0, 95.0, 105.0, 118.0, 124.0, 132.0],
        [19.0, 22.0, 25.0, 28.0, 30.0, 33.0],
        [12.0, 14.0, 17.0, 19.0, 21.0, 23.0],
    ];
    let mut panel_b: Vec<Plot> = Vec::new();
    for (i, (color, label)) in colors.iter().enumerate() {
        let pts: Vec<(f64, f64)> = counts_b[i]
            .iter()
            .enumerate()
            .map(|(m, &v)| (m as f64 + 1.0, v))
            .collect();
        panel_b.push(Plot::Scatter(
            ScatterPlot::new()
                .with_data(pts)
                .with_color(*color)
                .with_legend(*label)
                .with_size(6.0),
        ));
    }

    let layout_a = Layout::auto_from_plots(&panel_a)
        .with_title("Cohort H1")
        .with_x_label("Month")
        .with_y_label("Count");
    let layout_b = Layout::auto_from_plots(&panel_b)
        .with_title("Cohort H2")
        .with_x_label("Month")
        .with_y_label("Count");

    let scene = Figure::new(1, 2)
        .with_plots(vec![panel_a, panel_b])
        .with_layouts(vec![layout_a, layout_b])
        .with_shared_legend()
        .render();

    let svg = SvgBackend.render_scene(&scene);
    std::fs::write(format!("{OUT}/figure_shared.svg"), svg).unwrap();
    println!("  wrote {OUT}/figure_shared.svg");
}
