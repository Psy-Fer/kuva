use std::collections::HashMap;
use std::path::{Path, PathBuf};

use clap::Args;

use kuva::plot::clustermap::{AnnotationPlacement, AnnotationTrack, Clustermap, ClustermapNorm};
use kuva::plot::{ColLabelSide, RowLabelSide};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{parse_colormap, ColSpec, DataTable, HeaderMode, InputArgs, NaSet};
use crate::layout_args::{apply_base_args, BaseArgs};
use crate::output::write_output;

/// Clustered and annotated heatmap from a wide matrix (first column as row labels).
///
/// Rows and columns are clustered by default (UPGMA on Euclidean distance) with dendrograms.
/// Add metadata strips with `--row-annot` / `--col-annot`, which get a labelled legend; use
/// `--categorical` for text cells such as genotype calls.
#[derive(Args, Debug)]
pub struct ClustermapArgs {
    /// Don't cluster the rows (keep input order, no row dendrogram).
    #[arg(long)]
    pub no_cluster_rows: bool,

    /// Don't cluster the columns (keep input order, no column dendrogram).
    #[arg(long)]
    pub no_cluster_cols: bool,

    /// Normalize before colouring: `row` (z-score each row), `col` (z-score each column), or
    /// `none` (default).
    #[arg(long, value_name = "row|col|none")]
    pub normalize: Option<String>,

    /// Color map for numeric cells (default: viridis).
    #[arg(long, default_value = "viridis")]
    pub colormap: String,

    /// Treat cells as categories (text such as `ref`/`het`/`hom`) instead of numbers. Each
    /// category gets a colour and a legend entry; clustering is turned off. NA tokens (`NA`,
    /// `.`, empty) are missing and drawn grey.
    #[arg(long)]
    pub categorical: bool,

    /// Print each cell's value (or category) inside it.
    #[arg(long)]
    pub values: bool,

    /// Title for the colour bar, or for the cell legend with `--categorical`.
    #[arg(long)]
    pub legend: Option<String>,

    /// Row annotation strip from another table: `FILE` or `FILE:COLUMN`. The file's first
    /// column holds row labels (matched by name, not position); COLUMN (name or 0-based index,
    /// default 1) holds the category. Its header becomes the strip's label and legend title.
    /// Repeatable.
    #[arg(long, value_name = "FILE[:COLUMN]")]
    pub row_annot: Vec<String>,

    /// Column annotation strip, as `--row-annot` but keyed by column (sample) labels.
    /// Repeatable.
    #[arg(long, value_name = "FILE[:COLUMN]")]
    pub col_annot: Vec<String>,

    /// Colours for categories, in annotation strips and `--categorical` cells:
    /// `ctrl=#999999,treat=#d55e00`. The categories named here are also listed first in the
    /// legend, in this order; the rest follow in order of appearance.
    #[arg(long, value_name = "CAT=COLOR,...")]
    pub category_colors: Option<String>,

    /// Legend text for categories: `treat=Treated (10 uM),het=Heterozygous`.
    #[arg(long, value_name = "CAT=TEXT,...")]
    pub category_labels: Option<String>,

    /// Draw the column labels above the body instead of below.
    #[arg(long)]
    pub col_labels_top: bool,

    /// Draw the row labels left of the body instead of right.
    #[arg(long)]
    pub row_labels_left: bool,

    /// Slant of the column labels in degrees: 0 flat, 45 (default), 90 vertical.
    #[arg(long, value_name = "DEG")]
    pub col_label_rotation: Option<f64>,

    /// Fixed cell size in pixels, `W` or `WxH`; the canvas grows to fit the table.
    #[arg(long, value_name = "W[xH]")]
    pub cell_size: Option<String>,

    /// Gap between cells in pixels (0 makes them touch).
    #[arg(long, value_name = "PX")]
    pub cell_gap: Option<f64>,

    /// Legend text for missing values in cells and strips (default `NA`), e.g. `No call`.
    #[arg(long, value_name = "TEXT")]
    pub na_label: Option<String>,

    /// Put annotation strips between the labels and the cells instead of outside the labels
    /// (applies with --col-labels-top / --row-labels-left).
    #[arg(long)]
    pub annot_next_to_cells: bool,

    /// Hide the legend groups added for annotation strips.
    #[arg(long)]
    pub no_annot_legend: bool,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
}

/// Parse `k=v,k=v` pairs. Values may contain `=` after the first one; an entry without `=` is
/// an error.
fn parse_pairs(spec: &str, flag: &str) -> Result<Vec<(String, String)>, String> {
    spec.split(',')
        .filter(|p| !p.trim().is_empty())
        .map(|p| {
            p.split_once('=')
                .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
                .ok_or_else(|| format!("{flag}: '{p}' is not CATEGORY=VALUE"))
        })
        .collect()
}

