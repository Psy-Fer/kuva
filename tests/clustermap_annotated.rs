//! Annotated / categorical heatmaps built on `Clustermap`: label placement, categorical
//! annotation tracks with an automatic legend, categorical cells, and big-table sizing.
//! Writes `test_outputs/clustermap_annotated_*.svg` for visual review.
mod common;

use kuva::backend::svg::SvgBackend;
use kuva::plot::clustermap::{
    AnnotationPlacement, AnnotationTrack, Clustermap, ColLabelSide, RowLabelSide,
};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

fn render(cm: Clustermap, title: &str) -> String {
    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots).with_title(title);
    // Tests read individual `<rect>` cells back, so rect batching is off.
    SvgBackend::new()
        .with_rect_batching(false)
        .render_scene(&render_multiple(plots, layout))
}

fn write(name: &str, svg: &str) {
    common::write_test_output(format!("test_outputs/clustermap_annotated_{name}.svg"), svg)
        .unwrap();
}

/// Genes x samples, with a clear two-group structure so clustering is visible.
fn expression() -> (Vec<Vec<f64>>, Vec<String>, Vec<String>) {
    let genes = [
        "BRCA1", "TP53", "MYC", "EGFR", "KRAS", "PTEN", "CDK4", "RB1", "VEGFA", "ERBB2",
    ];
    let samples = [
        "ctrl_1", "ctrl_2", "ctrl_3", "treat_1", "treat_2", "treat_3", "rescue_1", "rescue_2",
    ];
    let data = (0..genes.len())
        .map(|g| {
            (0..samples.len())
                .map(|s| {
                    let up = (g % 2 == 0) == (3..6).contains(&s);
                    let base = if up { 6.0 } else { 2.0 };
                    base + ((g * 7 + s * 3) % 5) as f64 * 0.4
                })
                .collect()
        })
        .collect();
    (
        data,
        genes.iter().map(|s| s.to_string()).collect(),
        samples.iter().map(|s| s.to_string()).collect(),
    )
}

/// `<text ...>label</text>` attributes for a given label.
fn text_attrs<'a>(svg: &'a str, label: &str) -> &'a str {
    let end = svg
        .find(&format!(">{label}</text>"))
        .unwrap_or_else(|| panic!("no label {label}"));
    let start = svg[..end].rfind("<text").unwrap();
    &svg[start..end]
}

fn attr(tag: &str, name: &str) -> f64 {
    let key = format!(" {name}=\"");
    let from = tag.find(&key).unwrap() + key.len();
    tag[from..from + tag[from..].find('"').unwrap()]
        .parse()
        .unwrap()
}

/// Bounding box of the heatmap body: the `n_cells` rects that share one size (annotation
/// strips and the colorbar have other sizes).
fn body_extent(svg: &str, n_cells: usize) -> (f64, f64, f64, f64) {
    let rects: Vec<(f64, f64, f64, f64)> = svg
        .split("<rect ")
        .skip(1)
        .map(|r| format!(" {}", &r[..r.find('>').unwrap()]))
        .filter(|tag| tag.contains(" x=\"") && tag.contains(" height=\""))
        .map(|tag| {
            (
                attr(&tag, "x"),
                attr(&tag, "y"),
                attr(&tag, "width"),
                attr(&tag, "height"),
            )
        })
        .collect();
    let same_size = |a: &(f64, f64, f64, f64), b: &(f64, f64, f64, f64)| {
        (a.2 - b.2).abs() < 1e-6 && (a.3 - b.3).abs() < 1e-6
    };
    let size = rects
        .iter()
        .find(|r| rects.iter().filter(|o| same_size(r, o)).count() >= n_cells)
        .expect("body cells");
    let body: Vec<_> = rects.iter().filter(|r| same_size(r, size)).collect();
    let x0 = body.iter().map(|r| r.0).fold(f64::INFINITY, f64::min);
    let y0 = body.iter().map(|r| r.1).fold(f64::INFINITY, f64::min);
    let x1 = body.iter().map(|r| r.0 + r.2).fold(0.0, f64::max);
    let y1 = body.iter().map(|r| r.1 + r.3).fold(0.0, f64::max);
    (x0, y0, x1, y1)
}

