use std::collections::BTreeMap;

use clap::Args;

use kuva::plot::bar::BarPlot;
use kuva::render::layout::Layout;
use kuva::render::palette::Palette;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{apply_axis_args, apply_base_args, AxisArgs, BaseArgs};
use crate::output::write_output;

/// Bar chart from label and value columns.
#[derive(Args, Debug)]
pub struct BarArgs {
    /// Label column (0-based index or header name; default: 0).
    #[arg(long)]
    pub label_col: Option<ColSpec>,

    /// Value column (0-based index or header name; default: 1).
    #[arg(long)]
    pub value_col: Option<ColSpec>,

    /// Count occurrences of each unique value in this column (ignores --value-col).
    #[arg(long)]
    pub count_by: Option<ColSpec>,

    /// Aggregate --value-col by --label-col using this function: mean, median, sum, min, max.
    #[arg(long, value_name = "FUNC")]
    pub agg: Option<String>,

    /// Bar fill color (CSS string; default: "steelblue").
    #[arg(long)]
    pub color: Option<String>,

    /// Bar width as a fraction of the slot (default: 0.8).
    #[arg(long)]
    pub bar_width: Option<f64>,

    /// Value column(s). Comma-separated list for a wide-format grouped bar chart:
    /// `--y A,B,C` treats each column as a series and each row as a category.
    /// Overrides --value-col when provided with 2+ columns.
    #[arg(long, value_delimiter = ',')]
    pub y: Vec<ColSpec>,

    /// Group by this column and color each series separately (creates a grouped bar chart).
    #[arg(long)]
    pub color_by: Option<ColSpec>,

    /// Render categories on the Y-axis and values on the X-axis.
    #[arg(long)]
    pub horizontal: bool,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
}

