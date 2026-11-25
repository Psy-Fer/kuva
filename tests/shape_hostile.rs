//! Matrix plots with ragged rows, or label and coordinate lists that don't
//! match the matrix, must render instead of panicking. The renderers took
//! loop bounds from the first row or the label count and indexed every row
//! with them.

use kuva::backend::svg::SvgBackend;
use kuva::plot::{Clustermap, ClustermapNorm, ContourPlot, Heatmap, PhyloTree, Surface3DPlot};
use kuva::render::{layout::Layout, plots::Plot, render::render_multiple};

fn svg(plot: Plot) -> String {
    let plots = vec![plot];
    let layout = Layout::auto_from_plots(&plots);
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

#[test]
fn heatmap_rows_longer_or_shorter_than_the_first_render() {
    // Row 1 is short and row 2 long. The tooltip and value label of each cell
    // must come from that cell: the old code worked out row and column from a
    // running index, which assumed every row was full.
    let data = vec![vec![1.0, 2.0, 3.0], vec![4.0], vec![5.0, 6.0, 7.0, 8.0]];
    let plot = Heatmap::new()
        .with_data(data)
        .with_values()
        .with_tooltips()
        .with_x_edges([0.0, 1.0, 2.0, 4.0]);
    let out = svg(Plot::Heatmap(plot));
    for v in ["1.00", "4.00", "5.00", "7.00"] {
        assert!(out.contains(&format!(": {v}</title>")), "tooltip {v}");
    }
    // The fourth cell of the long row is past the first row's width.
    assert!(!out.contains("8.00"));

    // Same drawing as the long row cut to the first row's width.
    let cut = vec![vec![1.0, 2.0, 3.0], vec![5.0, 6.0, 7.0]];
    let long = vec![vec![1.0, 2.0, 3.0], vec![5.0, 6.0, 7.0, 2.0]];
    let render = |d| svg(Plot::Heatmap(Heatmap::new().with_data(d).with_values()));
    assert_eq!(render(long), render(cut));
}

fn grid() -> (Vec<Vec<f64>>, Vec<f64>, Vec<f64>) {
    let z = (0..5)
        .map(|r| (0..5).map(|c| ((r * c) % 7) as f64).collect())
        .collect();
    let x = (0..5).map(|c| c as f64).collect();
    let y = (0..5).map(|r| r as f64 * 2.0).collect();
    (z, x, y)
}

fn contour(z: Vec<Vec<f64>>, x: Vec<f64>, y: Vec<f64>, filled: bool) -> String {
    let mut c = ContourPlot::new()
        .with_grid(z, x, y)
        .with_levels(&[1.0, 2.5, 4.0]);
    if filled {
        c = c.with_filled();
    }
    // Fixed axes: the auto range follows the coordinate lists, which differ in
    // length between the inputs compared here.
    let plots = vec![Plot::Contour(c)];
    let layout = Layout::auto_from_plots(&plots)
        .with_x_axis_min(0.0)
        .with_x_axis_max(4.0)
        .with_y_axis_min(0.0)
        .with_y_axis_max(8.0);
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

#[test]
fn contour_with_ragged_grid_or_short_coordinates_draws_the_common_part() {
    for filled in [false, true] {
        let (z, x, y) = grid();
        // Cut to 4 columns and 4 rows. The last row and column hold no
        // extreme values, so the colour range is the same as the full grid's.
        let cut_z: Vec<Vec<f64>> = z[..4].iter().map(|r| r[..4].to_vec()).collect();
        let want =
            |x: &[f64], y: &[f64]| contour(cut_z.clone(), x[..4].to_vec(), y[..4].to_vec(), filled);

        // A ragged row limits the columns.
        let mut ragged = z[..4].to_vec();
        ragged[2].truncate(4);
        assert_eq!(
            contour(ragged, x.clone(), y[..4].to_vec(), filled),
            want(&x, &y)
        );

        // Short coordinate lists limit columns and rows.
        assert_eq!(
            contour(z.clone(), x[..4].to_vec(), y[..4].to_vec(), filled),
            want(&x, &y)
        );

        // Single row or column, empty rows, longer coordinates: no panic.
        for (z, x, y) in [
            (vec![vec![1.0, 2.0, 3.0]], x.clone(), y.clone()),
            (vec![vec![1.0], vec![2.0]], x.clone(), y.clone()),
            (vec![vec![], vec![1.0, 2.0]], x.clone(), y.clone()),
            (
                z.clone(),
                [x.clone(), x.clone()].concat(),
                [y.clone(), y.clone()].concat(),
            ),
        ] {
            assert!(contour(z, x, y, filled).contains("<svg"));
        }
    }
}

#[test]
fn surface_with_ragged_rows_renders() {
    let rows = vec![
        vec![1.0, 2.0, 3.0, 4.0],
        vec![2.0, 3.0],
        vec![],
        vec![3.0, 4.0, 5.0, 6.0, 7.0],
        vec![1.0, 1.0, 1.0, 1.0],
    ];
    for plot in [
        Surface3DPlot::new(rows.clone()),
        Surface3DPlot::new(rows.clone())
            .with_x_coords(vec![0.0, 1.0])
            .with_y_coords(vec![0.0]),
    ] {
        assert!(svg(Plot::Surface3D(plot)).contains("<svg"));
    }
}

fn labels(prefix: &str, n: usize) -> Vec<String> {
    (0..n).map(|i| format!("{prefix}{i}")).collect()
}

fn clustermap(
    data: Vec<Vec<f64>>,
    rows: Vec<String>,
    cols: Vec<String>,
    norm: ClustermapNorm,
    cluster: bool,
) -> String {
    let cm = Clustermap::new()
        .with_data(data)
        .with_row_labels(rows)
        .with_col_labels(cols)
        .with_normalization(norm)
        .with_cluster_rows(cluster)
        .with_cluster_cols(cluster)
        .with_values()
        .with_tooltips();
    svg(Plot::Clustermap(cm))
}

#[test]
fn clustermap_with_ragged_rows_or_mismatched_labels_matches_the_fixed_shape() {
    let base: Vec<Vec<f64>> = (0..5)
        .map(|r| (0..4).map(|c| ((r * 3 + c * 5) % 11) as f64).collect())
        .collect();
    // Row 1 short, row 3 long; 4 columns from the first row.
    let mut ragged = base.clone();
    ragged[1].truncate(2);
    ragged[3].push(99.0);
    let mut padded = base.clone();
    padded[1].truncate(2);
    padded[1].extend([f64::NAN, f64::NAN]);

    for norm in [
        ClustermapNorm::None,
        ClustermapNorm::RowZScore,
        ClustermapNorm::ColZScore,
    ] {
        for cluster in [false, true] {
            let want = clustermap(
                padded.clone(),
                labels("r", 5),
                labels("c", 4),
                norm.clone(),
                cluster,
            );
            // Ragged rows.
            let got = clustermap(
                ragged.clone(),
                labels("r", 5),
                labels("c", 4),
                norm.clone(),
                cluster,
            );
            assert_eq!(got, want, "{norm:?} cluster {cluster}");
            // Too many labels are cut, too few are filled with their index.
            let got = clustermap(
                padded.clone(),
                labels("r", 8),
                labels("c", 9),
                norm.clone(),
                cluster,
            );
            assert_eq!(got, want, "{norm:?} cluster {cluster}");
            let short = clustermap(
                padded.clone(),
                labels("r", 2),
                labels("c", 1),
                norm.clone(),
                cluster,
            );
            let mut rows = labels("r", 2);
            rows.extend(["2", "3", "4"].map(String::from));
            let mut cols = labels("c", 1);
            cols.extend(["1", "2", "3"].map(String::from));
            assert_eq!(
                short,
                clustermap(padded.clone(), rows, cols, norm.clone(), cluster)
            );
        }
    }
}

#[test]
fn clustermap_categorical_with_ragged_rows_renders() {
    let cats = vec![
        vec!["a".to_string(), "b".into(), "a".into()],
        vec!["b".to_string()],
        vec!["a".to_string(), "a".into(), "b".into(), "c".into()],
    ];
    let cm = Clustermap::new()
        .with_categorical_data(cats)
        .with_row_labels(labels("r", 6))
        .with_col_labels(labels("c", 1))
        .with_tooltips();
    assert!(svg(Plot::Clustermap(cm)).contains("<svg"));
}

#[test]
fn distance_matrix_smaller_than_the_labels_builds_a_tree() {
    let labels = ["a", "b", "c", "d"];
    for dist in [
        vec![vec![0.0, 1.0], vec![1.0, 0.0]],
        vec![vec![0.0, 1.0, 4.0], vec![1.0], vec![4.0, 2.0, 0.0, 3.0]],
        vec![],
    ] {
        let tree = PhyloTree::from_distance_matrix(&labels, &dist).with_phylogram();
        let leaves = tree.nodes.iter().filter(|n| n.children.is_empty()).count();
        assert_eq!(leaves, 4);
        assert!(tree.nodes.iter().all(|n| n.branch_length.is_finite()));
        assert!(svg(Plot::PhyloTree(tree)).contains("<svg"));
    }
}