#[test]
fn labels_default_bottom_right() {
    let (data, genes, samples) = expression();
    let n = data.len() * data[0].len();
    let cm = Clustermap::new()
        .with_data(data)
        .with_row_labels(genes)
        .with_col_labels(samples)
        .with_legend("log2 expr");
    let svg = render(cm, "Labels: default (bottom / right)");
    write("labels_default", &svg);
    let (x0, y0, x1, y1) = body_extent(&svg, n);
    let row = text_attrs(&svg, "BRCA1");
    assert!(attr(row, "x") > x1, "row labels right of the body");
    let col = text_attrs(&svg, "ctrl_1");
    assert!(attr(col, "y") > y1, "col labels below the body");
    assert!(col.contains("rotate(-45"), "default slant 45");
    assert!(x0 > 0.0 && y0 > 0.0);
    // The colorbar is drawn now (it was silently missing before).
    assert!(svg.contains(">log2 expr</text>"), "colorbar title");
}

#[test]
fn labels_top_left_sit_against_the_body() {
    let (data, genes, samples) = expression();
    let n = data.len() * data[0].len();
    let condition = [
        "#999999", "#999999", "#999999", "#d55e00", "#d55e00", "#d55e00", "#0072b2", "#0072b2",
    ];
    let pathway = ["#009e73", "#cc79a7"].repeat(5);
    let cm = Clustermap::new()
        .with_data(data)
        .with_row_labels(genes)
        .with_col_labels(samples)
        .with_col_annotation(AnnotationTrack::new(condition).with_label("Condition"))
        .with_row_annotation(AnnotationTrack::new(pathway).with_label("Pathway"))
        .with_col_labels_position(ColLabelSide::Top)
        .with_row_labels_position(RowLabelSide::Left)
        .with_legend("log2 expr");
    let svg = render(cm, "Labels: top / left");
    write("labels_top_left", &svg);
    let (x0, y0, _, _) = body_extent(&svg, n);
    let row = text_attrs(&svg, "BRCA1");
    assert!(row.contains("text-anchor=\"end\""));
    let rx = attr(row, "x");
    assert!(
        rx < x0 && x0 - rx < 10.0,
        "row labels just left of the body"
    );
    let col = text_attrs(&svg, "ctrl_1");
    let cy = attr(col, "y");
    assert!(cy < y0 && y0 - cy < 10.0, "col labels just above the body");
    assert!(col.contains("text-anchor=\"start\""));
    // The row strip's label goes on top too, reading upward beside the column labels.
    let strip = text_attrs(&svg, "Pathway");
    assert!(attr(strip, "y") < y0, "row-track label above the body");
    assert!(strip.contains("rotate(-90"));
}

#[test]
fn labels_vertical_and_flat() {
    let (data, genes, samples) = expression();
    let n = data.len() * data[0].len();
    for (deg, name) in [(90.0, "vertical"), (0.0, "flat")] {
        let cm = Clustermap::new()
            .with_data(data.clone())
            .with_row_labels(genes.clone())
            .with_col_labels(samples.clone())
            .with_cluster_cols(false)
            .with_col_labels_position(ColLabelSide::Top)
            .with_col_label_rotation(deg)
            .with_legend("log2 expr");
        let svg = render(cm, &format!("Column labels at {deg} degrees"));
        write(&format!("labels_{name}"), &svg);
        let (_, y0, _, _) = body_extent(&svg, n);
        let col = text_attrs(&svg, "rescue_2");
        assert!(attr(col, "y") < y0, "{name}: above the body");
        if deg == 0.0 {
            assert!(!col.contains("rotate("), "flat labels are not rotated");
            assert!(col.contains("text-anchor=\"middle\""));
        } else {
            assert!(col.contains("rotate(-90"), "vertical labels");
        }
    }
}

