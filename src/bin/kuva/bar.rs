use std::collections::BTreeMap;

use clap::Args;

use kuva::plot::bar::BarPlot;
use kuva::render::layout::Layout;
use kuva::render::palette::Palette;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, AxisArgs, apply_base_args, apply_axis_args};
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

    /// Group by this column and color each series separately (creates a grouped bar chart).
    #[arg(long)]
    pub color_by: Option<ColSpec>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
}

pub fn run(args: BarArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let color = args.color.unwrap_or_else(|| "steelblue".to_string());

    // --color-by: grouped bar chart (one series per unique value in color_by column)
    if let Some(ref color_by_col) = args.color_by {
        let label_col = args.label_col.unwrap_or(ColSpec::Index(0));
        let value_col = args.value_col.unwrap_or(ColSpec::Index(1));
        let labels  = table.col_str(&label_col)?;
        let series  = table.col_str(color_by_col)?;
        let values  = table.col_f64(&value_col)?;

        // Collect unique labels and series in insertion order
        let mut label_order: Vec<String> = Vec::new();
        let mut series_order: Vec<String> = Vec::new();
        // sums and counts for mean aggregation
        let mut sums: BTreeMap<(String, String), f64> = BTreeMap::new();
        let mut cnts: BTreeMap<(String, String), usize> = BTreeMap::new();

        for ((lbl, ser), val) in labels.into_iter().zip(series).zip(values) {
            if !label_order.contains(&lbl)  { label_order.push(lbl.clone()); }
            if !series_order.contains(&ser) { series_order.push(ser.clone()); }
            *sums.entry((lbl.clone(), ser.clone())).or_insert(0.0) += val;
            *cnts.entry((lbl, ser)).or_insert(0) += 1;
        }

        let pal = Palette::category10();
        let series_colors: Vec<String> = (0..series_order.len()).map(|i| pal[i].to_string()).collect();

        let mut plot = BarPlot::new();
        if let Some(w) = args.bar_width { plot = plot.with_width(w); }

        // If every x-label maps to exactly one series (1-to-1), use simple per-bar coloring
        // instead of a grouped layout (which would leave empty sub-bars off-center).
        let is_one_to_one = label_order.len() == series_order.len()
            && label_order.iter().all(|lbl| {
                series_order.iter().any(|ser| cnts.contains_key(&(lbl.clone(), ser.clone())))
                && series_order.iter().filter(|ser| cnts.contains_key(&(lbl.clone(), (*ser).clone()))).count() == 1
            });

        if is_one_to_one {
            // Simple mode: one colored bar per x-label
            let pairs: Vec<(String, f64)> = label_order.iter().map(|lbl| {
                let ser = series_order.iter()
                    .find(|ser| cnts.contains_key(&(lbl.clone(), (*ser).clone())))
                    .unwrap();
                let key = (lbl.clone(), ser.clone());
                let val = sums[&key] / cnts[&key] as f64;
                (lbl.clone(), val)
            }).collect();
            let bar_colors: Vec<String> = label_order.iter().enumerate().map(|(i, lbl)| {
                let si = series_order.iter().position(|ser| cnts.contains_key(&(lbl.clone(), ser.clone()))).unwrap_or(i);
                series_colors[si].clone()
            }).collect();
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
                let bar_values: Vec<(f64, &str)> = series_order.iter().enumerate().map(|(si, ser)| {
                    let key = (lbl.clone(), ser.clone());
                    let val = if let (Some(&s), Some(&c)) = (sums.get(&key), cnts.get(&key)) {
                        s / c as f64
                    } else {
                        0.0
                    };
                    (val, series_colors[si].as_str())
                }).collect();
                plot = plot.with_group(lbl, bar_values);
            }
            plot = plot.with_legend(series_order.iter().map(|s| s.as_str()).collect());
        }

        let plots = vec![Plot::Bar(plot)];
        let layout = Layout::auto_from_plots(&plots);
        let layout = apply_base_args(layout, &args.base);
        let layout = apply_axis_args(layout, &args.axis);
        let layout = layout.with_x_tick_rotate(-45.0);
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
        let value_col = args.value_col.unwrap_or(ColSpec::Index(1));
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
        order.into_iter().map(|label| {
            let vals = &groups[&label];
            let agg_val = match func.as_str() {
                "sum"    => vals.iter().sum(),
                "min"    => vals.iter().cloned().fold(f64::INFINITY, f64::min),
                "max"    => vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
                "median" => {
                    let mut s = vals.clone();
                    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    let n = s.len();
                    if n.is_multiple_of(2) { (s[n/2 - 1] + s[n/2]) / 2.0 } else { s[n/2] }
                }
                _ => vals.iter().sum::<f64>() / vals.len() as f64, // "mean" + fallback
            };
            (label, agg_val)
        }).collect()
    } else {
        let label_col = args.label_col.unwrap_or(ColSpec::Index(0));
        let value_col = args.value_col.unwrap_or(ColSpec::Index(1));
        let labels = table.col_str(&label_col)?;
        let values = table.col_f64(&value_col)?;
        labels.into_iter().zip(values).collect()
    };

    let mut plot = BarPlot::new()
        .with_bars(pairs)
        .with_color(&color);

    if let Some(w) = args.bar_width {
        plot = plot.with_width(w);
    }

    let plots = vec![Plot::Bar(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let layout = layout.with_x_tick_rotate(-45.0);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
