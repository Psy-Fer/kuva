//! Showcase documentation examples — elaborate, multi-plot compositions
//! distinct from the one-card-per-type gallery.
//!
//! Generates canonical SVG outputs used in `docs/src/showcase.md`.
//! Run with:
//!
//! ```bash
//! cargo run --example showcase
//! ```
//!
//! SVGs are written to `docs/src/assets/showcase/`.

use kuva::backend::svg::SvgBackend;
use kuva::plot::scatter::TrendLine;
use kuva::plot::surface3d::Surface3DPlot;
use kuva::plot::{
    BarPlot, Clustermap, ColorMap, LabelStyle, LinePlot, ManhattanPlot, MarkerShape, PhyloTree,
    RocGroup, RocPlot, ScatterPlot, VolcanoPlot,
};
use kuva::render::annotations::{ReferenceLine, TextAnnotation};
use kuva::render::figure::Figure;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

const OUT: &str = "docs/src/assets/showcase";

fn main() {
    std::fs::create_dir_all(OUT).expect("could not create docs/src/assets/showcase");

    interference_surface();
    figure_dashboard();
    genomics_dashboard();
    iris_scatter();
    temperature_trend();
    annotated_volcano();
    model_shootout();
    ecommerce_share();

    println!("Showcase SVGs written to {OUT}/");
}

/// A single, deliberately elaborate figure: two interfering ripple sources
/// (like two stones dropped in a pond) evaluated over a fine grid, rendered
/// as a 3D surface. No multi-panel layout, no dashboard framing — just one
/// plot type pushed hard, to show what kuva's 3D backend can do with a
/// genuinely complex function instead of a plain paraboloid.
fn interference_surface() {
    let source_a = (-3.2, -1.5);
    let source_b = (2.6, 2.0);

    let z_fn = |x: f64, y: f64| {
        let r_a = ((x - source_a.0).powi(2) + (y - source_a.1).powi(2)).sqrt();
        let r_b = ((x - source_b.0).powi(2) + (y - source_b.1).powi(2)).sqrt();
        let ripple_a = (r_a * 2.2).sin() / (r_a * 0.6 + 1.0);
        let ripple_b = (r_b * 2.2).sin() / (r_b * 0.6 + 1.0);
        3.5 * (ripple_a + ripple_b)
    };

    let surface = Surface3DPlot::new(vec![])
        .with_data_fn(z_fn, -8.0..=8.0, -8.0..=8.0, 70, 70)
        .with_z_colormap(ColorMap::Turbo)
        .with_wireframe_color("#00000030")
        .with_wireframe_width(0.3)
        .with_azimuth(-55.0)
        .with_elevation(38.0)
        .with_x_label("X")
        .with_y_label("Y")
        .with_z_label("Amplitude");

    let plots = vec![Plot::Surface3D(surface)];
    let layout = Layout::auto_from_plots(&plots).with_title("Two-Source Wave Interference");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/interference_surface.svg"), svg).unwrap();
}

/// A general-purpose two-panel dashboard: twin-Y axes in one cell (visits vs.
/// conversion rate) next to a plain scatter-with-trend panel (spend vs.
/// revenue), sharing one legend across both.
fn figure_dashboard() {
    let months: Vec<f64> = (1..=6).map(|m| m as f64).collect();

    let visits: Vec<(f64, f64)> = months
        .iter()
        .zip([12_000.0, 15_500.0, 14_200.0, 18_900.0, 21_300.0, 26_800.0])
        .map(|(&m, v)| (m, v))
        .collect();
    let conversion: Vec<(f64, f64)> = months
        .iter()
        .zip([2.1, 2.4, 2.3, 2.9, 3.4, 3.8])
        .map(|(&m, v)| (m, v))
        .collect();

    let primary = vec![Plot::Line(
        LinePlot::new()
            .with_data(visits)
            .with_color("steelblue")
            .with_legend("Site visits"),
    )];
    let secondary = vec![Plot::Line(
        LinePlot::new()
            .with_data(conversion)
            .with_color("crimson")
            .with_legend("Conversion rate (%)"),
    )];

    let spend_revenue: Vec<(f64, f64)> = vec![
        (4.0, 22.0),
        (6.5, 29.0),
        (5.0, 25.0),
        (8.0, 35.0),
        (9.5, 41.0),
        (12.0, 52.0),
        (7.5, 33.0),
        (10.5, 46.0),
    ];
    let scatter_panel = vec![Plot::Scatter(
        ScatterPlot::new()
            .with_data(spend_revenue)
            .with_color("seagreen")
            .with_size(5.0)
            .with_legend("Campaigns")
            .with_trend(TrendLine::Linear),
    )];

    // `with_layouts` matches positionally by cell index (`i < user_layouts.len()`),
    // so the twin-Y cell must come *after* every cell with an explicit layout —
    // otherwise the twin-Y cell would consume that layout instead of falling
    // through to `Layout::auto_from_twin_y_plots`.
    let layouts = vec![Layout::auto_from_plots(&scatter_panel)
        .with_title("Ad Spend vs. Revenue")
        .with_x_label("Ad spend ($k)")
        .with_y_label("Revenue ($k)")];

    let scene = Figure::new(1, 2)
        .with_title("Monthly Growth Dashboard")
        .with_plots(vec![scatter_panel, vec![]])
        .with_twin_y_plots(1, primary, secondary)
        .with_layouts(layouts)
        .with_shared_legend()
        .with_cell_size(460.0, 360.0)
        .render();

    std::fs::write(
        format!("{OUT}/figure_dashboard.svg"),
        SvgBackend.render_scene(&scene),
    )
    .unwrap();
}