/// The label band is measured from the labels, not a fixed 80 px: short labels give a
/// taller body than long ones on the same canvas.
#[test]
fn col_label_band_is_measured() {
    let (data, genes, _) = expression();
    let n = data.len() * data[0].len();
    let body_h = |labels: Vec<String>| {
        let cm = Clustermap::new()
            .with_data(data.clone())
            .with_row_labels(genes.clone())
            .with_col_labels(labels)
            .with_cluster_cols(false)
            .with_cluster_rows(false);
        let svg = render(cm, "band");
        let (_, y0, _, y1) = body_extent(&svg, n);
        y1 - y0
    };
    let short = body_h((1..=8).map(|i| format!("S{i}")).collect());
    let long = body_h(
        (1..=8)
            .map(|i| format!("a_rather_long_sample_name_{i}"))
            .collect(),
    );
    assert!(short > long + 40.0, "short {short} vs long {long}");
}

fn condition_track() -> AnnotationTrack {
    AnnotationTrack::categorical([
        "ctrl", "ctrl", "ctrl", "treat", "treat", "treat", "rescue", "rescue",
    ])
    .with_label("Condition")
}

/// The expression matrix with condition + batch column tracks and a pathway row track,
/// labels top/left: the full "annotated heatmap" layout.
#[test]
fn categorical_tracks_add_a_legend() {
    let (data, genes, samples) = expression();
    let cm = Clustermap::new()
        .with_data(data)
        .with_row_labels(genes)
        .with_col_labels(samples)
        .with_col_annotation(
            condition_track()
                .with_category_color("ctrl", "#999999")
                .with_category_label("treat", "Treated (10 uM)")
                .with_category_order(["ctrl", "treat", "rescue"]),
        )
        .with_col_annotation(
            AnnotationTrack::categorical(["A", "B", "A", "B", "A", "B", "A", ""])
                .with_label("Batch")
                .with_palette(kuva::Palette::okabe_ito()),
        )
        .with_row_annotation(
            AnnotationTrack::categorical(
                ["DNA repair", "Growth"]
                    .repeat(5)
                    .into_iter()
                    .collect::<Vec<_>>(),
            )
            .with_label("Pathway"),
        )
        .with_col_labels_position(ColLabelSide::Top)
        .with_row_labels_position(RowLabelSide::Left)
        .with_legend("log2 expr");
    let svg = render(cm, "Annotated heatmap");
    write("tracks", &svg);

    // One legend group per categorical track, titled with the track label.
    for title in ["Condition", "Batch", "Pathway"] {
        assert!(svg.contains(&format!(">{title}</text>")), "group {title}");
    }
    // Custom label, explicit colour, NA entry for the empty batch value.
    assert!(svg.contains(">Treated (10 uM)</text>"));
    assert!(svg.contains(">NA</text>"));
    assert!(svg.contains("#999999"), "ctrl override colour used");
    // The colorbar is still there alongside the legend.
    assert!(svg.contains(">log2 expr</text>"));
}

/// Colours depend only on category order, never on hash order: rendering twice gives the
/// same SVG, and the first category gets the palette's first colour.
#[test]
fn categorical_colours_are_deterministic() {
    let entries = condition_track().category_entries();
    let order: Vec<&str> = entries.iter().map(|e| e.value.as_str()).collect();
    assert_eq!(order, ["ctrl", "treat", "rescue"], "first appearance");
    assert_eq!(entries[0].color, kuva::Palette::category10()[0].to_string());

    let reordered = condition_track()
        .with_category_order(["rescue"])
        .category_entries();
    let order: Vec<&str> = reordered.iter().map(|e| e.value.as_str()).collect();
    assert_eq!(order, ["rescue", "ctrl", "treat"], "explicit order first");
    // An explicit colour doesn't use up a palette colour.
    let fixed = condition_track()
        .with_category_color("ctrl", "#000000")
        .category_entries();
    assert_eq!(fixed[1].color, kuva::Palette::category10()[0].to_string());

    let (data, genes, samples) = expression();
    let make = || {
        Clustermap::new()
            .with_data(data.clone())
            .with_row_labels(genes.clone())
            .with_col_labels(samples.clone())
            .with_col_annotation(condition_track())
    };
    assert_eq!(render(make(), "x"), render(make(), "x"));
}

