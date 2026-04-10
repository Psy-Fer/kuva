use clap::Args;

use kuva::plot::surface3d::Surface3DPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, apply_base_args};
use crate::output::write_output;

use crate::data::parse_colormap;

/// 3D surface plot with orthographic projection.
#[derive(Args, Debug)]
pub struct Surface3DArgs {
    /// Read input as a matrix of Z values (one row per line).
    #[arg(long)]
    pub matrix: bool,

    /// Column for X values (long-format mode).
    #[arg(long, default_value = "0")]
    pub x: ColSpec,

    /// Column for Y values (long-format mode).
    #[arg(long, default_value = "1")]
    pub y: ColSpec,

    /// Column for Z values (long-format mode).
    #[arg(long, default_value = "2")]
    pub z: ColSpec,

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

    /// Colormap: viridis, inferno, grayscale.
    #[arg(long)]
    pub z_color: Option<String>,

    /// Surface opacity (0.0–1.0).
    #[arg(long)]
    pub alpha: Option<f64>,

    /// Disable wireframe edges on the mesh.
    #[arg(long)]
    pub no_wireframe: bool,

    /// Uniform surface color (when no colormap).
    #[arg(long)]
    pub color: Option<String>,

    /// Upsample the grid to NxN resolution (bilinear interpolation).
    #[arg(long)]
    pub resolution: Option<usize>,

    /// Place Z-axis on the left side.
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

pub fn run(args: Surface3DArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let mut plot = if args.matrix {
        // Matrix mode: each row of the table is a row of Z values.
        // All columns are treated as numeric.
        if table.rows.is_empty() { return Err("empty matrix input".into()); }
        let mut z_data = Vec::with_capacity(table.rows.len());
        for row in &table.rows {
            let z_row: Result<Vec<f64>, String> = row.iter().map(|s| {
                s.trim().parse::<f64>().map_err(|_| format!("not a number: {s}"))
            }).collect();
            z_data.push(z_row?);
        }
        Surface3DPlot::new(z_data)
    } else {
        // Long format: x, y, z columns → pivot into grid
        let x_vals = table.col_f64(&args.x)?;
        let y_vals = table.col_f64(&args.y)?;
        let z_vals = table.col_f64(&args.z)?;

        // Find unique sorted x and y values
        let mut xs: Vec<f64> = x_vals.clone();
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        xs.dedup_by(|a, b| (*a - *b).abs() < 1e-12);
        let mut ys: Vec<f64> = y_vals.clone();
        ys.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        ys.dedup_by(|a, b| (*a - *b).abs() < 1e-12);

        let ncols = xs.len();
        let nrows = ys.len();
        let mut z_data = vec![vec![f64::NAN; ncols]; nrows];

        for k in 0..x_vals.len() {
            let xi = xs.partition_point(|&v| v < x_vals[k] - 1e-12);
            if xi >= ncols || (xs[xi] - x_vals[k]).abs() > 1e-12 {
                return Err("grid pivot: x value not found in sorted unique list".into());
            }
            let yi = ys.partition_point(|&v| v < y_vals[k] - 1e-12);
            if yi >= nrows || (ys[yi] - y_vals[k]).abs() > 1e-12 {
                return Err("grid pivot: y value not found in sorted unique list".into());
            }
            z_data[yi][xi] = z_vals[k];
        }

        Surface3DPlot::new(z_data)
            .with_x_coords(xs)
            .with_y_coords(ys)
    };

    // Upsample grid with bilinear interpolation if --resolution is set
    if let Some(res) = args.resolution {
        let res = res.clamp(2, 1000);
        let old_z = &plot.z_data;
        let old_nrows = old_z.len();
        let old_ncols = old_z.first().map_or(0, |r| r.len());
        if old_nrows >= 2 && old_ncols >= 2 {
            let old_xs: Vec<f64> = (0..old_ncols).map(|j| plot.x_at(j)).collect();
            let old_ys: Vec<f64> = (0..old_nrows).map(|i| plot.y_at(i)).collect();
            let new_xs: Vec<f64> = (0..res).map(|j| {
                old_xs[0] + (old_xs[old_ncols-1] - old_xs[0]) * j as f64 / (res-1) as f64
            }).collect();
            let new_ys: Vec<f64> = (0..res).map(|i| {
                old_ys[0] + (old_ys[old_nrows-1] - old_ys[0]) * i as f64 / (res-1) as f64
            }).collect();
            let mut new_z = vec![vec![0.0_f64; res]; res];
            let mut yi_carry = 0usize;
            for (ri, &ny) in new_ys.iter().enumerate() {
                // Carry forward: new_ys is monotonic, so yi only advances
                while yi_carry + 1 < old_nrows - 1 && old_ys[yi_carry + 1] < ny { yi_carry += 1; }
                let yi = yi_carry;
                let yt = if (old_ys[yi + 1] - old_ys[yi]).abs() < 1e-15 { 0.0 }
                    else { (ny - old_ys[yi]) / (old_ys[yi + 1] - old_ys[yi]) };
                let mut xi_carry = 0usize;
                for (ci, &nx) in new_xs.iter().enumerate() {
                    while xi_carry + 1 < old_ncols - 1 && old_xs[xi_carry + 1] < nx { xi_carry += 1; }
                    let xi = xi_carry;
                    let xt = if (old_xs[xi + 1] - old_xs[xi]).abs() < 1e-15 { 0.0 }
                        else { (nx - old_xs[xi]) / (old_xs[xi + 1] - old_xs[xi]) };
                    let z00 = old_z[yi][xi];
                    let z10 = old_z[yi][xi + 1];
                    let z01 = old_z[yi + 1][xi];
                    let z11 = old_z[yi + 1][xi + 1];
                    new_z[ri][ci] = z00 * (1.0-xt) * (1.0-yt)
                        + z10 * xt * (1.0-yt)
                        + z01 * (1.0-xt) * yt
                        + z11 * xt * yt;
                }
            }
            plot = plot.with_z_data(new_z).with_x_coords(new_xs).with_y_coords(new_ys);
        }
    }

    plot = plot
        .with_azimuth(args.azimuth)
        .with_elevation(args.elevation);
    if args.z_axis_left { plot = plot.with_z_axis_right(false); }
    if args.no_wireframe { plot = plot.with_no_wireframe(); }

    if let Some(ref c) = args.color { plot = plot.with_color(c.clone()); }
    if let Some(a) = args.alpha { plot = plot.with_alpha(a); }
    if let Some(ref xl) = args.x_label { plot = plot.with_x_label(xl.clone()); }
    if let Some(ref yl) = args.y_label { plot = plot.with_y_label(yl.clone()); }
    if let Some(ref zl) = args.z_label { plot = plot.with_z_label(zl.clone()); }
    if let Some(ref name) = args.z_color { plot = plot.with_z_colormap(parse_colormap(name)); }
    if args.no_grid { plot = plot.with_no_grid(); }
    if args.no_box { plot = plot.with_no_box(); }
    if let Some(n) = args.grid_lines { plot = plot.with_grid_lines(n); }

    let plots = vec![Plot::Surface3D(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
