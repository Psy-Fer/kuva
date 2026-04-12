use clap::Args;

use kuva::plot::treemap::{TreemapPlot, TreemapNode, TreemapColorMode, TreemapLayout, ColorMap};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, apply_base_args};
use crate::output::write_output;

/// Treemap — tile a rectangle proportionally to values, with optional hierarchical grouping.
#[derive(Args, Debug)]
pub struct TreemapArgs {
    /// Label column (name or 0-based index; default: 0).
    #[arg(long)]
    pub label: Option<ColSpec>,

    /// Value column (name or 0-based index; default: 1).
    #[arg(long)]
    pub value: Option<ColSpec>,

    /// Optional parent column for a two-level hierarchy.
    /// Rows with the same parent value are grouped under that parent node.
    #[arg(long)]
    pub parent: Option<ColSpec>,

    /// Column for per-leaf color in `value` mode, or explicit CSS color strings in `explicit` mode.
    #[arg(long)]
    pub color_col: Option<ColSpec>,

    /// Color mode: parent (default), value, explicit.
    #[arg(long, default_value = "parent", value_enum)]
    pub color_by: CliColorBy,

    /// Color map (used with `--color-by value`): viridis (default), inferno, turbo, grayscale.
    #[arg(long, value_enum)]
    pub colormap: Option<CliColorMap>,

    /// Layout algorithm: squarify (default), slicedice, binary.
    #[arg(long, value_enum)]
    pub layout: Option<CliLayout>,

    /// Padding in pixels between parent border and children (default: 4.0).
    #[arg(long)]
    pub padding: Option<f64>,

    /// Show colorbar in value mode.
    #[arg(long)]
    pub colorbar: bool,

    /// Colorbar label.
    #[arg(long)]
    pub colorbar_label: Option<String>,

    /// Suppress SVG hover tooltips.
    #[arg(long)]
    pub no_tooltips: bool,

    /// Maximum depth to render (0 = root only).
    #[arg(long)]
    pub max_depth: Option<usize>,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub input: InputArgs,
}

#[derive(clap::ValueEnum, Clone, Default, Debug)]
pub enum CliColorBy {
    #[default]
    Parent,
    Value,
    Explicit,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum CliColorMap {
    Viridis,
    Inferno,
    Turbo,
    Grayscale,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum CliLayout {
    Squarify,
    Slicedice,
    Binary,
}

fn cli_to_color_map(c: Option<&CliColorMap>) -> ColorMap {
    match c {
        Some(CliColorMap::Inferno)   => ColorMap::Inferno,
        Some(CliColorMap::Turbo)     => ColorMap::Turbo,
        Some(CliColorMap::Grayscale) => ColorMap::Grayscale,
        _                            => ColorMap::Viridis,
    }
}

fn cli_to_layout(c: Option<&CliLayout>) -> TreemapLayout {
    match c {
        Some(CliLayout::Slicedice) => TreemapLayout::SliceDice,
        Some(CliLayout::Binary)    => TreemapLayout::Binary,
        _                          => TreemapLayout::Squarify,
    }
}

pub fn run(args: TreemapArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let label_col = args.label.unwrap_or(ColSpec::Index(0));
    let value_col = args.value.unwrap_or(ColSpec::Index(1));

    let labels = table.col_str(&label_col)?;
    let values = table.col_f64(&value_col)?;

    if labels.is_empty() {
        return Err("treemap: input has no data".into());
    }

    let cmap = cli_to_color_map(args.colormap.as_ref());
    let layout_algo = cli_to_layout(args.layout.as_ref());

    let mut plot = TreemapPlot::new().with_layout(layout_algo);

    if let Some(pad) = args.padding {
        plot = plot.with_padding(pad);
    }
    if let Some(md) = args.max_depth {
        plot = plot.with_max_depth(md);
    }
    if args.no_tooltips {
        plot = plot.with_tooltips(false);
    }

    // ── Build tree structure ──────────────────────────────────────────────────

    if let Some(ref parent_col) = args.parent {
        // Two-level: group rows by parent
        let parents = table.col_str(parent_col)?;
        use std::collections::BTreeMap;
        let mut groups: BTreeMap<String, Vec<(String, f64)>> = BTreeMap::new();
        for ((lbl, val), parent) in labels.iter().zip(values.iter()).zip(parents.iter()) {
            groups.entry(parent.clone()).or_default().push((lbl.clone(), *val));
        }
        for (parent_name, children) in groups {
            let child_nodes: Vec<TreemapNode> = children.into_iter()
                .map(|(lbl, val)| TreemapNode::leaf(lbl, val))
                .collect();
            plot = plot.with_children(parent_name, child_nodes);
        }
    } else {
        // Flat: each row is a top-level leaf
        let color_mode = match args.color_by {
            CliColorBy::Explicit => {
                if let Some(ref cc) = args.color_col {
                    let colors = table.col_str(cc)?;
                    for ((lbl, val), color) in labels.iter().zip(values.iter()).zip(colors.iter()) {
                        plot = plot.with_node(TreemapNode::leaf_colored(lbl.clone(), *val, color.clone()));
                    }
                } else {
                    for (lbl, val) in labels.iter().zip(values.iter()) {
                        plot = plot.with_node(TreemapNode::leaf(lbl.clone(), *val));
                    }
                }
                TreemapColorMode::Explicit
            }
            CliColorBy::Value => {
                for (lbl, val) in labels.iter().zip(values.iter()) {
                    plot = plot.with_node(TreemapNode::leaf(lbl.clone(), *val));
                }
                if let Some(ref cc) = args.color_col {
                    let color_vals = table.col_f64(cc)?;
                    plot = plot.with_color_values(color_vals);
                }
                TreemapColorMode::ByValue(cmap)
            }
            CliColorBy::Parent => {
                for (lbl, val) in labels.iter().zip(values.iter()) {
                    plot = plot.with_node(TreemapNode::leaf(lbl.clone(), *val));
                }
                TreemapColorMode::ByParent
            }
        };
        plot = plot.with_color_mode(color_mode);
    }

    // ── Colorbar ──────────────────────────────────────────────────────────────
    if args.colorbar {
        plot = plot.with_colorbar(true);
    }
    if let Some(ref lbl) = args.colorbar_label {
        plot = plot.with_colorbar_label(lbl.clone());
    }

    let plots = vec![Plot::Treemap(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
