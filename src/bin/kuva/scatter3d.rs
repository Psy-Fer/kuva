use clap::Args;

use kuva::plot::scatter3d::Scatter3DPlot;
use kuva::plot::heatmap::ColorMap;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use kuva::render::palette::Palette;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, apply_base_args};
use crate::output::write_output;

use crate::data::parse_colormap;

/// 3D scatter plot with orthographic projection.
#[derive(Args, Debug)]
pub struct Scatter3DArgs {
    /// Column for X values (0-based index or header name).
    #[arg(long, default_value = "0")]
    pub x: ColSpec,

    /// Column for Y values (0-based index or header name).
    #[arg(long, default_value = "1")]
    pub y: ColSpec,

    /// Column for Z values (0-based index or header name).
    #[arg(long, default_value = "2")]
    pub z: ColSpec,

    /// Group by this column — one color per unique value.
    #[arg(long)]
    pub color_by: Option<ColSpec>,

    /// Point color (CSS color string).
    #[arg(long)]
    pub color: Option<String>,

    /// Point radius in pixels.
    #[arg(long)]
    pub size: Option<f64>,

    /// Azimuth viewing angle in degrees.
    #[arg(long, default_value_t = -60.0, allow_hyphen_values = true)]
    pub azimuth: f64,

    /// Elevation viewing angle in degrees.
    #[arg(long, default_value_t = 30.0, allow_hyphen_values = true)]
    pub elevation: f64,

    /// X-axis label.
    #[arg(long)]
    pub x_label: Option<String>,

    /// Y-axis label.
    #[arg(long)]
    pub y_label: Option<String>,

    /// Z-axis label.
    #[arg(long)]
    pub z_label: Option<String>,

    /// Colormap for z-coloring: viridis, inferno, grayscale.
    /// Takes precedence over --color-by group colors when both are set.
    #[arg(long)]
    pub z_color: Option<String>,

    /// Fade distant points for depth cue.
    #[arg(long)]
    pub depth_shade: bool,

    /// Place Z-axis on the left side instead of the right.
    #[arg(long)]
    pub z_axis_left: bool,

    /// Hide grid lines on back walls.
    #[arg(long)]
    pub no_grid: bool,

    /// Hide wireframe bounding box.
    #[arg(long)]
    pub no_box: bool,

    /// Number of grid/tick divisions per axis (default: 5).
    #[arg(long)]
    pub grid_lines: Option<usize>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
}

/// Apply shared optional args to a plot builder.
fn apply_options(mut plot: Scatter3DPlot, args: &Scatter3DArgs, z_cmap: &Option<ColorMap>) -> Scatter3DPlot {
    if let Some(ref c) = args.color {
        plot = plot.with_color(c.clone());
    }
    if let Some(s) = args.size {
        plot = plot.with_size(s);
    }
    if let Some(ref xl) = args.x_label {
        plot = plot.with_x_label(xl.clone());
    }
    if let Some(ref yl) = args.y_label {
        plot = plot.with_y_label(yl.clone());
    }
    if let Some(ref zl) = args.z_label {
        plot = plot.with_z_label(zl.clone());
    }
    if let Some(ref cm) = z_cmap {
        plot = plot.with_z_colormap(cm.clone());
    }
    if args.no_grid { plot = plot.with_no_grid(); }
    if args.no_box { plot = plot.with_no_box(); }
    if let Some(n) = args.grid_lines { plot = plot.with_grid_lines(n); }
    plot
}

pub fn run(args: Scatter3DArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let z_cmap = args.z_color.as_deref().map(parse_colormap);

    if let Some(ref cb) = args.color_by {
        // Merge all groups into a single plot with per-point colors so they
        // share one coordinate system and one set of axes.
        let pal = Palette::category10();
        let groups = table.group_by(cb)?;
        let mut all_data: Vec<(f64, f64, f64)> = Vec::new();
        let mut all_colors: Vec<String> = Vec::new();
        let mut group_names: Vec<String> = Vec::new();

        for (i, (name, subtable)) in groups.into_iter().enumerate() {
            group_names.push(name);
            let x_vals = subtable.col_f64(&args.x)?;
            let y_vals = subtable.col_f64(&args.y)?;
            let z_vals = subtable.col_f64(&args.z)?;
            let color = pal[i % pal.len()].to_string();
            for ((x, y), z) in x_vals.into_iter().zip(y_vals).zip(z_vals) {
                all_data.push((x, y, z));
                all_colors.push(color.clone());
            }
        }

        let mut plot = Scatter3DPlot::new()
            .with_data(all_data)
            .with_colors(all_colors)
            .with_azimuth(args.azimuth)
            .with_elevation(args.elevation);
        if args.z_axis_left { plot = plot.with_z_axis_right(false); }
        if args.depth_shade { plot = plot.with_depth_shade(); }
        plot = apply_options(plot, &args, &z_cmap);

        let plots = vec![Plot::Scatter3D(plot)];
        let mut layout = Layout::auto_from_plots(&plots);

        // Build legend entries from group names
        let entries: Vec<kuva::plot::legend::LegendEntry> = group_names.iter().enumerate().map(|(i, name)| {
            kuva::plot::legend::LegendEntry {
                label: name.clone(),
                color: pal[i % pal.len()].to_string(),
                shape: kuva::plot::legend::LegendShape::Circle,
                dasharray: None,
            }
        }).collect();
        if !entries.is_empty() {
            let max_len = entries.iter().map(|e| e.label.len()).max().unwrap_or(0);
            layout.show_legend = true;
            layout.legend_width = (max_len as f64 * 8.5 + 35.0).max(80.0);
            layout.legend_entries = Some(entries);
        }

        let layout = apply_base_args(layout, &args.base);
        let scene = render_multiple(plots, layout);
        write_output(scene, &args.base)
    } else {
        let x_vals = table.col_f64(&args.x)?;
        let y_vals = table.col_f64(&args.y)?;
        let z_vals = table.col_f64(&args.z)?;

        let data: Vec<(f64, f64, f64)> = x_vals.into_iter()
            .zip(y_vals)
            .zip(z_vals)
            .map(|((x, y), z)| (x, y, z))
            .collect();

        let mut plot = Scatter3DPlot::new()
            .with_data(data)
            .with_azimuth(args.azimuth)
            .with_elevation(args.elevation);
        if args.z_axis_left { plot = plot.with_z_axis_right(false); }
        if args.depth_shade { plot = plot.with_depth_shade(); }

        let plots = vec![Plot::Scatter3D(apply_options(plot, &args, &z_cmap))];
        let layout = Layout::auto_from_plots(&plots);
        let layout = apply_base_args(layout, &args.base);
        let scene = render_multiple(plots, layout);
        write_output(scene, &args.base)
    }
}
