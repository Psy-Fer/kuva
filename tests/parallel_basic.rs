mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::parallel::ParallelPlot;
use kuva::render::{layout::Layout, plots::Plot, render::render_multiple};
use std::fs;

fn write_parallel(name: &str, plot: ParallelPlot, title: &str) -> String {
    let plots = vec![Plot::Parallel(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title(title);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    fs::create_dir_all("test_outputs").unwrap();
    common::write_test_output(format!("test_outputs/parallel_{name}.svg"), &svg).unwrap();
    svg
}

// Iris-style data (4 dimensions, 3 groups)
fn iris_data() -> Vec<(&'static str, Vec<Vec<f64>>)> {
    vec![
        (
            "setosa",
            vec![
                vec![5.1, 3.5, 1.4, 0.2],
                vec![4.9, 3.0, 1.4, 0.2],
                vec![4.7, 3.2, 1.3, 0.2],
                vec![4.6, 3.1, 1.5, 0.2],
                vec![5.0, 3.6, 1.4, 0.2],
            ],
        ),
        (
            "versicolor",
            vec![
                vec![7.0, 3.2, 4.7, 1.4],
                vec![6.4, 3.2, 4.5, 1.5],
                vec![6.9, 3.1, 4.9, 1.5],
                vec![5.5, 2.3, 4.0, 1.3],
                vec![6.5, 2.8, 4.6, 1.5],
            ],
        ),
        (
            "virginica",
            vec![
                vec![6.3, 3.3, 6.0, 2.5],
                vec![5.8, 2.7, 5.1, 1.9],
                vec![7.1, 3.0, 5.9, 2.1],
                vec![6.3, 2.9, 5.6, 1.8],
                vec![6.5, 3.0, 5.8, 2.2],
            ],
        ),
    ]
}

#[test]
fn test_parallel_basic() {
    let mut plot =
        ParallelPlot::new().with_axis_names(["Sepal.L", "Sepal.W", "Petal.L", "Petal.W"]);
    for (group, rows) in iris_data() {
        for row in rows {
            plot = plot.with_row_group(group, row);
        }
    }
    let svg = write_parallel("basic", plot, "Iris — Parallel Coordinates");
    assert!(svg.contains("<path"), "Expected polyline paths in SVG");
    assert!(svg.contains("Sepal.L"), "Expected axis label in SVG");
}

#[test]
fn test_parallel_no_groups() {
    let plot = ParallelPlot::new()
        .with_axis_names(["A", "B", "C"])
        .with_rows(vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0, 6.0],
            vec![7.0, 8.0, 9.0],
        ])
        .with_color("steelblue");
    let svg = write_parallel("no_groups", plot, "Ungrouped Parallel Plot");
    assert!(svg.contains("<path"));
}

#[test]
fn test_parallel_no_ticks() {
    let plot = ParallelPlot::new()
        .with_axis_names(["X1", "X2", "X3", "X4"])
        .with_row_group("A", vec![1.0, 2.0, 3.0, 4.0])
        .with_row_group("B", vec![4.0, 3.0, 2.0, 1.0])
        .with_axis_ticks(false);
    write_parallel("no_ticks", plot, "No Tick Labels");
}

#[test]
fn test_parallel_axis_bands() {
    let mut plot = ParallelPlot::new()
        .with_axis_names(["A", "B", "C", "D"])
        .with_axis_bands(true);
    for (group, rows) in iris_data() {
        for row in rows {
            plot = plot.with_row_group(group, row);
        }
    }
    let svg = write_parallel("axis_bands", plot, "With Axis Bands");
    assert!(svg.contains("f5f5f5"), "Expected axis band rects");
}

#[test]
fn test_parallel_no_normalize() {
    // All axes share a common scale
    let plot = ParallelPlot::new()
        .with_axis_names(["Q1", "Q2", "Q3"])
        .with_row_group("X", vec![10.0, 20.0, 30.0])
        .with_row_group("Y", vec![25.0, 15.0, 35.0])
        .with_normalize(false);
    write_parallel("no_normalize", plot, "Shared Scale (No Normalize)");
}