/// A genomics-themed dashboard: a GWAS Manhattan plot, a gene-expression
/// clustermap, and a phylogenetic tree in one Figure. Manhattan draws
/// standard axes; Clustermap and PhyloTree are pixel-space plot types with
/// no axis chrome — the mix is intentional, showing how differently-shaped
/// plot types compose in one Figure rather than a rendering inconsistency.
fn genomics_dashboard() {
    // Six chromosomes, one clear peak on chr3, mild background noise.
    // `with_data` takes *raw* p-values (0..1) and computes -log10(p) itself —
    // not pre-transformed values, per its own doc example (`3e-8`, `5e-6`).
    let mut gwas: Vec<(String, f64)> = Vec::new();
    for (chrom, n, peak_at) in [
        ("chr1", 40, None),
        ("chr2", 40, None),
        ("chr3", 40, Some(20)),
        ("chr4", 40, None),
        ("chr5", 40, None),
        ("chr6", 40, None),
    ] {
        for i in 0..n {
            let base_p = 0.9 - 0.85 * ((i as f64 * 0.31).sin().abs());
            let p = match peak_at {
                Some(center) if (i as i64 - center as i64).unsigned_abs() < 4 => {
                    let dist = (i as i64 - center as i64).unsigned_abs() as f64;
                    10f64.powf(-8.0 + dist * 2.0)
                }
                _ => base_p,
            };
            gwas.push((chrom.to_string(), p));
        }
    }
    let manhattan = vec![Plot::Manhattan(ManhattanPlot::new().with_data(gwas))];

    // Same 8-gene / 6-condition expression matrix used in the Clustermap docs.
    let expr = vec![
        vec![8.2, 7.9, 0.4, 0.2, 0.1, 0.3],
        vec![7.8, 8.1, 0.3, 0.1, 0.4, 0.2],
        vec![0.2, 0.3, 7.5, 8.0, 0.1, 0.2],
        vec![0.1, 0.4, 8.1, 7.6, 0.3, 0.1],
        vec![0.3, 0.1, 0.2, 0.1, 8.3, 7.9],
        vec![0.2, 0.2, 0.1, 0.3, 7.8, 8.2],
        vec![4.1, 0.3, 3.9, 0.2, 4.2, 0.1],
        vec![0.1, 4.3, 0.2, 4.0, 0.3, 4.1],
    ];
    let clustermap = vec![Plot::Clustermap(
        Clustermap::new()
            .with_data(expr)
            .with_row_labels([
                "Gene1", "Gene2", "Gene3", "Gene4", "Gene5", "Gene6", "Gene7", "Gene8",
            ])
            .with_col_labels(["CtrlA", "CtrlB", "TreatA", "TreatB", "StimA", "StimB"])
            .with_color_map(ColorMap::Viridis),
    )];

    let edges: Vec<(&str, &str, f64)> = vec![
        ("root", "Bacteria", 1.5),
        ("root", "Eukarya", 2.0),
        ("Bacteria", "E. coli", 0.5),
        ("Bacteria", "B. subtilis", 0.7),
        ("Eukarya", "Yeast", 1.0),
        ("Eukarya", "Human", 0.8),
    ];
    let phylo = vec![Plot::PhyloTree(
        PhyloTree::from_edges(&edges)
            .with_clade_color(1, "#e41a1c")
            .with_clade_color(2, "#377eb8"),
    )];

    let layouts = vec![
        Layout::auto_from_plots(&manhattan)
            .with_title("GWAS Signal")
            .with_y_label("−log₁₀(p)"),
        Layout::auto_from_plots(&clustermap).with_title("Expression"),
        Layout::auto_from_plots(&phylo).with_title("Phylogeny"),
    ];

    let scene = Figure::new(1, 3)
        .with_title("Genomics Dashboard")
        .with_plots(vec![manhattan, clustermap, phylo])
        .with_layouts(layouts)
        .with_cell_size(380.0, 380.0)
        .render();

    std::fs::write(
        format!("{OUT}/genomics_dashboard.svg"),
        SvgBackend.render_scene(&scene),
    )
    .unwrap();
}

