use clap::Args;

use kuva::plot::funnel::{FunnelColorMode, FunnelOrientation, FunnelPlot};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{apply_base_args, BaseArgs};
use crate::output::write_output;

/// Funnel chart — show attrition / conversion through ordered stages.
#[derive(Args, Debug)]
pub struct FunnelArgs {
    /// Stage label column (name or 0-based index; default: 0).
    #[arg(long)]
    pub label: Option<ColSpec>,

    /// Stage value column (name or 0-based index; default: 1).
    #[arg(long)]
    pub value: Option<ColSpec>,

    /// Mirror (right-side) value column — enables diverging back-to-back mode.
    #[arg(long)]
    pub mirror_col: Option<ColSpec>,

    /// Label placed above the left (main) side in diverging mode.
    #[arg(long)]
    pub left_label: Option<String>,

    /// Label placed above the right (mirror) side in diverging mode.
    #[arg(long)]
    pub right_label: Option<String>,

    /// Funnel orientation: `vertical` (default) or `horizontal`.
    #[arg(long, default_value = "vertical")]
    pub orientation: String,

    /// Bar color mode: `uniform` (default), `stage`, `gradient`.
    #[arg(long, default_value = "uniform")]
    pub color_by: String,

    /// Hide trapezoidal connectors between bars.
    #[arg(long)]
    pub no_connectors: bool,

    /// Connector fill opacity 0–1 (default: 0.4).
    #[arg(long)]
    pub connector_opacity: Option<f64>,

    /// Hide absolute value labels on bars.
    #[arg(long)]
    pub no_values: bool,

    /// Show percentage-of-first-stage alongside value labels.
    #[arg(long)]
    pub show_percents: bool,

    /// Hide step-to-step conversion rate in connector areas.
    #[arg(long)]
    pub no_conversion: bool,

    /// Gap in pixels between adjacent bars (default: 4).
    #[arg(long)]
    pub stage_gap: Option<f64>,

    /// Show a legend with this label.
    #[arg(long)]
    pub legend: Option<String>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
}

pub fn run(args: FunnelArgs) -> Result<(), String> {
    let mut proj: Vec<ColSpec> = vec![
        args.label.clone().unwrap_or(ColSpec::Index(0)),
        args.value.clone().unwrap_or(ColSpec::Index(1)),
    ];
    if let Some(ref c) = args.mirror_col { proj.push(c.clone()); }
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
        &proj,
    )?;

    let label_col = args.label.unwrap_or(ColSpec::Index(0));
    let value_col = args.value.unwrap_or(ColSpec::Index(1));

    let labels = table.col_str(&label_col)?;
    let values = table.col_f64(&value_col)?;

    let orientation = match args.orientation.as_str() {
        "horizontal" | "h" => FunnelOrientation::Horizontal,
        _ => FunnelOrientation::Vertical,
    };

    let color_mode = match args.color_by.as_str() {
        "stage" | "by_stage" | "bystage" => FunnelColorMode::ByStage,
        "gradient" => FunnelColorMode::Gradient,
        _ => FunnelColorMode::Uniform,
    };

    let mut plot = FunnelPlot::new()
        .with_orientation(orientation)
        .with_color_mode(color_mode)
        .with_connectors(!args.no_connectors)
        .with_show_values(!args.no_values)
        .with_show_percents(args.show_percents)
        .with_show_conversion(!args.no_conversion);

    if let Some(op) = args.connector_opacity {
        plot = plot.with_connector_opacity(op);
    }
    if let Some(g) = args.stage_gap {
        plot = plot.with_stage_gap(g);
    }
    if let Some(ref label) = args.legend {
        plot = plot.with_legend(label.clone());
    }

    for (label, value) in labels.iter().zip(values.iter()) {
        plot = plot.with_stage(label.clone(), *value);
    }

    // Mirror mode: if mirror_col supplied, read right-side values
    if let Some(ref mcol) = args.mirror_col {
        let mirror_values = table.col_f64(mcol)?;
        plot = plot.with_mirror_stages(
            labels
                .iter()
                .zip(mirror_values.iter())
                .map(|(l, v)| (l.clone(), *v)),
        );
        if let Some(ref ll) = args.left_label {
            if let Some(ref rl) = args.right_label {
                plot = plot.with_mirror_labels(ll.clone(), rl.clone());
            }
        }
    }

    let plots = vec![Plot::Funnel(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