#[test]
fn test_parallel_custom_colors() {
    let plot = ParallelPlot::new()
        .with_axis_names(["Sepal.L", "Sepal.W", "Petal.L", "Petal.W"])
        .with_row_group("setosa", vec![5.1, 3.5, 1.4, 0.2])
        .with_row_group("versicolor", vec![7.0, 3.2, 4.7, 1.4])
        .with_row_group("virginica", vec![6.3, 3.3, 6.0, 2.5])
        .with_group_colors(["#e41a1c", "#377eb8", "#4daf4a"]);
    let svg = write_parallel("custom_colors", plot, "Custom Colors");
    assert!(svg.contains("e41a1c"), "Expected custom color in SVG");
}

#[test]
fn test_parallel_legend() {
    let mut plot = ParallelPlot::new()
        .with_axis_names(["Sepal.L", "Sepal.W", "Petal.L", "Petal.W"])
        .with_legend("Species");
    for (group, rows) in iris_data() {
        for row in rows {
            plot = plot.with_row_group(group, row);
        }
    }
    let svg = write_parallel("legend", plot, "Iris with Legend");
    assert!(svg.contains("setosa"), "Expected group name in legend");
    assert!(svg.contains("versicolor"), "Expected group name in legend");
}

#[test]
fn test_parallel_gene_expression() {
    // Simulated gene expression across 4 conditions, 3 gene clusters
    let conditions = ["WT", "KO", "Rescue", "Control"];
    let mut plot = ParallelPlot::new()
        .with_axis_names(conditions)
        .with_legend("Cluster")
        .with_opacity(0.5);

    // Cluster 1: high in KO
    for i in 0..8 {
        let base = 2.0 + i as f64 * 0.1;
        plot = plot.with_row_group("Cluster 1", vec![base, base + 3.0, base + 0.5, base]);
    }
    // Cluster 2: decreasing
    for i in 0..6 {
        let base = 5.0 - i as f64 * 0.2;
        plot = plot.with_row_group("Cluster 2", vec![base, base - 1.0, base - 2.0, base - 2.5]);
    }
    // Cluster 3: stable
    for i in 0..5 {
        let base = 3.0 + i as f64 * 0.05;
        plot = plot.with_row_group("Cluster 3", vec![base, base + 0.1, base - 0.1, base]);
    }

    let svg = write_parallel("gene_expression", plot, "Gene Expression Profiles");
    assert!(svg.contains("<path"));
    assert!(svg.contains("Cluster 1"));
}

#[test]
fn test_parallel_stroke_width() {
    let plot = ParallelPlot::new()
        .with_axis_names(["A", "B", "C"])
        .with_row_group("X", vec![1.0, 3.0, 2.0])
        .with_row_group("Y", vec![3.0, 1.0, 3.0])
        .with_stroke_width(3.0)
        .with_opacity(0.9);
    write_parallel("stroke_width", plot, "Thick Lines");
}

#[test]
fn test_parallel_many_axes() {
    let axis_names = (0..8).map(|i| format!("Dim{}", i + 1)).collect::<Vec<_>>();
    let mut plot = ParallelPlot::new().with_axis_names(axis_names);
    for k in 0..20 {
        let vals: Vec<f64> = (0..8)
            .map(|i| (k as f64 * 0.3 + i as f64 * 0.5).sin() * 5.0 + 5.0)
            .collect();
        plot = plot.with_row_group(format!("G{}", k % 3 + 1), vals);
    }
    write_parallel("many_axes", plot, "8-Dimensional Parallel Plot");
}