#[test]
fn annotation_legend_can_be_turned_off_and_raw_colour_tracks_add_none() {
    let (data, _, _) = expression();
    let off = Clustermap::new()
        .with_data(data.clone())
        .with_col_annotation(condition_track())
        .with_annotation_legend(false);
    assert!(off.annotation_legend_groups().is_empty());
    let raw = Clustermap::new()
        .with_data(data)
        .with_col_annotation(AnnotationTrack::new(["#ff0000"; 8]).with_label("Raw"));
    assert!(raw.annotation_legend_groups().is_empty());
}

/// Tracks on the default palette continue through it, so two tracks never share a colour
/// (until category10 runs out); a track with its own palette starts that palette fresh.
#[test]
fn default_palette_tracks_do_not_reuse_colours() {
    let (data, _, _) = expression();
    let cm = Clustermap::new()
        .with_data(data)
        .with_col_annotation(condition_track())
        .with_row_annotation(AnnotationTrack::categorical(
            ["DNA repair", "Growth"]
                .repeat(5)
                .into_iter()
                .collect::<Vec<_>>(),
        ))
        .with_row_annotation(
            AnnotationTrack::categorical(["x", "y"].repeat(5).into_iter().collect::<Vec<_>>())
                .with_palette(kuva::Palette::okabe_ito()),
        );
    let (col, row) = cm.annotation_entries();
    let c10 = kuva::Palette::category10();
    let colours = |e: &Vec<kuva::plot::clustermap::CategoryEntry>| {
        e.iter().map(|x| x.color.clone()).collect::<Vec<_>>()
    };
    assert_eq!(colours(&col[0]), [&c10[0], &c10[1], &c10[2]]);
    assert_eq!(
        colours(&row[0]),
        [&c10[3], &c10[4]],
        "continues after Condition"
    );
    let okabe = kuva::Palette::okabe_ito();
    assert_eq!(
        colours(&row[1]),
        [&okabe[0], &okabe[1]],
        "own palette starts fresh"
    );
}

fn genotypes() -> (Vec<Vec<String>>, Vec<String>, Vec<String>) {
    let variants: Vec<String> = (1..=12).map(|i| format!("rs{}", 1000 + i * 37)).collect();
    let samples: Vec<String> = (1..=9).map(|i| format!("HG00{i}")).collect();
    let calls = ["ref", "het", "hom", "ref", "het", "ref", ""];
    let data = (0..variants.len())
        .map(|v| {
            (0..samples.len())
                .map(|s| calls[(v * v * 3 + s * 5 + v * s * 2 + 1) % calls.len()].to_string())
                .collect()
        })
        .collect();
    (data, variants, samples)
}

/// Genotype matrix: categorical cells with explicit colours, custom legend text and a
/// population track. No colorbar, no clustering.
#[test]
fn categorical_cells_genotype_matrix() {
    let (data, variants, samples) = genotypes();
    let cm = Clustermap::new()
        .with_categorical_data(data)
        .with_row_labels(variants)
        .with_col_labels(samples)
        .with_category_colors([("ref", "#eeeeee"), ("het", "#4c9be8"), ("hom", "#1a3d6e")])
        .with_category_labels([
            ("ref", "Reference"),
            ("het", "Heterozygous"),
            ("hom", "Homozygous alt"),
        ])
        .with_category_order(["ref", "het", "hom"])
        .with_col_annotation(
            AnnotationTrack::categorical([
                "EUR", "EUR", "EUR", "AFR", "AFR", "AFR", "EAS", "EAS", "EAS",
            ])
            .with_label("Population"),
        )
        .with_col_labels_position(ColLabelSide::Top)
        .with_legend("Genotype");
    assert!(!cm.uses_colorbar());
    let svg = render(cm, "Genotypes");
    write("genotypes", &svg);
    for t in [
        "Genotype",
        "Reference",
        "Heterozygous",
        "Homozygous alt",
        "NA",
        "Population",
    ] {
        assert!(svg.contains(&format!(">{t}</text>")), "legend text {t}");
    }
    assert!(
        svg.contains("#4c9be8") && svg.contains("#cccccc"),
        "het colour and NA grey"
    );
    // No clustering: rows stay in input order, so rs1037 is the top row label.
    let first = svg.find(">rs1037</text>").unwrap();
    assert!(first < svg.find(">rs1074</text>").unwrap());
    // No dendrogram lines.
    assert!(!svg.contains("<line"), "no dendrograms in categorical mode");
}