/// The real Fisher/Anderson Iris dataset (150 flowers, all 3 species):
/// petal length vs. petal width, one marker shape and trend line per
/// species. An arrowed annotation calls out setosa's famous linear
/// separability from the other two species.
fn iris_scatter() {
    // (petal_length, petal_width) — real data, github.com/mwaskom/seaborn-data/iris.csv
    let setosa: Vec<(f64, f64)> = vec![
        (1.4, 0.2),
        (1.4, 0.2),
        (1.3, 0.2),
        (1.5, 0.2),
        (1.4, 0.2),
        (1.7, 0.4),
        (1.4, 0.3),
        (1.5, 0.2),
        (1.4, 0.2),
        (1.5, 0.1),
        (1.5, 0.2),
        (1.6, 0.2),
        (1.4, 0.1),
        (1.1, 0.1),
        (1.2, 0.2),
        (1.5, 0.4),
        (1.3, 0.4),
        (1.4, 0.3),
        (1.7, 0.3),
        (1.5, 0.3),
        (1.7, 0.2),
        (1.5, 0.4),
        (1.0, 0.2),
        (1.7, 0.5),
        (1.9, 0.2),
        (1.6, 0.2),
        (1.6, 0.4),
        (1.5, 0.2),
        (1.4, 0.2),
        (1.6, 0.2),
        (1.6, 0.2),
        (1.5, 0.4),
        (1.5, 0.1),
        (1.4, 0.2),
        (1.5, 0.2),
        (1.2, 0.2),
        (1.3, 0.2),
        (1.4, 0.1),
        (1.3, 0.2),
        (1.5, 0.2),
        (1.3, 0.3),
        (1.3, 0.3),
        (1.3, 0.2),
        (1.6, 0.6),
        (1.9, 0.4),
        (1.4, 0.3),
        (1.6, 0.2),
        (1.4, 0.2),
        (1.5, 0.2),
        (1.4, 0.2),
    ];
    let versicolor: Vec<(f64, f64)> = vec![
        (4.7, 1.4),
        (4.5, 1.5),
        (4.9, 1.5),
        (4.0, 1.3),
        (4.6, 1.5),
        (4.5, 1.3),
        (4.7, 1.6),
        (3.3, 1.0),
        (4.6, 1.3),
        (3.9, 1.4),
        (3.5, 1.0),
        (4.2, 1.5),
        (4.0, 1.0),
        (4.7, 1.4),
        (3.6, 1.3),
        (4.4, 1.4),
        (4.5, 1.5),
        (4.1, 1.0),
        (4.5, 1.5),
        (3.9, 1.1),
        (4.8, 1.8),
        (4.0, 1.3),
        (4.9, 1.5),
        (4.7, 1.2),
        (4.3, 1.3),
        (4.4, 1.4),
        (4.8, 1.4),
        (5.0, 1.7),
        (4.5, 1.5),
        (3.5, 1.0),
        (3.8, 1.1),
        (3.7, 1.0),
        (3.9, 1.2),
        (5.1, 1.6),
        (4.5, 1.5),
        (4.5, 1.6),
        (4.7, 1.5),
        (4.4, 1.3),
        (4.1, 1.3),
        (4.0, 1.3),
        (4.4, 1.2),
        (4.6, 1.4),
        (4.0, 1.2),
        (3.3, 1.0),
        (4.2, 1.3),
        (4.2, 1.2),
        (4.2, 1.3),
        (4.3, 1.3),
        (3.0, 1.1),
        (4.1, 1.3),
    ];
    let virginica: Vec<(f64, f64)> = vec![
        (6.0, 2.5),
        (5.1, 1.9),
        (5.9, 2.1),
        (5.6, 1.8),
        (5.8, 2.2),
        (6.6, 2.1),
        (4.5, 1.7),
        (6.3, 1.8),
        (5.8, 1.8),
        (6.1, 2.5),
        (5.1, 2.0),
        (5.3, 1.9),
        (5.5, 2.1),
        (5.0, 2.0),
        (5.1, 2.4),
        (5.3, 2.3),
        (5.5, 1.8),
        (6.7, 2.2),
        (6.9, 2.3),
        (5.0, 1.5),
        (5.7, 2.3),
        (4.9, 2.0),
        (6.7, 2.0),
        (4.9, 1.8),
        (5.7, 2.1),
        (6.0, 1.8),
        (4.8, 1.8),
        (4.9, 1.8),
        (5.6, 2.1),
        (5.8, 1.6),
        (6.1, 1.9),
        (6.4, 2.0),
        (5.6, 2.2),
        (5.1, 1.5),
        (5.6, 1.4),
        (6.1, 2.3),
        (5.6, 2.4),
        (5.5, 1.8),
        (4.8, 1.8),
        (5.4, 2.1),
        (5.6, 2.4),
        (5.1, 2.3),
        (5.1, 1.9),
        (5.9, 2.3),
        (5.7, 2.5),
        (5.2, 2.3),
        (5.0, 1.9),
        (5.2, 2.0),
        (5.4, 2.3),
        (5.1, 1.8),
    ];

    /// `(label, marker, colour, points)` for one iris species.
    type Species = (&'static str, MarkerShape, &'static str, Vec<(f64, f64)>);
    let species: [Species; 3] = [
        ("setosa", MarkerShape::Circle, "#1f77b4", setosa),
        ("versicolor", MarkerShape::Square, "#ff7f0e", versicolor),
        ("virginica", MarkerShape::Triangle, "#2ca02c", virginica),
    ];

    let plots: Vec<Plot> = species
        .into_iter()
        .map(|(label, marker, color, data)| {
            Plot::Scatter(
                ScatterPlot::new()
                    .with_data(data)
                    .with_color(color)
                    .with_size(5.0)
                    .with_marker(marker)
                    .with_legend(label)
                    .with_trend(TrendLine::Linear),
            )
        })
        .collect();

    let layout = Layout::auto_from_plots(&plots)
        .with_title("The Iris Dataset")
        .with_x_label("Petal length (cm)")
        .with_y_label("Petal width (cm)")
        .with_annotation(
            TextAnnotation::new("Setosa is linearly separable", 3.2, 0.3)
                .with_arrow(1.5, 0.25)
                .with_color("#1f77b4"),
        );

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/iris_scatter.svg"), svg).unwrap();
}

