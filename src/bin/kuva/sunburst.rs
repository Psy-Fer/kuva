use clap::Args;

use kuva::plot::sunburst::{SunburstPlot, SunburstColorMode};
use kuva::plot::treemap::TreemapNode;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs, parse_colormap};
use crate::layout_args::{BaseArgs, apply_base_args};
use crate::output::write_output;

/// Sunburst chart — radial hierarchy where arc widths encode proportional values.
#[derive(Args, Debug)]
pub struct SunburstArgs {
    /// Label column (name or 0-based index; default: 0).
    #[arg(long)]
    pub label: Option<ColSpec>,

    /// Value column (name or 0-based index; default: 1).
    #[arg(long)]
    pub value: Option<ColSpec>,

    /// Optional parent column for a two-level hierarchy.
    /// Rows with the same parent value are grouped under that parent.
    #[arg(long)]
    pub parent: Option<ColSpec>,

    /// Column for per-leaf color values in `value` mode, or explicit CSS colors in `explicit` mode.
    #[arg(long)]
    pub color_col: Option<ColSpec>,

    /// Color mode: parent (default), value, explicit.
    #[arg(long, default_value = "parent", value_enum)]
    pub color_by: CliColorBy,

    /// Color map used with `--color-by value` (default: viridis). Run `kuva sunburst --help` for accepted names.
    #[arg(long)]
    pub colormap: Option<String>,

    /// Fractional inner radius for donut-style chart (0.0 = solid disc, e.g. 0.3 = 30% hole).
    #[arg(long)]
    pub inner_radius: Option<f64>,

    /// Starting angle in degrees (0 = top/north, clockwise; default: 0).
    #[arg(long)]
    pub start_angle: Option<f64>,

    /// Gap in pixels between adjacent rings (default: 1.0).
    #[arg(long)]
    pub ring_gap: Option<f64>,

    /// Minimum arc angle in degrees for a label to be shown (default: 15.0).
    #[arg(long)]
    pub min_label_angle: Option<f64>,

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


pub fn run(args: SunburstArgs) -> Result<(), String> {
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
        return Err("sunburst: input has no data".into());
    }

    let cmap = parse_colormap(args.colormap.as_deref().unwrap_or("viridis"));

    let mut plot = SunburstPlot::new();

    if let Some(r) = args.inner_radius {
        plot = plot.with_inner_radius(r);
    }
    if let Some(a) = args.start_angle {
        plot = plot.with_start_angle(a);
    }
    if let Some(g) = args.ring_gap {
        plot = plot.with_ring_gap(g);
    }
    if let Some(mla) = args.min_label_angle {
        plot = plot.with_min_label_angle(mla);
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
        // Flat: each row is a top-level leaf (all in root ring)
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
                SunburstColorMode::Explicit
            }
            CliColorBy::Value => {
                for (lbl, val) in labels.iter().zip(values.iter()) {
                    plot = plot.with_node(TreemapNode::leaf(lbl.clone(), *val));
                }
                if let Some(ref cc) = args.color_col {
                    let color_vals = table.col_f64(cc)?;
                    plot = plot.with_color_values(color_vals);
                }
                SunburstColorMode::ByValue(cmap)
            }
            CliColorBy::Parent => {
                // Flat mode: wrap all leaves under an invisible root to get category colors
                for (lbl, val) in labels.iter().zip(values.iter()) {
                    plot = plot.with_node(TreemapNode::leaf(lbl.clone(), *val));
                }
                SunburstColorMode::ByParent
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

    let plots = vec![Plot::Sunburst(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
