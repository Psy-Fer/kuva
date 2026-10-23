use clap::Args;

use kuva::plot::Quiver3DPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{parse_colormap, ColSpec, DataTable, InputArgs};
use crate::layout_args::{apply_base_args, BaseArgs};
use crate::output::write_output;
use crate::quiver::CliPivot;

/// 3D quiver plot — vector field rendered as arrows in an orthographic 3D box.
#[derive(Args, Debug)]
pub struct Quiver3DArgs {
    /// Anchor X column (0-based index or header name; default: 0).
    #[arg(long)]
    pub x_col: Option<ColSpec>,

    /// Anchor Y column (0-based index or header name; default: 1).
    #[arg(long)]
    pub y_col: Option<ColSpec>,

    /// Anchor Z column (0-based index or header name; default: 2).
    #[arg(long)]
    pub z_col: Option<ColSpec>,

    /// Vector U (x-component) column (0-based index or header name; default: 3).
    #[arg(long)]
    pub u_col: Option<ColSpec>,

    /// Vector V (y-component) column (0-based index or header name; default: 4).
    #[arg(long)]
    pub v_col: Option<ColSpec>,

    /// Vector W (z-component) column (0-based index or header name; default: 5).
    #[arg(long)]
    pub w_col: Option<ColSpec>,

    /// Arrow color (CSS color string).
    #[arg(long)]
    pub color: Option<String>,

    /// Scale multiplier applied to `(u, v, w)` before projection (disables auto-scaling).
    #[arg(long)]
    pub arrow_scale: Option<f64>,

    /// Fraction of a grid cell spanned by the longest arrow when auto-scaling.
    /// Default `0.9`. Mutually exclusive with `--arrow-scale`.
    #[arg(long, conflicts_with = "arrow_scale")]
    pub auto_scale: Option<f64>,

    /// Shaft width in pixels.
    #[arg(long)]
    pub shaft_width: Option<f64>,

    /// Arrow head length in pixels (default: proportional to the projected shaft).
    #[arg(long)]
    pub head_length: Option<f64>,

    /// Arrow head half-width in pixels.
    #[arg(long)]
    pub head_width: Option<f64>,

    /// Colormap name — arrows are colored by magnitude: viridis, inferno, grayscale.
    #[arg(long)]
    pub colormap: Option<String>,

    /// Colorbar label (only applies when `--colormap` is set).
    #[arg(long)]
    pub colorbar_label: Option<String>,

    /// Legend label for the series.
    #[arg(long)]
    pub legend: Option<String>,

    /// Where (x, y, z) sits on each arrow: `tail` (default), `middle`, or `tip`.
    #[arg(long, value_enum)]
    pub pivot: Option<CliPivot>,

    /// Fade distant arrows for a depth cue.
    #[arg(long)]
    pub depth_shade: bool,

    /// Size the 3D box from the arrow anchors only (outer arrows extend past the box faces).
    #[arg(long)]
    pub tight_bounds: bool,

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

pub fn run(args: Quiver3DArgs) -> Result<(), String> {
    let x_col = args.x_col.clone().unwrap_or(ColSpec::Index(0));
    let y_col = args.y_col.clone().unwrap_or(ColSpec::Index(1));
    let z_col = args.z_col.clone().unwrap_or(ColSpec::Index(2));
    let u_col = args.u_col.clone().unwrap_or(ColSpec::Index(3));
    let v_col = args.v_col.clone().unwrap_or(ColSpec::Index(4));
    let w_col = args.w_col.clone().unwrap_or(ColSpec::Index(5));
    let proj = [&x_col, &y_col, &z_col, &u_col, &v_col, &w_col]
        .into_iter()
        .cloned()
        .collect::<Vec<ColSpec>>();
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.header_mode(),
        args.input.delimiter,
        &proj,
    )?;

    let xs = table.col_f64(&x_col)?;
    let ys = table.col_f64(&y_col)?;
    let zs = table.col_f64(&z_col)?;
    let us = table.col_f64(&u_col)?;
    let vs = table.col_f64(&v_col)?;
    let ws = table.col_f64(&w_col)?;

    let n = xs.len();
    if [ys.len(), zs.len(), us.len(), vs.len(), ws.len()]
        .iter()
        .any(|&len| len != n)
    {
        return Err(format!(
            "column length mismatch: x={}, y={}, z={}, u={}, v={}, w={}",
            n,
            ys.len(),
            zs.len(),
            us.len(),
            vs.len(),
            ws.len()
        ));
    }

    let mut plot = Quiver3DPlot::new()
        .with_arrows(
            xs.iter()
                .zip(&ys)
                .zip(&zs)
                .zip(&us)
                .zip(&vs)
                .zip(&ws)
                .map(|(((((x, y), z), u), v), w)| (*x, *y, *z, *u, *v, *w)),
        )
        .with_azimuth(args.azimuth)
        .with_elevation(args.elevation);

    if let Some(c) = args.color {
        plot = plot.with_color(c);
    }
    if let Some(s) = args.arrow_scale {
        plot = plot.with_scale(s);
    }
    if let Some(f) = args.auto_scale {
        plot = plot.with_auto_scale(f);
    }
    if let Some(w) = args.shaft_width {
        plot = plot.with_shaft_width(w);
    }
    if let Some(l) = args.head_length {
        plot = plot.with_head_length(l);
    }
    if let Some(w) = args.head_width {
        plot = plot.with_head_width(w);
    }
    if let Some(name) = args.colormap {
        plot = plot.with_color_map(parse_colormap(&name));
    }
    if let Some(label) = args.colorbar_label {
        plot = plot.with_color_legend_label(label);
    }
    if let Some(s) = args.legend {
        plot = plot.with_legend(s);
    }
    if let Some(p) = args.pivot {
        plot = plot.with_pivot(p.into());
    }
    if args.depth_shade {
        plot = plot.with_depth_shade();
    }
    if args.tight_bounds {
        plot = plot.with_tight_bounds();
    }
    if let Some(xl) = args.x_label {
        plot = plot.with_x_label(xl);
    }
    if let Some(yl) = args.y_label {
        plot = plot.with_y_label(yl);
    }
    if let Some(zl) = args.z_label {
        plot = plot.with_z_label(zl);
    }
    if args.z_axis_left {
        plot = plot.with_z_axis_right(false);
    }
    if args.no_grid {
        plot = plot.with_no_grid();
    }
    if args.no_box {
        plot = plot.with_no_box();
    }
    if let Some(n) = args.grid_lines {
        plot = plot.with_grid_lines(n);
    }

    #[cfg(feature = "emit_code")]
    if args.base.emit_code {
        print!(
            "{}",
            crate::emit_code::assemble(
                &[
                    "kuva::plot::Quiver3DPlot",
                    "kuva::plot::QuiverPivot",
                    "kuva::plot::ColorMap",
                ],
                "Quiver3D",
                &[crate::emit_code::emit_quiver3d_plot(&plot)],
                &args.base,
                None,
                None,
            )
        );
        return Ok(());
    }

    let plots = vec![Plot::Quiver3D(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
