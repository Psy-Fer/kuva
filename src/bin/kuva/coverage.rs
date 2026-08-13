use std::collections::HashMap;
use std::path::PathBuf;

use clap::Args;

use kuva::render::coverage::CoveragePlot;

use crate::data::{ColSpec, DataTable, HeaderMode, InputArgs};
use crate::layout_args::{theme_from_name, BaseArgs};
use crate::output::write_output;

/// Genomic coverage plot: stacked sequencing-depth tracks (one per sample) sharing a genomic
/// x-axis, with optional variant marks and below-axis feature bands (amplicons / primers / genes).
///
/// The main input is a depth table with a position column and one or more sample-depth columns.
/// Variants and features come from their own files (see `--variants` / `--features`).
///
/// This is a composite figure built on the `TrackStack` primitive. For finer control (extra
/// tracks, region highlights, custom layout) use the `CoveragePlot` / `TrackStack` library API.
#[derive(Args, Debug)]
pub struct CoverageArgs {
    /// Position column in the depth table (0-based index or header name; default: 0).
    #[arg(long)]
    pub x: Option<ColSpec>,

    /// Sample depth column(s), comma-separated (e.g. `--samples tumour,normal`). One filled
    /// depth track is drawn per column. Default: column 1.
    #[arg(long, value_delimiter = ',')]
    pub samples: Vec<ColSpec>,

    /// Overlay all sample depths in one shared track (common y-axis) instead of one per sample.
    #[arg(long)]
    pub overlay_samples: bool,

    /// Variants file (TSV/CSV): two columns `position,type`. Each distinct type becomes a
    /// coloured tick group in one variant lane, with a shared-legend entry.
    #[arg(long, value_name = "FILE")]
    pub variants: Option<PathBuf>,

    /// Features file (TSV/CSV): three columns `start,end,label`. Drawn as a lane above the axis;
    /// overlapping bands tile onto pool-coloured rows (amplicons / primers).
    #[arg(long, value_name = "FILE")]
    pub features: Option<PathBuf>,

    /// Name for the feature track (default: "features").
    #[arg(long)]
    pub feature_name: Option<String>,

    /// Regions file (TSV/CSV): three columns `start,end,label`. A reference genome annotation
    /// (genes / ORFs), drawn just above the axis; labels that don't fit their band are omitted.
    #[arg(long, value_name = "FILE")]
    pub regions: Option<PathBuf>,

    /// Name for the region/gene track (default: "genes").
    #[arg(long)]
    pub region_name: Option<String>,

    /// Locus start (x-min). Pins the genomic window; default is the data extent.
    #[arg(long, allow_hyphen_values = true)]
    pub locus_start: Option<f64>,

    /// Locus end (x-max).
    #[arg(long, allow_hyphen_values = true)]
    pub locus_end: Option<f64>,

    /// X-axis label (default: "position").
    #[arg(long)]
    pub x_label: Option<String>,

    /// Draw a horizontal threshold line at this depth on every coverage track
    /// (e.g. a minimum-coverage cutoff). Repeatable.
    #[arg(long, value_name = "DEPTH")]
    pub min_coverage: Vec<f64>,

    #[command(flatten)]
    pub input: InputArgs,

    #[command(flatten)]
    pub base: BaseArgs,
}