/// Real NASA GISS annual global temperature anomaly, 1980-2025 (source:
/// data.giss.nasa.gov/gistemp, GLB.Ts+dSST.csv, J-D column, vs. the
/// 1951-1980 baseline). A raw annual line plus a computed 5-year rolling
/// mean, two reference lines, and an annotation on the warmest year on
/// record: the same annual-plus-smoothed presentation NASA's own public
/// charts use for this series.
fn temperature_trend() {
    let annual: Vec<(i32, f64)> = vec![
        (1980, 0.25),
        (1981, 0.32),
        (1982, 0.14),
        (1983, 0.31),
        (1984, 0.15),
        (1985, 0.12),
        (1986, 0.18),
        (1987, 0.32),
        (1988, 0.39),
        (1989, 0.27),
        (1990, 0.45),
        (1991, 0.41),
        (1992, 0.22),
        (1993, 0.23),
        (1994, 0.31),
        (1995, 0.44),
        (1996, 0.33),
        (1997, 0.46),
        (1998, 0.61),
        (1999, 0.38),
        (2000, 0.39),
        (2001, 0.53),
        (2002, 0.63),
        (2003, 0.61),
        (2004, 0.53),
        (2005, 0.68),
        (2006, 0.64),
        (2007, 0.66),
        (2008, 0.54),
        (2009, 0.66),
        (2010, 0.72),
        (2011, 0.61),
        (2012, 0.65),
        (2013, 0.68),
        (2014, 0.75),
        (2015, 0.90),
        (2016, 1.01),
        (2017, 0.91),
        (2018, 0.85),
        (2019, 0.98),
        (2020, 1.01),
        (2021, 0.85),
        (2022, 0.89),
        (2023, 1.17),
        (2024, 1.28),
        (2025, 1.19),
    ];

    let annual_line = Plot::Line(
        LinePlot::new()
            .with_data(annual.iter().map(|&(y, v)| (y as f64, v)))
            .with_color("#aaaaaa")
            .with_legend("Annual mean"),
    );

    // 5-year centered rolling mean.
    let smoothed: Vec<(f64, f64)> = annual
        .windows(5)
        .map(|w| {
            let year = w[2].0 as f64;
            let mean = w.iter().map(|&(_, v)| v).sum::<f64>() / 5.0;
            (year, mean)
        })
        .collect();
    let smoothed_line = Plot::Line(
        LinePlot::new()
            .with_data(smoothed)
            .with_color("crimson")
            .with_stroke_width(2.5)
            .with_legend("5-year mean"),
    );

    let plots = vec![annual_line, smoothed_line];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Global Temperature Anomaly, 1980-2025")
        .with_x_label("Year")
        .with_y_label("Anomaly vs. 1951-1980 baseline (°C)")
        .with_reference_line(
            ReferenceLine::horizontal(0.0)
                .with_color("#555")
                .with_label("1951-1980 baseline"),
        )
        .with_reference_line(
            ReferenceLine::horizontal(1.5)
                .with_color("crimson")
                .with_dasharray("2 3")
                .with_label("Paris Agreement 1.5°C"),
        )
        .with_annotation(
            TextAnnotation::new("2024: warmest year on record", 2010.0, 1.15)
                .with_arrow(2024.0, 1.28)
                .with_color("crimson"),
        );

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/temperature_trend.svg"), svg).unwrap();
}