/// Binned numeric cells: copy number with a labelled swatch legend; values outside every
/// bin (negative) are NA.
#[test]
fn discrete_color_map_copy_number() {
    use kuva::plot::clustermap::DiscreteColorMap;
    let data: Vec<Vec<f64>> = (0..8)
        .map(|r| {
            (0..10)
                .map(|c| ((r * r + c * 3 + r * c) % 7) as f64 - 1.0)
                .collect()
        })
        .collect();
    let map = DiscreteColorMap::new()
        .with_bin(0.0, "#2166ac", "0 (deletion)")
        .with_bin(1.0, "#92c5de", "1")
        .with_bin(2.0, "#f7f7f7", "2 (normal)")
        .with_bin(3.0, "#f4a582", "3")
        .with_bin(4.0, "#b2182b", "4+");
    let cm = Clustermap::new()
        .with_data(data)
        .with_row_labels((1..=8).map(|i| format!("chr{i}")))
        .with_col_labels((1..=10).map(|i| format!("tumour_{i}")))
        .with_discrete_color_map(map)
        .with_cluster_rows(false)
        .with_legend("Copy number");
    let (title, entries) = cm.cell_legend_group().unwrap();
    assert_eq!(title, "Copy number");
    let labels: Vec<&str> = entries.iter().map(|e| e.label.as_str()).collect();
    assert_eq!(labels, ["0 (deletion)", "1", "2 (normal)", "3", "4+", "NA"]);
    let svg = render(cm, "Copy number");
    write("copy_number", &svg);
    assert!(svg.contains("#b2182b") && svg.contains("#cccccc"));
    assert!(
        !svg.contains("linearGradient"),
        "swatch legend, not a gradient colorbar"
    );
}

/// Distinct x (or y) positions of the body cells, sorted.
fn cell_positions(svg: &str, n_cells: usize, axis: &str) -> Vec<f64> {
    let (x0, y0, x1, y1) = body_extent(svg, n_cells);
    let mut v: Vec<f64> = svg
        .split("<rect ")
        .skip(1)
        .map(|r| format!(" {}", &r[..r.find('>').unwrap()]))
        .filter(|t| t.contains(" x=\"") && t.contains(" height=\""))
        .filter(|t| {
            let (x, y) = (attr(t, "x"), attr(t, "y"));
            x >= x0 - 1e-6 && x < x1 && y >= y0 - 1e-6 && y < y1
        })
        .map(|t| attr(&t, axis))
        .collect();
    v.sort_by(f64::total_cmp);
    v.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
    v
}

/// `with_cell_size` fixes the cell pitch exactly and grows the canvas with the table.
#[test]
fn cell_size_sizes_the_canvas() {
    let make = |rows: usize, cols: usize| {
        let data: Vec<Vec<f64>> = (0..rows)
            .map(|r| {
                (0..cols)
                    .map(|c| ((r * 7 + c * 13 + r * c) % 11) as f64)
                    .collect()
            })
            .collect();
        Clustermap::new()
            .with_data(data)
            .with_row_labels((0..rows).map(|i| format!("gene_{i:02}")))
            .with_col_labels((0..cols).map(|i| format!("sample_{i:02}")))
            .with_col_annotation(
                AnnotationTrack::categorical((0..cols).map(|c| ["a", "b", "c"][c % 3]))
                    .with_label("Group"),
            )
            .with_col_labels_position(ColLabelSide::Top)
            .with_col_label_rotation(90.0)
            .with_cell_size(14.0, 12.0)
            .with_legend("Value")
    };
    let size = |cm: Clustermap| {
        let plots = vec![Plot::Clustermap(cm)];
        let c = kuva::render::layout::ComputedLayout::from_layout(&Layout::auto_from_plots(&plots));
        (c.width, c.height)
    };
    let (w_small, h_small) = size(make(10, 8));
    let (w_big, h_big) = size(make(40, 30));
    assert!(
        (w_big - w_small - 22.0 * 14.0).abs() < 1e-6,
        "width grows by 22 cells"
    );
    assert!(
        (h_big - h_small - 30.0 * 12.0).abs() < 1e-6,
        "height grows by 30 cells"
    );

    let svg = render(make(40, 30), "Cell size 14 x 12");
    write("cell_size", &svg);
    let xs = cell_positions(&svg, 40 * 30, "x");
    let ys = cell_positions(&svg, 40 * 30, "y");
    assert_eq!(xs.len(), 30);
    assert_eq!(ys.len(), 40);
    assert!(
        ((xs[1] - xs[0]) - 14.0).abs() < 0.02,
        "x pitch {}",
        xs[1] - xs[0]
    );
    assert!(
        ((ys[1] - ys[0]) - 12.0).abs() < 0.02,
        "y pitch {}",
        ys[1] - ys[0]
    );
}