pub fn run(args: BarArgs) -> Result<(), String> {
    // Multi-column --y mode: wide-format grouped bar (rows = categories, columns = series)
    if args.y.len() > 1 && args.color_by.is_none() {
        let label_spec = args.label_col.clone().unwrap_or(ColSpec::Index(0));
        let proj: Vec<ColSpec> = std::iter::once(label_spec.clone())
            .chain(args.y.iter().cloned())
            .collect();
        let table = DataTable::parse(
            args.input.input.as_deref(),
            args.input.no_header,
            args.input.delimiter,
            &proj,
        )?;
        // Aggregate each y-column by label (mean, or --agg func if provided).
        // This handles both pre-aggregated wide data (one row per label) and
        // long-format data (many rows per label, values averaged per group).
        let raw_labels = table.col_str(&label_spec)?;
        let y_data: Vec<Vec<f64>> = args
            .y
            .iter()
            .map(|c| table.col_f64(c))
            .collect::<Result<_, _>>()?;
        let series_names: Vec<String> = args
            .y
            .iter()
            .map(|c| table.col_display_name(c))
            .collect();

        // Collect unique labels in first-seen order, accumulating values per (label, series).
        let mut label_order: Vec<String> = Vec::new();
        let mut sums: BTreeMap<(String, usize), f64> = BTreeMap::new();
        let mut cnts: BTreeMap<(String, usize), usize> = BTreeMap::new();
        for (row_i, label) in raw_labels.iter().enumerate() {
            if !label_order.contains(label) {
                label_order.push(label.clone());
            }
            for (si, col_vals) in y_data.iter().enumerate() {
                *sums.entry((label.clone(), si)).or_insert(0.0) += col_vals[row_i];
                *cnts.entry((label.clone(), si)).or_insert(0) += 1;
            }
        }
        let agg_fn = args.agg.as_deref().unwrap_or("mean");

        let pal = Palette::category10();
        let colors: Vec<String> = (0..args.y.len()).map(|i| pal[i].to_string()).collect();

        let mut plot = BarPlot::new();
        if let Some(w) = args.bar_width {
            plot = plot.with_width(w);
        }
        for label in &label_order {
            let bar_values: Vec<(f64, String)> = (0..args.y.len())
                .map(|si| {
                    let key = (label.clone(), si);
                    let val = match agg_fn {
                        "sum" => *sums.get(&key).unwrap_or(&0.0),
                        "min" | "max" => {
                            // min/max require the raw values — fall back to mean
                            let s = sums.get(&key).copied().unwrap_or(0.0);
                            let c = cnts.get(&key).copied().unwrap_or(1).max(1);
                            s / c as f64
                        }
                        _ => {
                            // mean (default)
                            let s = sums.get(&key).copied().unwrap_or(0.0);
                            let c = cnts.get(&key).copied().unwrap_or(1).max(1);
                            s / c as f64
                        }
                    };
                    (val, colors[si].clone())
                })
                .collect();
            plot = plot.with_group(label, bar_values);
        }
        plot = plot.with_legend(series_names.iter().map(|s| s.as_str()).collect());
        if args.horizontal {
            plot = plot.with_horizontal(true);
        }
        let plots = vec![Plot::Bar(plot)];
        let layout = Layout::auto_from_plots(&plots);
        let layout = apply_base_args(layout, &args.base);
        let layout = apply_axis_args(layout, &args.axis);
        let layout = if args.horizontal {
            layout
        } else {
            layout.with_x_tick_rotate(-45.0)
        };
        let scene = render_multiple(plots, layout);
        return write_output(scene, &args.base);
    }

    let mut proj: Vec<ColSpec> = vec![
        args.label_col.clone().unwrap_or(ColSpec::Index(0)),
        if args.y.len() == 1 {
            args.y[0].clone()
        } else {
            args.value_col.clone().unwrap_or(ColSpec::Index(1))
        },
    ];
    if let Some(ref c) = args.count_by {
        proj.push(c.clone());
    }
    if let Some(ref c) = args.color_by {
        proj.push(c.clone());
    }
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
        &proj,
    )?;

    let color = args.color.unwrap_or_else(|| "steelblue".to_string());
    let effective_value_col: ColSpec = if args.y.len() == 1 {
        args.y[0].clone()
    } else {
        args.value_col.unwrap_or(ColSpec::Index(1))
    };

    // --color-by: grouped bar chart (one series per unique value in color_by column)
    if let Some(ref color_by_col) = args.color_by {
        let label_col = args.label_col.unwrap_or(ColSpec::Index(0));
        let value_col = effective_value_col.clone();
        let labels = table.col_str(&label_col)?;
        let series = table.col_str(color_by_col)?;
        let values = table.col_f64(&value_col)?;

        // Collect unique labels and series in insertion order
        let mut label_order: Vec<String> = Vec::new();
        let mut series_order: Vec<String> = Vec::new();
        // sums and counts for mean aggregation
        let mut sums: BTreeMap<(String, String), f64> = BTreeMap::new();
        let mut cnts: BTreeMap<(String, String), usize> = BTreeMap::new();

        for ((lbl, ser), val) in labels.into_iter().zip(series).zip(values) {
            if !label_order.contains(&lbl) {
                label_order.push(lbl.clone());
            }
            if !series_order.contains(&ser) {
                series_order.push(ser.clone());
            }
            *sums.entry((lbl.clone(), ser.clone())).or_insert(0.0) += val;
            *cnts.entry((lbl, ser)).or_insert(0) += 1;
        }

        let pal = Palette::category10();
        let series_colors: Vec<String> = (0..series_order.len())
            .map(|i| pal[i].to_string())
            .collect();

        let mut plot = BarPlot::new();
        if let Some(w) = args.bar_width {
            plot = plot.with_width(w);
        }

        // If every x-label maps to exactly one series (1-to-1), use simple per-bar coloring
        // instead of a grouped layout (which would leave empty sub-bars off-center).
        let is_one_to_one = label_order.len() == series_order.len()
            && label_order.iter().all(|lbl| {
                series_order
                    .iter()
                    .any(|ser| cnts.contains_key(&(lbl.clone(), ser.clone())))
                    && series_order
                        .iter()
                        .filter(|ser| cnts.contains_key(&(lbl.clone(), (*ser).clone())))
                        .count()
                        == 1
            });

        if is_one_to_one {
            // Simple mode: one colored bar per x-label
            let pairs: Vec<(String, f64)> = label_order
                .iter()
                .map(|lbl| {
                    let ser = series_order
                        .iter()
                        .find(|ser| cnts.contains_key(&(lbl.clone(), (*ser).clone())))
                        .unwrap();
                    let key = (lbl.clone(), ser.clone());
                    let val = sums[&key] / cnts[&key] as f64;
                    (lbl.clone(), val)
                })
                .collect();
            let bar_colors: Vec<String> = label_order
                .iter()
                .enumerate()
                .map(|(i, lbl)| {
                    let si = series_order
                        .iter()
                        .position(|ser| cnts.contains_key(&(lbl.clone(), ser.clone())))
                        .unwrap_or(i);
                    series_colors[si].clone()
                })
                .collect();
            plot = plot.with_bars(pairs);
            // Color each bar individually by rebuilding groups with per-bar colors
            for (i, group) in plot.groups.iter_mut().enumerate() {
                if let Some(bar) = group.bars.first_mut() {
                    bar.color = bar_colors[i].clone();
                }
            }
            // No legend needed: x-axis labels already identify each bar
        } else {
            for lbl in &label_order {
                let bar_values: Vec<(f64, &str)> = series_order
                    .iter()
                    .enumerate()
                    .map(|(si, ser)| {
                        let key = (lbl.clone(), ser.clone());
                        let val = if let (Some(&s), Some(&c)) = (sums.get(&key), cnts.get(&key)) {
                            s / c as f64
                        } else {
                            0.0
                        };
                        (val, series_colors[si].as_str())
                    })
                    .collect();
                plot = plot.with_group(lbl, bar_values);
            }
            plot = plot.with_legend(series_order.iter().map(|s| s.as_str()).collect());
        }

        if args.horizontal {
            plot = plot.with_horizontal(true);
        }
        let plots = vec![Plot::Bar(plot)];
        let layout = Layout::auto_from_plots(&plots);
        let layout = apply_base_args(layout, &args.base);
        let layout = apply_axis_args(layout, &args.axis);
        let layout = if args.horizontal { layout } else { layout.with_x_tick_rotate(-45.0) };
        let scene = render_multiple(plots, layout);
        return write_output(scene, &args.base);
    }

    let pairs: Vec<(String, f64)> = if let Some(ref count_col) = args.count_by {
        let values = table.col_str(count_col)?;
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for v in values {
            *counts.entry(v).or_insert(0) += 1;
        }
        counts.into_iter().map(|(k, c)| (k, c as f64)).collect()
    } else if let Some(ref func) = args.agg {
        let label_col = args.label_col.unwrap_or(ColSpec::Index(0));
        let value_col = effective_value_col.clone();
        let labels = table.col_str(&label_col)?;
        let values = table.col_f64(&value_col)?;
        // Accumulate values per group (preserve insertion order via Vec).
        let mut order: Vec<String> = Vec::new();
        let mut groups: BTreeMap<String, Vec<f64>> = BTreeMap::new();
        for (label, val) in labels.into_iter().zip(values) {
            if !groups.contains_key(&label) {
                order.push(label.clone());
            }
            groups.entry(label).or_default().push(val);
        }
        order
            .into_iter()
            .map(|label| {
                let vals = &groups[&label];
                let agg_val = match func.as_str() {
                    "sum" => vals.iter().sum(),
                    "min" => vals.iter().cloned().fold(f64::INFINITY, f64::min),
                    "max" => vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
                    "median" => {
                        let mut s = vals.clone();
                        s.sort_by(|a, b| a.partial_cmp(b).unwrap());
                        let n = s.len();
                        if n.is_multiple_of(2) {
                            (s[n / 2 - 1] + s[n / 2]) / 2.0
                        } else {
                            s[n / 2]
                        }
                    }
                    _ => vals.iter().sum::<f64>() / vals.len() as f64, // "mean" + fallback
                };
                (label, agg_val)
            })
            .collect()
    } else {
        let label_col = args.label_col.unwrap_or(ColSpec::Index(0));
        let labels = table.col_str(&label_col)?;
        let values = table.col_f64(&effective_value_col)?;
        labels.into_iter().zip(values).collect()
    };

    let mut plot = BarPlot::new().with_bars(pairs).with_color(&color);

    if let Some(w) = args.bar_width {
        plot = plot.with_width(w);
    }
    if args.horizontal {
        plot = plot.with_horizontal(true);
    }

    let plots = vec![Plot::Bar(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let layout = if args.horizontal { layout } else { layout.with_x_tick_rotate(-45.0) };
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