/// A volcano plot built entirely from real DESeq2 output for the classic
/// Himes et al. 2014 airway RNA-seq experiment (dexamethasone-treated
/// airway smooth muscle cells): ~230 genes taken directly, in their
/// original file order (not pre-sorted by significance), from
/// github.com/stephenturner/deseq-to-fgsea's published results table, plus
/// the 6 most significant genes overall (from a separate real DESeq2
/// airway run). No synthetic filler — the "flat" non-significant cloud and
/// the graded significant tail are both real, which is what gives this one
/// its actual volcano shape instead of two disconnected clusters.
/// Automatic significance coloring and threshold lines; `.with_label_top(6)`
/// naturally picks out the 6 real most-significant genes by name.
fn annotated_volcano() {
    // (gene_id, log2FoldChange, pvalue) — real DESeq2 output, airway dataset,
    // genes in their original (Ensembl ID) file order, not sorted by p-value.
    //
    // `approx_constant` is deny-by-default and fires on measured values that
    // happen to land near a math constant: the -0.318 fold change below reads
    // as -1/π and the 0.7071 p-value as 1/√2. They are neither; substituting
    // `FRAC_1_PI`/`FRAC_1_SQRT_2` would silently falsify the dataset.
    #[allow(clippy::approx_constant)]
    let points: Vec<(&str, f64, f64)> = vec![
        ("ENSG00000000003", 0.381, 1.52e-4),
        ("ENSG00000000419", -0.207, 0.0653),
        ("ENSG00000000457", -0.038, 0.7915),
        ("ENSG00000000460", 0.088, 0.7588),
        ("ENSG00000000938", 1.378, 0.6937),
        ("ENSG00000000971", -0.426, 1.38e-6),
        ("ENSG00000001036", 0.241, 0.0066),
        ("ENSG00000001084", 0.048, 0.7751),
        ("ENSG00000001167", 0.500, 3.48e-5),
        ("ENSG00000001460", 0.124, 0.4910),
        ("ENSG00000001461", 0.041, 0.6889),
        ("ENSG00000001497", 0.004, 0.9697),
        ("ENSG00000001561", -0.225, 0.2990),
        ("ENSG00000001617", 0.250, 0.0208),
        ("ENSG00000001626", 1.087, 0.1699),
        ("ENSG00000001629", 0.227, 0.0049),
        ("ENSG00000001630", -0.013, 0.9678),
        ("ENSG00000001631", 0.037, 0.7223),
        ("ENSG00000002016", 0.311, 0.0522),
        ("ENSG00000002079", 1.981, 0.2818),
        ("ENSG00000002330", -0.089, 0.5660),
        ("ENSG00000002549", -0.187, 0.0777),
        ("ENSG00000002586", -0.090, 0.2645),
        ("ENSG00000002587", 0.717, 0.8278),
        ("ENSG00000002726", 0.464, 0.8953),
        ("ENSG00000002745", 0.269, 0.7289),
        ("ENSG00000002746", -0.102, 0.6067),
        ("ENSG00000002822", 0.160, 0.2265),
        ("ENSG00000002834", -0.397, 1.97e-5),
        ("ENSG00000002919", -0.404, 0.0039),
        ("ENSG00000002933", -0.361, 0.0137),
        ("ENSG00000003056", 0.073, 0.4311),
        ("ENSG00000003096", 0.941, 6.00e-11),
        ("ENSG00000003137", 0.897, 0.0022),
        ("ENSG00000003147", -0.318, 0.8997),
        ("ENSG00000003249", 0.280, 0.1500),
        ("ENSG00000003393", -0.172, 0.0725),
        ("ENSG00000003400", 0.408, 0.0726),
        ("ENSG00000003402", -1.190, 5.66e-23),
        ("ENSG00000003436", 0.197, 0.1893),
        ("ENSG00000003509", 0.051, 0.7071),
        ("ENSG00000003756", 0.189, 0.0472),
        ("ENSG00000003987", -1.005, 0.0070),
        ("ENSG00000003989", -0.551, 0.1470),
        ("ENSG00000004059", -0.367, 3.29e-5),
        ("ENSG00000004139", 0.166, 0.4513),
        ("ENSG00000004142", -0.144, 0.0985),
        ("ENSG00000004399", 0.039, 0.7288),
        ("ENSG00000004455", -0.030, 0.8015),
        ("ENSG00000004478", 0.073, 0.5189),
        ("ENSG00000004487", 0.300, 6.33e-4),
        ("ENSG00000004534", 0.188, 0.0398),
        ("ENSG00000004660", 0.565, 0.0360),
        ("ENSG00000004700", -0.369, 2.89e-4),
        ("ENSG00000004766", 0.030, 0.7881),
        ("ENSG00000004776", 0.074, 0.4085),
        ("ENSG00000004777", 0.409, 0.0564),
        ("ENSG00000004779", -0.104, 0.4145),
        ("ENSG00000004799", -2.645, 2.58e-5),
        ("ENSG00000004838", 0.415, 0.4539),
        ("ENSG00000004846", 1.698, 0.0023),
        ("ENSG00000004864", 0.001, 0.9970),
        ("ENSG00000004866", 0.562, 3.66e-5),
        ("ENSG00000004897", -0.054, 0.5377),
        ("ENSG00000004961", 0.147, 0.2708),
        ("ENSG00000004975", 0.320, 0.0010),
        ("ENSG00000005007", -0.239, 0.0039),
        ("ENSG00000005020", -0.218, 0.0474),
        ("ENSG00000005022", 0.033, 0.7289),
        ("ENSG00000005059", -0.087, 0.5750),
        ("ENSG00000005075", -0.057, 0.5994),
        ("ENSG00000005100", -0.361, 0.0063),
        ("ENSG00000005108", -0.556, 0.3376),
        ("ENSG00000005156", 0.393, 0.0223),
        ("ENSG00000005175", 0.223, 0.1028),
        ("ENSG00000005187", -0.028, 0.9650),
        ("ENSG00000005189", 0.366, 0.1698),
        ("ENSG00000005194", -0.166, 0.1324),
        ("ENSG00000005206", -0.065, 0.5506),
        ("ENSG00000005238", 0.183, 0.0938),
        ("ENSG00000005243", 0.098, 0.2919),
        ("ENSG00000005249", -0.607, 7.86e-4),
        ("ENSG00000005302", -0.146, 0.2437),
        ("ENSG00000005339", 0.102, 0.3173),
        ("ENSG00000005379", -0.240, 0.2044),
        ("ENSG00000005436", -0.148, 0.2683),
        ("ENSG00000005448", -0.304, 0.0770),
        ("ENSG00000005469", 0.242, 0.0720),
        ("ENSG00000005471", 1.120, 0.0017),
        ("ENSG00000005483", -0.085, 0.3422),
        ("ENSG00000005486", 0.112, 0.2573),
        ("ENSG00000005700", 0.055, 0.5707),
        ("ENSG00000005801", 0.304, 0.0501),
        ("ENSG00000005810", 0.022, 0.8118),
        ("ENSG00000005812", -0.469, 2.58e-5),
        ("ENSG00000005882", 0.067, 0.5650),
        ("ENSG00000005884", -0.349, 0.0061),
        ("ENSG00000005889", -0.082, 0.3953),
        ("ENSG00000005893", -0.067, 0.3729),
        ("ENSG00000005955", -0.108, 0.2881),
        ("ENSG00000005961", 0.265, 0.7009),
        ("ENSG00000006007", -0.169, 0.0485),
        ("ENSG00000006015", 0.042, 0.7617),
        ("ENSG00000006016", -0.289, 0.1926),
        ("ENSG00000006025", 0.949, 1.20e-7),
        ("ENSG00000006042", 0.032, 0.6876),
        ("ENSG00000006062", 0.698, 3.81e-4),
        ("ENSG00000006114", 0.513, 8.33e-8),
        ("ENSG00000006118", 0.502, 2.63e-5),
        ("ENSG00000006125", -0.042, 0.6902),
        ("ENSG00000006194", -0.101, 0.4097),
        ("ENSG00000006210", -1.310, 0.0500),
        ("ENSG00000006282", 0.109, 0.2169),
        ("ENSG00000006283", 1.467, 3.12e-7),
        ("ENSG00000006327", 0.257, 0.2230),
        ("ENSG00000006432", 0.114, 0.8914),
        ("ENSG00000006451", 0.558, 2.03e-6),
        ("ENSG00000006453", 0.209, 0.2119),
        ("ENSG00000006459", 0.008, 0.9669),
        ("ENSG00000006468", 0.264, 0.2094),
        ("ENSG00000006530", -0.004, 0.9721),
        ("ENSG00000006534", -0.449, 1.30e-4),
        ("ENSG00000006576", 0.430, 8.75e-6),
        ("ENSG00000006607", -0.598, 2.46e-7),
        ("ENSG00000006625", 0.163, 0.3087),
        ("ENSG00000006634", 0.358, 0.0859),
        ("ENSG00000006638", 0.644, 0.0079),
        ("ENSG00000006652", 0.044, 0.6449),
        ("ENSG00000006695", -0.224, 0.1783),
        ("ENSG00000006704", 0.224, 0.1418),
        ("ENSG00000006712", 0.202, 0.0456),
        ("ENSG00000006715", 0.063, 0.4648),
        ("ENSG00000006740", 0.048, 0.8817),
        ("ENSG00000006744", -0.165, 0.1219),
        ("ENSG00000006756", 0.004, 0.9728),
        ("ENSG00000006757", -0.016, 0.9300),
        ("ENSG00000006788", -3.207, 3.95e-4),
        ("ENSG00000006831", -0.242, 0.0092),
        ("ENSG00000006837", 0.742, 0.1772),
        ("ENSG00000007047", -0.091, 0.3795),
        ("ENSG00000007080", -0.241, 0.0353),
        ("ENSG00000007168", -0.107, 0.1607),
        ("ENSG00000007202", 0.006, 0.9414),
        ("ENSG00000007237", 1.405, 1.47e-13),
        ("ENSG00000007255", -0.259, 0.2276),
        ("ENSG00000007341", -0.125, 0.5038),
        ("ENSG00000007372", -0.305, 0.3651),
        ("ENSG00000007376", -0.108, 0.4324),
        ("ENSG00000007384", -0.064, 0.5184),
        ("ENSG00000007392", 0.173, 0.0893),
        ("ENSG00000007402", -0.344, 0.5100),
        ("ENSG00000007516", -0.366, 0.5674),
        ("ENSG00000007520", -0.104, 0.4089),
        ("ENSG00000007541", -0.163, 0.2081),
        ("ENSG00000007545", -0.093, 0.5298),
        ("ENSG00000007866", -0.596, 3.66e-12),
        ("ENSG00000007923", -0.032, 0.7524),
        ("ENSG00000007933", -0.245, 0.0303),
        ("ENSG00000007944", 0.498, 6.06e-5),
        ("ENSG00000007952", -0.637, 0.2793),
        ("ENSG00000008018", -0.043, 0.6052),
        ("ENSG00000008056", -0.704, 0.1363),
        ("ENSG00000008083", -0.442, 0.0025),
        ("ENSG00000008086", 0.241, 0.3055),
        ("ENSG00000008118", 0.373, 0.6494),
        ("ENSG00000008128", 0.207, 0.6003),
        ("ENSG00000008130", -0.776, 4.62e-10),
        ("ENSG00000008226", 0.272, 0.7406),
        ("ENSG00000008256", -1.183, 2.01e-30),
        ("ENSG00000008277", 0.281, 0.5193),
        ("ENSG00000008282", -0.280, 0.0086),
        ("ENSG00000008283", -0.008, 0.9454),
        ("ENSG00000008294", 0.031, 0.6933),
        ("ENSG00000008300", -0.264, 0.2235),
        ("ENSG00000008311", -1.096, 3.26e-24),
        // Real most-significant genes overall (separate DESeq2 airway run,
        // github.com/lashlock/compbio tutorial output).
        ("ENSG00000152583", 3.967, 1.54e-76),
        ("ENSG00000179094", 2.714, 4.13e-60),
        ("ENSG00000116584", -1.027, 1.16e-57),
        ("ENSG00000189221", 3.091, 8.58e-56),
        ("ENSG00000120129", 2.759, 5.39e-48),
        ("ENSG00000148175", 1.402, 4.46e-46),
    ];

    let volcano = VolcanoPlot::new()
        .with_points(points.into_iter().map(|(n, fc, p)| (n.to_string(), fc, p)))
        .with_fc_cutoff(1.0)
        .with_p_cutoff(0.001)
        .with_label_top(6)
        .with_label_style(LabelStyle::Arrow {
            offset_x: 0.3,
            offset_y: 0.3,
        })
        .with_legend("Significance");

    let plots = vec![Plot::Volcano(volcano)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Airway Smooth Muscle: Dexamethasone Response")
        .with_x_label("log₂(fold change)")
        .with_y_label("−log₁₀(p-value)")
        .with_x_axis_max(7.0);

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/annotated_volcano.svg"), svg).unwrap();
}

/// Three naive "classifiers" compared on one ROC axes, built from real
/// diagnostic measurements in the UCI Breast Cancer Wisconsin (Diagnostic)
/// dataset (569-sample cohort; first 120 rows used here): mean radius
/// alone, mean texture alone, and a simple combined z-score of both. Each
/// is just a raw feature value treated as a classifier score, not a
/// trained model, but AUC is computed and labeled automatically per group
/// from the real malignant/benign labels.
///
/// The curves actually cross: the combined score has a higher TPR than
/// radius alone in the middle of the FPR range, but radius alone has a much
/// larger lead right at the very start (FPR 0 to ~0.05, where TPR is 0.56
/// to 0.70 for radius alone vs. 0.11 to 0.46 for the combined score). That
/// early lead, in the operating region a real diagnostic test usually cares
/// about most, outweighs the combined score's smaller mid-range advantage
/// once integrated over the whole curve, which is why radius alone still
/// wins on total AUC despite not having the more visually obvious "knee."
fn model_shootout() {
    // (is_malignant, radius_mean, texture_mean) — real data, UCI Breast
    // Cancer Wisconsin (Diagnostic) dataset, first 120 rows.
    let samples: Vec<(bool, f64, f64)> = vec![
        (true, 17.99, 10.38),
        (true, 20.57, 17.77),
        (true, 19.69, 21.25),
        (true, 11.42, 20.38),
        (true, 20.29, 14.34),
        (true, 12.45, 15.70),
        (true, 18.25, 19.98),
        (true, 13.71, 20.83),
        (true, 13.00, 21.82),
        (true, 12.46, 24.04),
        (true, 16.02, 23.24),
        (true, 15.78, 17.89),
        (true, 19.17, 24.80),
        (true, 15.85, 23.95),
        (true, 13.73, 22.61),
        (true, 14.54, 27.54),
        (true, 14.68, 20.13),
        (true, 16.13, 20.68),
        (true, 19.81, 22.15),
        (false, 13.54, 14.36),
        (false, 13.08, 15.71),
        (false, 9.504, 12.44),
        (true, 15.34, 14.26),
        (true, 21.16, 23.04),
        (true, 16.65, 21.38),
        (true, 17.14, 16.40),
        (true, 14.58, 21.53),
        (true, 18.61, 20.25),
        (true, 15.30, 25.27),
        (true, 17.57, 15.05),
        (true, 18.63, 25.11),
        (true, 11.84, 18.70),
        (true, 17.02, 23.98),
        (true, 19.27, 26.47),
        (true, 16.13, 17.88),
        (true, 16.74, 21.59),
        (true, 14.25, 21.72),
        (false, 13.03, 18.42),
        (true, 14.99, 25.20),
        (true, 13.48, 20.82),
        (true, 13.44, 21.58),
        (true, 10.95, 21.35),
        (true, 19.07, 24.81),
        (true, 13.28, 20.28),
        (true, 13.17, 21.81),
        (true, 18.65, 17.60),
        (false, 8.196, 16.84),
        (true, 13.17, 18.66),
        (false, 12.05, 14.63),
        (false, 13.49, 22.30),
        (false, 11.76, 21.60),
        (false, 13.64, 16.34),
        (false, 11.94, 18.24),
        (true, 18.22, 18.70),
        (true, 15.10, 22.02),
        (false, 11.52, 18.75),
        (true, 19.21, 18.57),
        (true, 14.71, 21.59),
        (true, 18.94, 21.31),
        (false, 8.888, 14.64),
        (true, 17.20, 24.52),
        (true, 13.80, 15.79),
        (false, 12.31, 16.52),
        (true, 16.07, 19.65),
        (false, 13.53, 10.94),
        (true, 18.05, 16.15),
        (true, 20.18, 23.97),
        (false, 12.86, 18.00),
        (false, 11.45, 20.97),
        (false, 13.34, 15.86),
        (true, 25.22, 24.91),
        (true, 19.10, 26.29),
        (false, 12.00, 15.65),
        (true, 18.46, 18.52),
        (true, 14.48, 21.46),
        (true, 19.02, 24.59),
        (false, 12.36, 21.80),
        (false, 14.64, 15.24),
        (false, 14.62, 24.02),
        (true, 15.37, 22.76),
        (false, 13.27, 14.76),
        (false, 13.45, 18.30),
        (true, 15.06, 19.83),
        (true, 20.26, 23.03),
        (false, 12.18, 17.84),
        (false, 9.787, 19.94),
        (false, 11.60, 12.84),
        (true, 14.42, 19.77),
        (true, 13.61, 24.98),
        (false, 6.981, 13.43),
        (false, 12.18, 20.52),
        (false, 9.876, 19.40),
        (false, 10.49, 19.29),
        (true, 13.11, 15.56),
        (false, 11.64, 18.33),
        (false, 12.36, 18.54),
        (false, 11.34, 21.26),
        (false, 9.777, 16.99),
        (false, 12.63, 20.76),
        (false, 14.26, 19.65),
        (false, 10.51, 20.19),
        (false, 8.726, 15.83),
        (false, 11.93, 21.53),
        (false, 8.950, 15.76),
        (true, 14.87, 16.67),
        (true, 15.78, 22.91),
        (true, 17.95, 20.01),
        (true, 18.66, 17.12),
        (true, 24.25, 20.20),
        (false, 14.50, 10.89),
        (false, 13.37, 16.39),
        (false, 13.85, 17.21),
        (true, 13.61, 24.69),
        (true, 19.00, 18.91),
        (false, 15.10, 16.39),
        (true, 19.79, 25.12),
        (false, 12.19, 13.29),
        (true, 15.46, 19.48),
        (true, 16.16, 21.54),
        (false, 15.71, 13.93),
        (true, 18.45, 21.91),
        (true, 12.77, 22.47),
        (false, 11.71, 16.67),
        (false, 11.43, 15.39),
        (true, 14.95, 17.57),
        (true, 17.05, 19.08),
        (false, 11.32, 27.08),
        (false, 11.22, 33.81),
    ];

    let radius_scores: Vec<(f64, bool)> = samples.iter().map(|&(m, r, _)| (r, m)).collect();
    let texture_scores: Vec<(f64, bool)> = samples.iter().map(|&(m, _, t)| (t, m)).collect();

    let n = samples.len() as f64;
    let radius_mean = samples.iter().map(|&(_, r, _)| r).sum::<f64>() / n;
    let radius_std = (samples
        .iter()
        .map(|&(_, r, _)| (r - radius_mean).powi(2))
        .sum::<f64>()
        / n)
        .sqrt();
    let texture_mean = samples.iter().map(|&(_, _, t)| t).sum::<f64>() / n;
    let texture_std = (samples
        .iter()
        .map(|&(_, _, t)| (t - texture_mean).powi(2))
        .sum::<f64>()
        / n)
        .sqrt();
    let combined_scores: Vec<(f64, bool)> = samples
        .iter()
        .map(|&(m, r, t)| {
            let z = (r - radius_mean) / radius_std + (t - texture_mean) / texture_std;
            (z, m)
        })
        .collect();

    let combined = RocGroup::new("Radius + texture (combined)")
        .with_raw(combined_scores)
        .with_color("seagreen");
    let radius = RocGroup::new("Mean radius alone")
        .with_raw(radius_scores)
        .with_color("steelblue");
    let texture = RocGroup::new("Mean texture alone")
        .with_raw(texture_scores)
        .with_color("#999999");

    let roc = RocPlot::new()
        .with_groups([combined, radius, texture])
        .with_legend("Naive classifiers");

    let plots = vec![Plot::Roc(roc)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Breast Cancer Diagnosis: Feature Discriminative Power")
        .with_x_label("False Positive Rate")
        .with_y_label("True Positive Rate")
        .with_annotation(
            TextAnnotation::new("Radius alone leads at very low FPR", 0.28, 0.42)
                .with_arrow(0.02, 0.61)
                .with_color("steelblue"),
        );

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/model_shootout.svg"), svg).unwrap();
}

/// E-commerce's share of total US retail sales, real Census Bureau data
/// (Quarterly Retail E-Commerce Sales, 1st Quarter 2026 report, adjusted
/// series), with the real reported standard error as symmetric error bars,
/// a reference line at the year-ago level, and an annotation on the most
/// recent quarter. `BarPlot` categories are positioned at 1-indexed integer
/// coordinates (1.0, 2.0, ...), not 0-indexed — confirmed by rendering and
/// checking which bar an annotation arrow actually lands on before picking
/// the final coordinates here.
fn ecommerce_share() {
    let quarters = [
        ("1Q 2025", 16.0),
        ("2Q 2025", 16.3),
        ("3Q 2025", 16.4),
        ("4Q 2025", 16.7),
        ("1Q 2026", 16.9),
    ];
    let errors: Vec<f64> = vec![0.2, 0.3, 0.3, 0.3, 0.3];

    let bar = BarPlot::new()
        .with_bars(quarters.to_vec())
        .with_color("steelblue")
        .with_error(errors)
        .with_error_color("#333");

    let plots = vec![Plot::Bar(bar)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("E-Commerce's Share of US Retail Sales")
        .with_y_label("Percent of total retail sales")
        // Zoomed in from a 14% baseline (not zero) — a 0.9-point rise is
        // real but small relative to the 0-100% scale bars conventionally
        // start at, and would be invisible otherwise. Noted in the docs
        // prose alongside the image, not just here.
        .with_y_axis_min(14.0)
        .with_y_axis_max(18.0)
        .with_reference_line(
            ReferenceLine::horizontal(16.0)
                .with_color("#555")
                .with_label("1Q 2025 level"),
        )
        .with_annotation(
            TextAnnotation::new("+0.9pp in a year", 2.3, 17.6)
                .with_arrow(5.0, 17.15)
                .with_color("steelblue"),
        );

    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::write(format!("{OUT}/ecommerce_share.svg"), svg).unwrap();
}