#[test]
fn cell_gap_sets_the_spacing() {
    let (data, genes, samples) = expression();
    let n = data.len() * data[0].len();
    let cm = Clustermap::new()
        .with_data(data)
        .with_row_labels(genes)
        .with_col_labels(samples)
        .with_cell_size(30.0, 20.0)
        .with_cell_gap(3.0);
    let svg = render(cm, "Cell gap 3");
    write("cell_gap", &svg);
    let tag = svg
        .split("<rect ")
        .skip(1)
        .map(|r| format!(" {}", &r[..r.find('>').unwrap()]))
        .find(|t| t.contains(" x=\"") && (attr(t, "height") - 17.0).abs() < 1e-6)
        .expect("a cell 20 - 3 px tall");
    assert!((attr(&tag, "width") - 27.0).abs() < 1e-6);
    let _ = n;
}

#[test]
fn na_label_replaces_na_in_the_legend() {
    let cm = Clustermap::new()
        .with_categorical_data([["a", ""], ["b", "a"]])
        .with_na_label("No call")
        .with_col_annotation(
            AnnotationTrack::categorical(["x", ""])
                .with_label("Track")
                .with_na_label("Unknown"),
        );
    let svg = render(cm, "na");
    assert!(svg.contains(">No call</text>") && svg.contains(">Unknown</text>"));
    assert!(!svg.contains(">NA</text>"));
}

/// `NextToCells` swaps strips and labels on the label side: strips against the cells, labels
/// outside them. Strip positions are read from the strip rects, labels from the text.
#[test]
fn annotation_placement_next_to_cells() {
    let (data, genes, samples) = expression();
    let n = data.len() * data[0].len();
    let make = |placement| {
        Clustermap::new()
            .with_data(data.clone())
            .with_row_labels(genes.clone())
            .with_col_labels(samples.clone())
            .with_col_annotation(condition_track())
            .with_row_annotation(
                AnnotationTrack::categorical(
                    ["DNA repair", "Growth"]
                        .repeat(5)
                        .into_iter()
                        .collect::<Vec<_>>(),
                )
                .with_label("Pathway"),
            )
            .with_col_labels_position(ColLabelSide::Top)
            .with_row_labels_position(RowLabelSide::Left)
            .with_col_label_rotation(90.0)
            .with_annotation_placement(placement)
    };
    let svg = render(
        make(AnnotationPlacement::NextToCells),
        "Strips next to the cells",
    );
    write("placement_next_to_cells", &svg);
    let (x0, y0, _, _) = body_extent(&svg, n);
    // The column labels end above the 15 px strip (plus gaps), not right against the body.
    let col = text_attrs(&svg, "ctrl_1");
    assert!(
        y0 - attr(col, "y") > 15.0,
        "labels outside the column strip"
    );
    // Row labels sit left of the 15 px row strip.
    let row = text_attrs(&svg, "BRCA1");
    assert!(x0 - attr(row, "x") > 15.0, "labels outside the row strip");

    // The default keeps labels against the cells.
    let svg = render(make(AnnotationPlacement::OutsideLabels), "default");
    let (x0, y0, _, _) = body_extent(&svg, n);
    assert!(y0 - attr(text_attrs(&svg, "ctrl_1"), "y") < 10.0);
    assert!(x0 - attr(text_attrs(&svg, "BRCA1"), "x") < 10.0);
}

