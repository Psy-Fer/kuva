use clap::Args;

use visus::plot::manhattan::{ManhattanPlot, GenomeBuild};
use visus::render::layout::Layout;
use visus::render::plots::Plot;
use visus::render::render::render_multiple;

use crate::data::{ColSpec, DataTable, InputArgs};
use crate::layout_args::{BaseArgs, AxisArgs, apply_base_args, apply_axis_args};
use crate::output::write_output;

#[derive(Args, Debug)]
pub struct ManhattanArgs {
    /// Chromosome column (0-based index or header name; default: 0).
    #[arg(long)]
    pub chr_col: Option<ColSpec>,

    /// Base-pair position column (default: 1). Used only with --genome-build.
    #[arg(long)]
    pub pos_col: Option<ColSpec>,

    /// p-value column (0-based index or header name; default: 2).
    #[arg(long)]
    pub pvalue_col: Option<ColSpec>,

    /// Reference genome for bp-mode layout: hg19, hg38, t2t.
    #[arg(long)]
    pub genome_build: Option<String>,

    /// Genome-wide significance threshold in -log10(p) scale (default: 7.301).
    #[arg(long)]
    pub genome_wide: Option<f64>,

    /// Suggestive significance threshold in -log10(p) scale (default: 5.0).
    #[arg(long)]
    pub suggestive: Option<f64>,

    /// Label this many most-significant points above the genome-wide threshold.
    #[arg(long)]
    pub top_n: Option<usize>,

    /// Point radius in pixels (default: 2.5).
    #[arg(long)]
    pub point_size: Option<f64>,

    /// Color for even-indexed chromosomes (default: "steelblue").
    #[arg(long)]
    pub color_a: Option<String>,

    /// Color for odd-indexed chromosomes (default: "#5aadcb").
    #[arg(long)]
    pub color_b: Option<String>,

    /// Show a legend for the significance thresholds.
    #[arg(long)]
    pub legend: bool,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
}

pub fn run(args: ManhattanArgs) -> Result<(), String> {
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.no_header,
        args.input.delimiter,
    )?;

    let chr_col = args.chr_col.unwrap_or(ColSpec::Index(0));
    let pos_col = args.pos_col.unwrap_or(ColSpec::Index(1));
    let pvalue_col = args.pvalue_col.unwrap_or(ColSpec::Index(2));

    let chroms = table.col_str(&chr_col)?;
    let pvalues = table.col_f64(&pvalue_col)?;

    let mut plot = ManhattanPlot::new();

    // Load data: bp mode when --genome-build given, sequential otherwise
    if let Some(ref build_str) = args.genome_build {
        let build = match build_str.as_str() {
            "hg19" | "hg37" | "GRCh37" => GenomeBuild::Hg19,
            "hg38" | "GRCh38" => GenomeBuild::Hg38,
            "t2t" | "T2T" | "hs1" => GenomeBuild::T2T,
            other => return Err(format!(
                "unknown genome build '{}'; use hg19, hg38, or t2t", other
            )),
        };
        let positions = table.col_f64(&pos_col)?;
        let data: Vec<(String, f64, f64)> = chroms.into_iter()
            .zip(positions)
            .zip(pvalues)
            .map(|((c, p), pv)| (c, p, pv))
            .collect();
        plot = plot.with_data_bp(data, build);
    } else {
        let data: Vec<(String, f64)> = chroms.into_iter().zip(pvalues).collect();
        plot = plot.with_data(data);
    }

    if let Some(t) = args.genome_wide {
        plot = plot.with_genome_wide(t);
    }
    if let Some(t) = args.suggestive {
        plot = plot.with_suggestive(t);
    }
    if let Some(n) = args.top_n {
        plot = plot.with_label_top(n);
    }
    if let Some(s) = args.point_size {
        plot = plot.with_point_size(s);
    }
    if let Some(ref c) = args.color_a {
        plot = plot.with_color_a(c.clone());
    }
    if let Some(ref c) = args.color_b {
        plot = plot.with_color_b(c.clone());
    }
    if args.legend {
        plot = plot.with_legend("GWAS thresholds");
    }

    let plots = vec![Plot::Manhattan(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let layout = apply_base_args(layout, &args.base);
    let layout = apply_axis_args(layout, &args.axis);
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}