pub fn run(args: CoverageArgs) -> Result<(), String> {
    let x_col = args.x.clone().unwrap_or(ColSpec::Index(0));
    let sample_cols: Vec<ColSpec> = if args.samples.is_empty() {
        vec![ColSpec::Index(1)]
    } else {
        args.samples.clone()
    };

    // Depth table: position + one column per sample.
    let mut proj = vec![x_col.clone()];
    proj.extend(sample_cols.iter().cloned());
    let table = DataTable::parse(
        args.input.input.as_deref(),
        args.input.header_mode(),
        args.input.delimiter,
        &proj,
    )?;
    // Expand column ranges / globs against the parsed table (issue #109).
    let sample_cols = table.expand_columns(&sample_cols)?;
    let xs = table.col_f64(&x_col)?;

    let mut cov = CoveragePlot::new();

    if let Some(name) = &args.base.theme {
        cov = cov.with_theme(theme_from_name(name));
    }
    if let Some(title) = &args.base.title {
        cov = cov.with_title(title.clone());
    }
    if args.overlay_samples {
        cov = cov.with_overlaid_samples();
    }
    for &depth in &args.min_coverage {
        cov = cov.with_coverage_threshold(depth);
    }

    for col in &sample_cols {
        let ys = table.col_f64(col)?;
        let name = table.col_display_name(col);
        let data: Vec<(f64, f64)> = xs.iter().copied().zip(ys).collect();
        cov = cov.with_sample(name, data);
    }

    // Variants (separate file): position,type -> one tick group per distinct type.
    if let Some(path) = &args.variants {
        let pos = ColSpec::Index(0);
        let ty = ColSpec::Index(1);
        let vt = DataTable::parse(
            Some(path.as_path()),
            HeaderMode::Auto,
            args.input.delimiter,
            &[pos.clone(), ty.clone()],
        )?;
        let positions = vt.col_f64(&pos)?;
        let types = vt.col_str(&ty)?;

        // Group positions by type, preserving first-seen order.
        let mut order: Vec<String> = Vec::new();
        let mut groups: HashMap<String, Vec<f64>> = HashMap::new();
        for (p, t) in positions.iter().zip(types.iter()) {
            if !groups.contains_key(t) {
                order.push(t.clone());
            }
            groups.entry(t.clone()).or_default().push(*p);
        }

        // Colourblind-friendly variant palette (Wong-derived), cycled per type.
        const VARIANT_COLORS: [&str; 6] = [
            "#d1495b", "#e9c46a", "#2a9d8f", "#8d5a97", "#5b8e7d", "#e76f51",
        ];
        for (i, t) in order.iter().enumerate() {
            let color = VARIANT_COLORS[i % VARIANT_COLORS.len()];
            let ps = groups.remove(t).unwrap_or_default();
            cov = cov.with_variants(t.clone(), color, ps);
        }
    }

    // Features (separate file): start,end,label -> below-axis bands.
    if let Some(path) = &args.features {
        let start = ColSpec::Index(0);
        let end = ColSpec::Index(1);
        let label = ColSpec::Index(2);
        let ft = DataTable::parse(
            Some(path.as_path()),
            HeaderMode::Auto,
            args.input.delimiter,
            &[start.clone(), end.clone(), label.clone()],
        )?;
        let starts = ft.col_f64(&start)?;
        let ends = ft.col_f64(&end)?;
        let labels = ft.col_str(&label)?;
        for ((s, e), l) in starts.iter().zip(ends.iter()).zip(labels.iter()) {
            cov = cov.with_feature(*s, *e, l.clone());
        }
        if let Some(name) = &args.feature_name {
            cov = cov.with_feature_track_name(name.clone());
        }
    }

    // Regions (genes/ORFs): start,end,label -> reference annotation lane above the axis.
    if let Some(path) = &args.regions {
        let start = ColSpec::Index(0);
        let end = ColSpec::Index(1);
        let label = ColSpec::Index(2);
        let rt = DataTable::parse(
            Some(path.as_path()),
            HeaderMode::Auto,
            args.input.delimiter,
            &[start.clone(), end.clone(), label.clone()],
        )?;
        let starts = rt.col_f64(&start)?;
        let ends = rt.col_f64(&end)?;
        let labels = rt.col_str(&label)?;
        for ((s, e), l) in starts.iter().zip(ends.iter()).zip(labels.iter()) {
            cov = cov.with_region(*s, *e, l.clone());
        }
        if let Some(name) = &args.region_name {
            cov = cov.with_region_track_name(name.clone());
        }
    }

    if let (Some(lo), Some(hi)) = (args.locus_start, args.locus_end) {
        cov = cov.with_locus(lo, hi);
    }
    if let Some(label) = &args.x_label {
        cov = cov.with_x_label(label.clone());
    }

    let width = args.base.width.unwrap_or(1000.0);
    let scene = match args.base.height {
        Some(h) => cov.render_sized(width, h),
        None => cov.render(width),
    };
    write_output(scene, &args.base)
}