#[test]
fn empty_category_palette_falls_back_to_default() {
    let cm = Clustermap::new()
        .with_categorical_data(vec![vec!["a", "b"], vec!["b", "a"]])
        .with_category_palette(Vec::<String>::new())
        .with_row_annotation(
            AnnotationTrack::categorical(["x", "y"]).with_palette(Vec::<String>::new()),
        );
    let svg = render(cm, "Empty palette");
    assert!(svg.contains("<svg"));
    assert!(!svg.contains("NaN"));
}

#[test]
fn ragged_col_zscore_with_discrete_color_map_does_not_panic() {
    use kuva::plot::clustermap::{ClustermapNorm, DiscreteColorMap};
    let cm = Clustermap::new()
        .with_data(vec![vec![1.0, 2.0], vec![3.0]])
        .with_normalization(ClustermapNorm::ColZScore)
        .with_discrete_color_map(DiscreteColorMap::new().with_bin(0.0, "#ff0000", "pos"));
    let legend = cm.cell_legend_group();
    assert!(legend.is_some());
    let svg = render(cm, "Ragged ColZScore");
    assert!(svg.contains("<svg"));
    assert!(!svg.contains("NaN"));
}

#[test]
fn duplicate_row_and_col_labels_preserve_all_rows() {
    let cm = Clustermap::new()
        .with_data(vec![vec![1.0, 2.0], vec![10.0, 20.0], vec![1.1, 2.1]])
        .with_row_labels(["GeneA", "GeneA", "GeneB"])
        .with_col_labels(["S", "S"])
        .with_values();
    let svg = render(cm, "Duplicate labels");
    assert!(
        svg.contains(">1.00</text>"),
        "row 0 value 1.00 must not be dropped"
    );
    assert!(
        svg.contains(">10.00</text>"),
        "row 1 value 10.00 must be preserved"
    );
    assert!(
        svg.contains(">1.10</text>"),
        "row 2 value 1.10 must be preserved"
    );
}

#[test]
fn nan_cell_does_not_poison_zscore_or_upgma() {
    use kuva::plot::clustermap::ClustermapNorm;
    let cm = Clustermap::new()
        .with_data(vec![
            vec![1.0, 2.0, f64::NAN],
            vec![10.0, 20.0, 30.0],
            vec![2.0, 4.0, 6.0],
        ])
        .with_row_labels(["R1", "R2", "R3"])
        .with_col_labels(["C1", "C2", "C3"])
        .with_normalization(ClustermapNorm::RowZScore)
        .with_values();
    let svg = render(cm, "NaN z-score");
    // Row 0's finite values (1.0, 2.0) have z-scores -1.00 and 1.00; the NaN cell shows NA.
    assert!(svg.contains(">-1.00</text>"), "finite cells keep z-score");
    assert!(svg.contains(">1.00</text>"), "finite cells keep z-score");
    assert!(svg.contains(">NA</text>"), "missing cell should show NA");
    assert!(
        !svg.contains(">NaN</text>"),
        "missing cell must not print NaN"
    );
    assert!(
        !svg.contains("x1=\"NaN\"") && !svg.contains("y1=\"NaN\"") && !svg.contains("x2=\"inf\""),
        "dendrogram coordinates must stay finite"
    );
}

#[test]
fn non_finite_library_rotation_and_cell_gap_stay_finite() {
    let cm = Clustermap::new()
        .with_data(vec![vec![1.0, 2.0], vec![3.0, 4.0]])
        .with_col_labels(["S1", "S2"])
        .with_col_label_rotation(f64::NAN)
        .with_cell_gap(f64::NAN);
    let svg = render(cm, "Non-finite layout opts");
    assert!(svg.contains("<svg"));
    assert!(!svg.contains("NaN"));
}

#[test]
fn small_scale_data_preserves_zscore_and_color_contrast() {
    use kuva::plot::clustermap::ClustermapNorm;
    let cm = Clustermap::new()
        .with_data(vec![vec![1e-20, 1e-18, 1e-16], vec![2e-20, 2e-18, 2e-16]])
        .with_cluster_rows(false)
        .with_cluster_cols(false)
        .with_normalization(ClustermapNorm::RowZScore)
        .with_values();
    let svg = render(cm, "Small-scale z-score");
    assert!(
        svg.contains(">1.41</text>"),
        "small-scale row should not collapse to 0.00 z-score"
    );
}