#[test]
fn test_parallel_curved() {
    let mut plot = ParallelPlot::new()
        .with_axis_names(["Sepal.L", "Sepal.W", "Petal.L", "Petal.W"])
        .with_curved(true)
        .with_opacity(0.55);
    for (group, rows) in iris_data() {
        for row in rows {
            plot = plot.with_row_group(group, row);
        }
    }
    let svg = write_parallel("curved", plot, "Iris — Curved Lines");
    // Curved paths use C (cubicBezier) commands, not just L
    assert!(
        svg.contains(" C "),
        "Expected cubic bezier commands in curved SVG"
    );
}

#[test]
fn test_parallel_mean_lines() {
    let mut plot = ParallelPlot::new()
        .with_axis_names(["Sepal.L", "Sepal.W", "Petal.L", "Petal.W"])
        .with_mean(true)
        .with_opacity(0.3);
    for (group, rows) in iris_data() {
        for row in rows {
            plot = plot.with_row_group(group, row);
        }
    }
    let svg = write_parallel("mean_lines", plot, "Iris — With Mean Lines");
    // Mean lines are drawn as paths; we should have at least n_rows + n_groups paths
    let path_count = svg.matches("<path").count();
    // 15 individual rows + 3 mean lines = 18 minimum
    assert!(
        path_count >= 18,
        "Expected individual + mean line paths, got {path_count}"
    );
}

#[test]
fn test_parallel_curved_with_mean() {
    let mut plot = ParallelPlot::new()
        .with_axis_names(["Sepal.L", "Sepal.W", "Petal.L", "Petal.W"])
        .with_curved(true)
        .with_mean(true)
        .with_opacity(0.4)
        .with_legend("Species");
    for (group, rows) in iris_data() {
        for row in rows {
            plot = plot.with_row_group(group, row);
        }
    }
    write_parallel("curved_mean", plot, "Iris — Curved + Means");
}

#[test]
fn test_parallel_inverted_axis() {
    // Invert the second axis (index 1) so high Sepal.W values appear at bottom
    let mut plot = ParallelPlot::new()
        .with_axis_names(["Sepal.L", "Sepal.W", "Petal.L", "Petal.W"])
        .with_invert_axis(1);
    for (group, rows) in iris_data() {
        for row in rows {
            plot = plot.with_row_group(group, row);
        }
    }
    let svg = write_parallel("inverted_axis", plot, "Iris — Sepal.W Inverted");
    // Inverted axes get a ▼ glyph and orange fill on the axis label
    assert!(
        svg.contains("▼"),
        "Expected ▼ indicator glyph for inverted axis"
    );
    assert!(
        svg.contains("d46000"),
        "Expected orange color for inverted axis"
    );
}

#[test]
fn test_parallel_inverted_multiple_axes() {
    let mut plot = ParallelPlot::new()
        .with_axis_names(["A", "B", "C", "D"])
        .with_inverted_axes([0, 2])
        .with_curved(true);
    for i in 0..10 {
        let v = i as f64;
        plot = plot.with_row_group("X", vec![v, 10.0 - v, v * 0.5, 10.0 - v * 0.5]);
    }
    write_parallel("multi_inverted", plot, "Multiple Inverted Axes");
}

#[test]
fn test_parallel_label_within_canvas() {
    // With legend: verify the last axis label is not bleeding into legend space
    // (just a render-completion smoke test — visual check via SVG output)
    let mut plot = ParallelPlot::new()
        .with_axis_names(["Alpha", "Beta", "Gamma", "Delta", "Epsilon"])
        .with_legend("Groups");
    for i in 0..6 {
        let v = i as f64;
        plot = plot.with_row_group(
            format!("G{}", i % 3),
            vec![v, v + 1.0, v - 1.0, v * 0.5, v + 2.0],
        );
    }
    let svg = write_parallel("label_within_canvas", plot, "Label Placement With Legend");
    assert!(svg.contains("Epsilon"), "Expected last axis label in SVG");
    assert!(svg.contains("Alpha"), "Expected first axis label in SVG");
}
