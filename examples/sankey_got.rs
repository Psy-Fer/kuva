//! Game of Thrones Sankey/alluvial example using the real affiliation dataset.
//!
//! Run with:
//!
//! ```bash
//! cargo run --example sankey_got
//! ```
//!
//! Outputs are written to `test_outputs/`.

use kuva::backend::svg::SvgBackend;
use kuva::plot::SankeyPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use std::collections::HashMap;

const DATA: &str = include_str!("data/sankey_got.tsv");
const OUT: &str = "test_outputs";
const AXES: [&str; 9] = [
    "Origin",
    "Starting Affiliation",
    "End of S1",
    "End of S2",
    "End of S3",
    "End of S4",
    "End of S5",
    "End of S6",
    "End of S7",
];

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create test_outputs");

    let base = build_got_plot();
    let default_svg = render_svg(
        "sankey_got_default.svg",
        "Game of Thrones Affiliations — Input Order",
        base.clone(),
    );
    let crossings_svg = render_svg(
        "sankey_got_crossings_left.svg",
        "Game of Thrones Affiliations — Crossing Reduction",
        base.with_crossing_reduction()
            .with_left_coloring()
            .with_node_order_seed(42),
    );
    let rows = got_rows();
    let default_crossings = crossing_count_from_svg(&default_svg, &rows);
    let reduced_crossings = crossing_count_from_svg(&crossings_svg, &rows);
    let summary = format!(
        "Game of Thrones Sankey crossing counts\n\
         default_input_order\t{default_crossings}\n\
         crossing_reduction\t{reduced_crossings}\n\
         improvement\t{}\n",
        default_crossings.saturating_sub(reduced_crossings)
    );
    std::fs::write(format!("{OUT}/sankey_got_crossing_summary.txt"), &summary)
        .expect("could not write crossing summary");
    print!("{summary}");

    println!("Game of Thrones Sankey example SVGs written to {OUT}/");
}

fn build_got_plot() -> SankeyPlot {
    let mut plot = SankeyPlot::new().with_axis_names(AXES);
    for row in got_rows() {
        let strata: Vec<&str> = row.iter().map(String::as_str).collect();
        plot = plot.with_alluvium(strata, 1.0);
    }
    plot
}

fn got_rows() -> Vec<Vec<String>> {
    let mut lines = DATA.lines();
    let header: Vec<&str> = lines
        .next()
        .expect("missing header")
        .split('\t')
        .collect();
    let axis_indices = AXES
        .iter()
        .map(|name| {
            header
                .iter()
                .position(|col| col == name)
                .unwrap_or_else(|| panic!("missing column '{name}'"))
        })
        .collect::<Vec<_>>();
    let mut rows = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<String> = line.split('\t').map(str::to_string).collect();
        let mut ordered = Vec::with_capacity(axis_indices.len());
        for &idx in &axis_indices {
            ordered.push(fields[idx].clone());
        }
        rows.push(ordered);
    }
    rows
}

fn render_svg(filename: &str, title: &str, sankey: SankeyPlot) -> String {
    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title(title)
        .with_width(2400.0)
        .with_height(1800.0);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/{filename}"), svg).expect("could not write SVG");
    std::fs::read_to_string(format!("{OUT}/{filename}")).expect("could not re-read SVG")
}

fn crossing_count_from_svg(svg: &str, rows: &[Vec<String>]) -> usize {
    let orders = rendered_orders(svg);
    let mut total = 0usize;
    for axis in 0..AXES.len() - 1 {
        let left = &orders[axis];
        let right = &orders[axis + 1];
        for i in 0..rows.len() {
            for j in i + 1..rows.len() {
                let a0 = &rows[i][axis];
                let a1 = &rows[i][axis + 1];
                let b0 = &rows[j][axis];
                let b1 = &rows[j][axis + 1];
                if a0 == b0 || a1 == b1 {
                    continue;
                }
                let left_i = *left
                    .get(a0)
                    .unwrap_or_else(|| panic!("missing left node '{a0}' for axis {axis}"));
                let left_j = *left
                    .get(b0)
                    .unwrap_or_else(|| panic!("missing left node '{b0}' for axis {axis}"));
                let right_i = *right
                    .get(a1)
                    .unwrap_or_else(|| panic!("missing right node '{a1}' for axis {}", axis + 1));
                let right_j = *right
                    .get(b1)
                    .unwrap_or_else(|| panic!("missing right node '{b1}' for axis {}", axis + 1));
                if (left_i < left_j && right_i > right_j)
                    || (left_i > left_j && right_i < right_j)
                {
                    total += 1;
                }
            }
        }
    }
    total
}

fn rendered_orders(svg: &str) -> Vec<HashMap<String, usize>> {
    let mut labels_by_x: Vec<(f64, Vec<(f64, String)>)> = Vec::new();
    let mut idx = 0usize;
    while let Some(pos) = svg[idx..].find("<text ") {
        let start = idx + pos;
        let tag_end = svg[start..]
            .find('>')
            .map(|n| start + n + 1)
            .expect("unterminated text tag");
        let end = svg[tag_end..]
            .find("</text>")
            .map(|n| tag_end + n)
            .expect("unterminated text content");
        let tag = &svg[start..tag_end];
        let content = svg[tag_end..end].replace("&apos;", "'");
        idx = end + "</text>".len();

        if content.starts_with("Game of Thrones Affiliations") {
            continue;
        }
        if !tag.contains("text-anchor=") {
            continue;
        }

        let x = attr_value(tag, "x")
            .and_then(|v| v.parse::<f64>().ok())
            .expect("missing text x");
        let y = attr_value(tag, "y")
            .and_then(|v| v.parse::<f64>().ok())
            .expect("missing text y");

        if let Some((_, labels)) = labels_by_x.iter_mut().find(|(known_x, _)| (known_x - x).abs() < 1.0)
        {
            labels.push((y, content));
        } else {
            labels_by_x.push((x, vec![(y, content)]));
        }
    }

    labels_by_x.sort_by(|a, b| a.0.total_cmp(&b.0));
    labels_by_x
        .into_iter()
        .map(|(_, mut labels)| {
            labels.sort_by(|a, b| a.0.total_cmp(&b.0));
            labels
                .into_iter()
                .enumerate()
                .map(|(rank, (_, label))| (label, rank))
                .collect()
        })
        .collect()
}

fn attr_value<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!("{name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let end = start + tag[start..].find('"')?;
    Some(&tag[start..end])
}