/// Parse `--cell-size`: `12` (square) or `14x10`.
fn parse_cell_size(spec: &str) -> Result<(f64, f64), String> {
    let num = |s: &str| {
        s.trim()
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite() && *v > 0.0)
            .ok_or_else(|| format!("--cell-size: '{spec}' is not W or WxH in pixels"))
    };
    match spec.split_once(['x', 'X']) {
        Some((w, h)) => Ok((num(w)?, num(h)?)),
        None => {
            let v = num(spec)?;
            Ok((v, v))
        }
    }
}

/// Split `FILE[:COLUMN]`. The file part is the longest prefix that names an existing file, so
/// paths with colons and `name:X` column specs both work.
fn split_annot_spec(spec: &str) -> Result<(PathBuf, ColSpec), String> {
    if Path::new(spec).is_file() {
        return Ok((PathBuf::from(spec), ColSpec::Index(1)));
    }
    for (i, _) in spec.match_indices(':') {
        let (file, col) = (&spec[..i], &spec[i + 1..]);
        if Path::new(file).is_file() {
            let col = col.parse::<ColSpec>()?;
            return Ok((PathBuf::from(file), col));
        }
    }
    Err(format!(
        "annotation '{spec}': no such file (expected FILE or FILE:COLUMN)"
    ))
}

/// Read one annotation strip: `(label, value per key)` where `keys` are the row or column
/// labels in data order. Unmatched keys get a missing value and a warning.
fn read_annotation(
    spec: &str,
    keys: &[String],
    delimiter: Option<char>,
    na: &NaSet,
) -> Result<(String, Vec<String>), String> {
    let (path, col) = split_annot_spec(spec)?;
    let table = DataTable::parse(Some(&path), HeaderMode::Auto, delimiter, &[])?;
    let key_col = ColSpec::Index(0);
    let ids = table.col_str(&key_col)?;
    let values = table.col_str(&col)?;
    let label = table.col_display_name(&col);
    let lookup: HashMap<&str, &str> = ids
        .iter()
        .map(|s| s.trim())
        .zip(values.iter().map(String::as_str))
        .collect();
    let mut missing = 0usize;
    let resolved = keys
        .iter()
        .map(|k| match lookup.get(k.trim()) {
            Some(v) if !na.is_na(v) => v.trim().to_string(),
            Some(_) => String::new(),
            None => {
                missing += 1;
                String::new()
            }
        })
        .collect();
    if missing > 0 {
        eprintln_safe!(
            "warning: annotation '{}': {missing} of {} labels not found in {}; shown as NA",
            label,
            keys.len(),
            path.display()
        );
    }
    Ok((label, resolved))
}

pub fn run(args: ClustermapArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.header_mode(),
        args.input.delimiter,
        &[],
    )?;
    if table.rows.is_empty() {
        return Err("clustermap input has no data rows".into());
    }
    // The width comes from the header, or the longest row without one. Shorter
    // rows are padded with missing cells (an empty value is missing); a row
    // longer than the header is an error rather than silently cut.
    let longest = table.rows.iter().map(Vec::len).max().unwrap_or(0);
    let ncols = table.header.as_ref().map_or(longest, Vec::len);
    if let Some(i) = table.rows.iter().position(|r| r.len() > ncols) {
        return Err(format!(
            "data row {} has {} fields but the header has {ncols}",
            i + 1,
            table.rows[i].len()
        ));
    }
    if ncols < 2 {
        return Err("clustermap input needs at least 2 columns (row label + data)".into());
    }
    let nrows = table.rows.len();
    use crate::data::CELL_LIMIT;
    if nrows * (ncols - 1) > CELL_LIMIT {
        return Err(format!(
            "clustermap has {} cells ({nrows} rows x {} data columns), over the {CELL_LIMIT} \
             cell limit. Aggregate or filter the matrix first.",
            nrows * (ncols - 1),
            ncols - 1
        ));
    }
    let row_labels: Vec<String> = table.rows.iter().map(|r| r[0].clone()).collect();
    let col_labels: Vec<String> = match table.header {
        Some(ref hdr) => hdr[1..].to_vec(),
        None => (1..ncols).map(|i| format!("col_{i}")).collect(),
    };
    let na = NaSet::default_tokens();
    let cell = |r: usize, c: usize| -> &str { table.rows[r].get(c + 1).map_or("", String::as_str) };

    let mut cm = Clustermap::new()
        .with_row_labels(row_labels.clone())
        .with_col_labels(col_labels.clone());

    if args.categorical {
        let cats: Vec<Vec<String>> = (0..nrows)
            .map(|r| {
                (0..ncols - 1)
                    .map(|c| {
                        let v = cell(r, c);
                        if na.is_na(v) {
                            String::new()
                        } else {
                            v.trim().to_string()
                        }
                    })
                    .collect()
            })
            .collect();
        cm = cm.with_categorical_data(cats);
    } else {
        let matrix: Vec<Vec<f64>> = (0..nrows)
            .map(|r| {
                (0..ncols - 1)
                    .map(|c| {
                        let v = cell(r, c);
                        v.trim()
                            .parse::<f64>()
                            .ok()
                            .filter(|x| x.is_finite())
                            .ok_or_else(|| {
                                format!(
                                    "row {} ('{}'), column '{}': '{}' is not a number \
                                 (use --categorical for text cells)",
                                    r + 1,
                                    row_labels[r],
                                    col_labels[c],
                                    v
                                )
                            })
                    })
                    .collect::<Result<Vec<f64>, String>>()
            })
            .collect::<Result<_, _>>()?;
        cm = cm
            .with_data(matrix)
            .with_color_map(parse_colormap(&args.colormap));
        cm = cm.with_normalization(
            match args.normalize.as_deref().map(str::to_ascii_lowercase) {
                None => ClustermapNorm::None,
                Some(n) if n == "none" => ClustermapNorm::None,
                Some(n) if n == "row" => ClustermapNorm::RowZScore,
                Some(n) if n == "col" || n == "column" => ClustermapNorm::ColZScore,
                Some(other) => {
                    return Err(format!("--normalize: '{other}' is not row, col or none"));
                }
            },
        );
        cm = cm
            .with_cluster_rows(!args.no_cluster_rows)
            .with_cluster_cols(!args.no_cluster_cols);
    }

    let colors = match args.category_colors {
        Some(ref s) => parse_pairs(s, "--category-colors")?,
        None => Vec::new(),
    };
    let labels = match args.category_labels {
        Some(ref s) => parse_pairs(s, "--category-labels")?,
        None => Vec::new(),
    };
    let order: Vec<String> = colors.iter().map(|(k, _)| k.clone()).collect();
    let style_track = |mut t: AnnotationTrack| {
        t = t.with_category_order(order.clone());
        if let Some(ref na) = args.na_label {
            t = t.with_na_label(na.clone());
        }
        for (k, v) in &colors {
            t = t.with_category_color(k.clone(), v.clone());
        }
        for (k, v) in &labels {
            t = t.with_category_label(k.clone(), v.clone());
        }
        t
    };
    for spec in &args.row_annot {
        let (label, values) = read_annotation(spec, &row_labels, args.input.delimiter, &na)?;
        cm = cm.with_row_annotation(style_track(
            AnnotationTrack::categorical(values).with_label(label),
        ));
    }
    for spec in &args.col_annot {
        let (label, values) = read_annotation(spec, &col_labels, args.input.delimiter, &na)?;
        cm = cm.with_col_annotation(style_track(
            AnnotationTrack::categorical(values).with_label(label),
        ));
    }
    if args.categorical {
        cm = cm
            .with_category_colors(colors.clone())
            .with_category_labels(labels.clone())
            .with_category_order(order.clone());
    }

    if let Some(ref na) = args.na_label {
        cm = cm.with_na_label(na.clone());
    }
    if args.values {
        cm = cm.with_values();
    }
    if let Some(ref label) = args.legend {
        cm = cm.with_legend(label.clone());
    }
    if args.col_labels_top {
        cm = cm.with_col_labels_position(ColLabelSide::Top);
    }
    if args.row_labels_left {
        cm = cm.with_row_labels_position(RowLabelSide::Left);
    }
    if let Some(deg) = args.col_label_rotation {
        cm = cm.with_col_label_rotation(deg);
    }
    if let Some(ref spec) = args.cell_size {
        let (w, h) = parse_cell_size(spec)?;
        cm = cm.with_cell_size(w, h);
    }
    if let Some(gap) = args.cell_gap {
        cm = cm.with_cell_gap(gap);
    }
    if args.annot_next_to_cells {
        cm = cm.with_annotation_placement(AnnotationPlacement::NextToCells);
    }
    if args.no_annot_legend {
        cm = cm.with_annotation_legend(false);
    }

    #[cfg(feature = "emit_code")]
    if args.base.emit_code {
        print!(
            "{}",
            crate::emit_code::assemble(
                &crate::emit_code::clustermap_uses(&cm),
                "Clustermap",
                &[crate::emit_code::emit_clustermap_plot(&cm)],
                &args.base,
                None,
                None,
            )
        );
        return Ok(());
    }

    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    write_output(render_multiple(plots, layout), &args.base)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairs_and_cell_size_parse() {
        assert_eq!(
            parse_pairs("a=#fff, b = x=y ,", "--f").unwrap(),
            vec![("a".into(), "#fff".into()), ("b".into(), "x=y".into())]
        );
        assert!(parse_pairs("nope", "--f").is_err());
        assert_eq!(parse_cell_size("12").unwrap(), (12.0, 12.0));
        assert_eq!(parse_cell_size("14x10").unwrap(), (14.0, 10.0));
        assert!(parse_cell_size("0").is_err());
        assert!(parse_cell_size("ax3").is_err());
    }
}
