//! Full-featured showcase of all 62 kuva plot types.
//! Each cell uses a larger dataset and includes a title, axis labels,
//! and a legend where applicable.
//!
//! Run with:
//!   cargo run --example all_plots_complex
//!
//! Output: docs/src/assets/overview/all_plots_complex.svg

use kuva::backend::svg::SvgBackend;
use kuva::plot::brick::BrickTemplate;
use kuva::plot::{
    BarPlot, BoxPlot, BrickPlot, BumpPlot, CalendarPlot, CandlestickPlot, ChordPlot, Clustermap,
    ContourPlot, DensityPlot, DicePlot, DotPlot, EcdfPlot, ForestPlot, FunnelPlot, GanttPlot,
    Heatmap, HexbinPlot, Histogram, Histogram2D, HorizonPlot, JointPlot, LinePlot, LollipopPlot,
    ManhattanPlot, MosaicPlot, NetworkPlot, ParallelPlot, PhyloTree, PieLabelPosition, PiePlot,
    PolarMode, PolarPlot, PopulationPyramid, PrGroup, PrPlot, QQPlot, QuiverPlot, RadarPlot,
    RaincloudPlot, RidgelinePlot, RocGroup, RocPlot, RosePlot, SankeyPlot, Scatter3DPlot,
    ScatterPlot, SeriesPlot, SlopePlot, StackedAreaPlot, StreamgraphPlot, StripPlot, SunburstPlot,
    Surface3DPlot, SurvivalPlot, SyntenyPlot, TernaryPlot, TextPlot, TreemapNode, TreemapPlot,
    UpSetPlot, VennPlot, ViolinPlot, VolcanoPlot, WafflePlot, WaterfallPlot,
};
use kuva::render::figure::Figure;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;

// ── Deterministic pseudo-random (LCG) ─────────────────────────────────────
fn lcg(seed: &mut u64) -> f64 {
    *seed = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    (*seed >> 33) as f64 / 4_294_967_296.0
}

fn logistic_dataset(n: usize, mu: f64, scale: f64) -> Vec<(f64, bool)> {
    let mut data = Vec::with_capacity(2 * n);
    for i in 1..=n {
        let p = i as f64 / (n + 1) as f64;
        let logit = (p / (1.0 - p)).ln();
        let pos = 1.0 / (1.0 + (-(mu + scale * logit)).exp());
        let neg = 1.0 / (1.0 + (-(-mu + scale * logit)).exp());
        data.push((pos, true));
        data.push((neg, false));
    }
    data
}

fn main() {
    // ── Row 0: Scatter ─────────────────────────────────────────────────────
    let mut seed = 12345u64;
    let mut scatter_pts: Vec<(f64, f64, &'static str)> = Vec::new();
    for (cx, cy, col) in [
        (2.0, 3.0, "steelblue"),
        (6.0, 6.0, "firebrick"),
        (10.0, 2.0, "forestgreen"),
    ] {
        for _ in 0..40 {
            let x = cx + (lcg(&mut seed) - 0.5) * 2.5;
            let y = cy + (lcg(&mut seed) - 0.5) * 2.5;
            scatter_pts.push((x, y, col));
        }
    }
    let sc_a = ScatterPlot::new()
        .with_data(
            scatter_pts
                .iter()
                .take(40)
                .map(|(x, y, _)| (*x, *y))
                .collect::<Vec<_>>(),
        )
        .with_color("steelblue")
        .with_legend("Group A");
    let sc_b = ScatterPlot::new()
        .with_data(
            scatter_pts
                .iter()
                .skip(40)
                .take(40)
                .map(|(x, y, _)| (*x, *y))
                .collect::<Vec<_>>(),
        )
        .with_color("firebrick")
        .with_legend("Group B");
    let sc_c = ScatterPlot::new()
        .with_data(
            scatter_pts
                .iter()
                .skip(80)
                .map(|(x, y, _)| (*x, *y))
                .collect::<Vec<_>>(),
        )
        .with_color("forestgreen")
        .with_legend("Group C");

    // ── Row 0: Line ────────────────────────────────────────────────────────
    let line_x: Vec<f64> = (0..60).map(|i| i as f64 * 0.2).collect();
    let line_a = LinePlot::new()
        .with_data(
            line_x
                .iter()
                .map(|&x| (x, 1.0 / (1.0 + (-(x - 3.0)).exp())))
                .collect::<Vec<_>>(),
        )
        .with_color("steelblue")
        .with_legend("Low");
    let line_b = LinePlot::new()
        .with_data(
            line_x
                .iter()
                .map(|&x| (x, 1.0 / (1.0 + (-(x - 6.0)).exp())))
                .collect::<Vec<_>>(),
        )
        .with_color("firebrick")
        .with_legend("Mid");
    let line_c = LinePlot::new()
        .with_data(
            line_x
                .iter()
                .map(|&x| (x, 1.0 / (1.0 + (-(x - 9.0)).exp())))
                .collect::<Vec<_>>(),
        )
        .with_color("forestgreen")
        .with_legend("High");

    // ── Row 0: Bar ─────────────────────────────────────────────────────────
    let bar = BarPlot::new()
        .with_group(
            "Control",
            vec![
                (4.2, "steelblue"),
                (3.8, "firebrick"),
                (5.1, "forestgreen"),
                (4.7, "orange"),
            ],
        )
        .with_group(
            "Drug A",
            vec![
                (6.1, "steelblue"),
                (5.9, "firebrick"),
                (7.2, "forestgreen"),
                (6.5, "orange"),
            ],
        )
        .with_group(
            "Drug B",
            vec![
                (3.5, "steelblue"),
                (4.1, "firebrick"),
                (3.2, "forestgreen"),
                (3.9, "orange"),
            ],
        )
        .with_group(
            "Combo",
            vec![
                (8.3, "steelblue"),
                (7.7, "firebrick"),
                (9.1, "forestgreen"),
                (8.0, "orange"),
            ],
        )
        .with_legend(vec!["T1", "T2", "T3", "T4"]);

    // ── Row 0: Histogram ───────────────────────────────────────────────────
    let mut hist_vals: Vec<f64> = Vec::new();
    let mut s = 99991u64;
    for _ in 0..100 {
        let u: f64 = (0..12).map(|_| lcg(&mut s)).sum::<f64>() - 6.0;
        hist_vals.push(u * 10.0 + 50.0);
    }
    let h_min = hist_vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let h_max = hist_vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let histogram = Histogram::new()
        .with_data(hist_vals)
        .with_range((h_min.floor(), h_max.ceil()))
        .with_bins(12)
        .with_color("steelblue")
        .with_legend("Count");

    // ── Row 0: Histogram2D ─────────────────────────────────────────────────
    let mut h2d_pts: Vec<(f64, f64)> = Vec::new();
    let mut s = 22222u64;
    for _ in 0..500 {
        let ux: f64 = (0..6).map(|_| lcg(&mut s)).sum::<f64>() - 3.0;
        let uy: f64 = (0..6).map(|_| lcg(&mut s)).sum::<f64>() - 3.0;
        h2d_pts.push((ux * 1.5 + 5.0, uy * 1.5 + 8.0));
    }
    for _ in 0..500 {
        let ux: f64 = (0..6).map(|_| lcg(&mut s)).sum::<f64>() - 3.0;
        let uy: f64 = (0..6).map(|_| lcg(&mut s)).sum::<f64>() - 3.0;
        h2d_pts.push((ux * 1.5 + 14.0, uy * 1.5 + 10.0));
    }
    let hist2d = Histogram2D::new().with_data(h2d_pts, (0.0, 20.0), (0.0, 20.0), 40, 40);

    // ── Row 0: Hexbin (bivariate spiral) ───────────────────────────────────
    let mut s = 11111u64;
    let hx: Vec<f64> = (0..300)
        .map(|i| {
            let t = i as f64 * 0.1;
            t.cos() * (1.0 + t * 0.1) + (lcg(&mut s) - 0.5) * 0.3
        })
        .collect();
    let hy: Vec<f64> = (0..300)
        .map(|i| {
            let t = i as f64 * 0.1;
            t.sin() * (1.0 + t * 0.1) + (lcg(&mut s) - 0.5) * 0.3
        })
        .collect();
    let hexbin = HexbinPlot::new()
        .with_data(hx, hy)
        .with_n_bins(12)
        .with_colorbar(true);

    // ── Row 1: Box/Violin/Strip ─────────────────────────────────────────────
    let group_data: Vec<(&str, Vec<f64>)> = {
        let mut s = 33333u64;
        ["Ctrl", "Low", "Mid", "High", "Max"]
            .iter()
            .enumerate()
            .map(|(gi, &label)| {
                let mean = 2.0 + gi as f64 * 1.5;
                let spread = 0.5 + gi as f64 * 0.1;
                let vals: Vec<f64> = (0..30)
                    .map(|_| {
                        let u: f64 = (0..6).map(|_| lcg(&mut s)).sum::<f64>() - 3.0;
                        mean + u * spread
                    })
                    .collect();
                (label, vals)
            })
            .collect()
    };
    let box_plot = group_data
        .iter()
        .fold(BoxPlot::new(), |b, (lbl, v)| b.with_group(*lbl, v.clone()));
    let violin = group_data.iter().fold(ViolinPlot::new(), |v, (lbl, d)| {
        v.with_group(*lbl, d.clone())
    });
    let strip = group_data
        .iter()
        .fold(StripPlot::new().with_color("steelblue"), |st, (lbl, d)| {
            st.with_group(*lbl, d.clone())
        });

    // ── Row 1: Waterfall ───────────────────────────────────────────────────
    let waterfall = WaterfallPlot::new()
        .with_delta("Revenue", 520.0)
        .with_delta("COGS", -180.0)
        .with_delta("Gross Profit", 340.0)
        .with_delta("R&D", -90.0)
        .with_delta("S&M", -70.0)
        .with_delta("G&A", -40.0)
        .with_delta("Other", 15.0)
        .with_total("EBIT")
        .with_connectors()
        .with_values();

    // ── Row 1: StackedArea ─────────────────────────────────────────────────
    let sa_x: Vec<f64> = (0..10).map(|i| 2015.0 + i as f64).collect();
    let stacked_area = StackedAreaPlot::new()
        .with_x(sa_x)
        .with_series([
            120.0, 135.0, 148.0, 162.0, 175.0, 190.0, 205.0, 220.0, 238.0, 255.0,
        ])
        .with_color("#4e79a7")
        .with_legend("Alpha")
        .with_series([
            85.0, 92.0, 98.0, 105.0, 112.0, 118.0, 125.0, 133.0, 141.0, 150.0,
        ])
        .with_color("#f28e2b")
        .with_legend("Beta")
        .with_series([
            60.0, 65.0, 71.0, 78.0, 83.0, 90.0, 96.0, 103.0, 110.0, 118.0,
        ])
        .with_color("#59a14f")
        .with_legend("Gamma")
        .with_series([40.0, 45.0, 50.0, 56.0, 62.0, 68.0, 75.0, 82.0, 89.0, 97.0])
        .with_color("#e15759")
        .with_legend("Delta");

    // ── Row 1: Streamgraph (5 series × 12 months) ─────────────────────────
    let stream_x: Vec<f64> = (1..=12).map(|m| m as f64).collect();
    let streamgraph = StreamgraphPlot::new()
        .with_x(stream_x)
        .with_series([
            20.0, 28.0, 35.0, 42.0, 50.0, 55.0, 52.0, 45.0, 38.0, 30.0, 22.0, 18.0,
        ])
        .with_color("#4e79a7")
        .with_label("Spring")
        .with_series([
            10.0, 12.0, 20.0, 30.0, 40.0, 48.0, 55.0, 50.0, 42.0, 32.0, 18.0, 12.0,
        ])
        .with_color("#f28e2b")
        .with_label("Summer")
        .with_series([
            35.0, 30.0, 22.0, 15.0, 10.0, 8.0, 10.0, 14.0, 22.0, 32.0, 40.0, 44.0,
        ])
        .with_color("#59a14f")
        .with_label("Autumn")
        .with_series([
            50.0, 42.0, 30.0, 18.0, 10.0, 5.0, 6.0, 8.0, 14.0, 25.0, 38.0, 48.0,
        ])
        .with_color("#e15759")
        .with_label("Winter")
        .with_legend("Season");

    // ── Row 2: Pie ─────────────────────────────────────────────────────────
    let pie = PiePlot::new()
        .with_slice("Alpha", 32.0, "#4e79a7")
        .with_slice("Beta", 21.0, "#f28e2b")
        .with_slice("Gamma", 17.0, "#59a14f")
        .with_slice("Delta", 14.0, "#e15759")
        .with_slice("Epsilon", 9.0, "#76b7b2")
        .with_slice("Zeta", 7.0, "#edc948")
        .with_label_position(PieLabelPosition::Outside)
        .with_percent();

    // ── Row 2: Series ──────────────────────────────────────────────────────
    let ser_x: Vec<f64> = (0..80).map(|i| i as f64).collect();
    let ser1 = SeriesPlot::new()
        .with_data(ser_x.iter().map(|&x| (x * 0.1).sin()).collect::<Vec<_>>())
        .with_color("steelblue")
        .with_line_point_style()
        .with_legend("sin(0.1x)");
    let ser2 = SeriesPlot::new()
        .with_data(ser_x.iter().map(|&x| (x * 0.15).cos()).collect::<Vec<_>>())
        .with_color("firebrick")
        .with_line_point_style()
        .with_legend("cos(0.15x)");
    let ser3 = SeriesPlot::new()
        .with_data(
            ser_x
                .iter()
                .map(|&x| (x * 0.1).sin() * (x * 0.05).cos())
                .collect::<Vec<_>>(),
        )
        .with_color("forestgreen")
        .with_line_point_style()
        .with_legend("product");

    // ── Row 2: Band ────────────────────────────────────────────────────────
    let band_x: Vec<f64> = (0..50).map(|i| i as f64 * 0.2).collect();
    let band_y: Vec<f64> = band_x.iter().map(|&x| x.sin()).collect();
    let band_lo: Vec<f64> = band_y.iter().map(|&y| y - 0.4).collect();
    let band_hi: Vec<f64> = band_y.iter().map(|&y| y + 0.4).collect();
    let band_line = LinePlot::new()
        .with_data(
            band_x
                .iter()
                .zip(band_y.iter())
                .map(|(&x, &y)| (x, y))
                .collect::<Vec<_>>(),
        )
        .with_color("steelblue")
        .with_band(band_lo, band_hi)
        .with_legend("sin(x) ± 0.4");

    // ── Row 2: Heatmap ─────────────────────────────────────────────────────
    let genes = ["BRCA1", "TP53", "EGFR", "MYC", "KRAS", "PTEN"];
    let heat_matrix = vec![
        vec![1.00, 0.72, 0.31, -0.45, 0.18, -0.62],
        vec![0.72, 1.00, 0.55, -0.23, 0.41, -0.38],
        vec![0.31, 0.55, 1.00, 0.08, 0.63, -0.11],
        vec![-0.45, -0.23, 0.08, 1.00, -0.29, 0.74],
        vec![0.18, 0.41, 0.63, -0.29, 1.00, 0.05],
        vec![-0.62, -0.38, -0.11, 0.74, 0.05, 1.00],
    ];
    let hmap = Heatmap::new().with_data(heat_matrix).with_labels(
        genes.iter().map(|s| s.to_string()).collect(),
        genes.iter().map(|s| s.to_string()).collect(),
    );

    // ── Row 2: DotPlot ─────────────────────────────────────────────────────
    let pathways = ["PI3K", "MAPK", "WNT", "Notch"];
    let cell_types = ["T cell", "B cell", "NK", "Macrophage"];
    let sizes = [
        [8.0, 14.0, 5.0, 12.0],
        [11.0, 7.0, 16.0, 9.0],
        [6.0, 13.0, 10.0, 15.0],
        [14.0, 5.0, 12.0, 7.0],
    ];
    let colors = [
        [0.8, 1.5, 0.3, 1.2],
        [1.1, 0.7, 2.0, 0.9],
        [0.5, 1.4, 1.0, 1.7],
        [1.6, 0.4, 1.3, 0.6],
    ];
    let dot_data: Vec<(&str, &str, f64, f64)> = pathways
        .iter()
        .enumerate()
        .flat_map(|(pi, &path)| {
            cell_types
                .iter()
                .enumerate()
                .map(move |(ci, &ct)| (path, ct, sizes[pi][ci], colors[pi][ci]))
        })
        .collect();
    let dot = DotPlot::new()
        .with_data(dot_data)
        .with_size_legend("% expressed")
        .with_colorbar("mean expr");

    // ── Row 2: Clustermap ─────────────────────────────────────────────────
    let clustermap = Clustermap::new()
        .with_data(vec![
            vec![0.95_f64, 0.88, 0.12, 0.08, 0.05],
            vec![0.85_f64, 0.91, 0.15, 0.10, 0.09],
            vec![0.10_f64, 0.13, 0.90, 0.87, 0.82],
            vec![0.08_f64, 0.11, 0.85, 0.92, 0.88],
            vec![0.06_f64, 0.09, 0.80, 0.84, 0.93],
        ])
        .with_row_labels(["GeneA", "GeneB", "GeneC", "GeneD", "GeneE"])
        .with_col_labels(["S1", "S2", "S3", "S4", "S5"]);

    // ── Row 3: Volcano ─────────────────────────────────────────────────────
    let volcano_data = [
        ("BRCA1", 2.5, 0.001),
        ("TP53", 1.8, 0.01),
        ("EGFR", 3.2, 0.0001),
        ("MYC", 1.5, 0.03),
        ("KRAS", 2.1, 0.005),
        ("CDK4", 1.2, 0.04),
        ("PTEN", 2.8, 0.002),
        ("RB1", 1.9, 0.008),
        ("AKT1", 3.5, 0.00005),
        ("VEGFA", 2.3, 0.003),
        ("CDKN2A", -2.3, 0.002),
        ("SMAD4", -1.9, 0.008),
        ("VHL", -3.0, 0.0005),
        ("CASP3", -1.6, 0.04),
        ("BCL2", -2.7, 0.001),
        ("FAS", -1.4, 0.035),
        ("PUMA", -2.0, 0.007),
        ("BAX", -1.7, 0.015),
        ("P21", -3.2, 0.0002),
        ("MDM2", -2.5, 0.003),
        ("GAPDH", 0.3, 0.5),
        ("ACTB", -0.5, 0.3),
        ("TUBA1", 0.8, 0.1),
        ("HIST1", -0.2, 0.7),
        ("RPL5", 0.6, 0.2),
        ("RPS6", -0.9, 0.15),
        ("EEF1A", 0.1, 0.8),
        ("HNRNPA", -0.7, 0.4),
        ("SF3B1", 0.4, 0.6),
        ("SRSF1", -0.3, 0.9),
        ("GeneA", 1.5, 0.2),
        ("GeneB", -1.1, 0.07),
        ("GeneC", 0.9, 0.12),
        ("GeneD", -0.8, 0.08),
        ("GeneE", 1.3, 0.18),
    ];
    let volcano = VolcanoPlot::new()
        .with_points(volcano_data)
        .with_label_top(5);

    // ── Row 3: Manhattan ───────────────────────────────────────────────────
    let mut man_data: Vec<(&str, f64)> = Vec::new();
    let mut s = 77777u64;
    for (chrom, has_signal) in [("1", true), ("2", false), ("3", true)] {
        for i in 0..80usize {
            let r = lcg(&mut s);
            let p = if has_signal && i > 35 && i < 42 {
                r * 5e-9
            } else {
                0.05 + r * 0.85
            };
            man_data.push((chrom, p));
        }
    }
    let manhattan = ManhattanPlot::new()
        .with_data(man_data)
        .with_genome_wide(7.3)
        .with_suggestive(5.0)
        .with_label_top(3);

    // ── Row 3: Candlestick ─────────────────────────────────────────────────
    let ohlc = [
        ("Jan", 98.0, 108.0, 95.0, 105.0),
        ("Feb", 105.0, 118.0, 103.0, 115.0),
        ("Mar", 115.0, 117.0, 106.0, 109.0),
        ("Apr", 109.0, 112.0, 100.0, 103.0),
        ("May", 103.0, 114.0, 101.0, 112.0),
        ("Jun", 112.0, 125.0, 110.0, 122.0),
        ("Jul", 122.0, 128.0, 118.0, 120.0),
        ("Aug", 120.0, 122.0, 110.0, 113.0),
        ("Sep", 113.0, 120.0, 111.0, 118.0),
        ("Oct", 118.0, 130.0, 116.0, 127.0),
    ];
    let candle = ohlc
        .iter()
        .fold(CandlestickPlot::new(), |c, &(l, o, h, lo, cl)| {
            c.with_candle(l, o, h, lo, cl)
        });

    // ── Row 3: Contour ─────────────────────────────────────────────────────
    let n = 8usize;
    let coords: Vec<f64> = (0..n)
        .map(|i| -2.0 + i as f64 / (n as f64 - 1.0) * 4.0)
        .collect();
    let contour_pts: Vec<(f64, f64, f64)> = coords
        .iter()
        .flat_map(|&y| {
            coords.iter().map(move |&x| {
                let z = (-(x * x + y * y) / 1.5).exp()
                    + 0.5 * (-((x - 1.0).powi(2) + (y + 1.0).powi(2)) / 0.5).exp();
                (x, y, z)
            })
        })
        .collect();
    let contour = ContourPlot::new()
        .with_points(contour_pts)
        .with_filled()
        .with_n_levels(8)
        .with_legend("Density");

    // ── Row 3: UpSet ───────────────────────────────────────────────────────
    let upset = UpSetPlot::new().with_data(
        ["A", "B", "C", "D"],
        [60_usize, 50, 45, 35],
        [
            (1u64, 28),
            (2u64, 22),
            (4u64, 18),
            (8u64, 12),
            (3u64, 15),
            (5u64, 11),
            (9u64, 9),
            (6u64, 8),
            (12u64, 6),
            (7u64, 5),
            (14u64, 4),
            (15u64, 3),
        ],
    );

    // ── Row 3: Network ─────────────────────────────────────────────────────
    let network = NetworkPlot::new()
        .with_node_color("Hub", "#4e79a7")
        .with_node_color("Alpha", "#f28e2b")
        .with_node_color("Beta", "#59a14f")
        .with_node_color("Gamma", "#e15759")
        .with_node_color("Delta", "#76b7b2")
        .with_node_color("Epsilon", "#edc948")
        .with_edge("Hub", "Alpha", 3.0)
        .with_edge("Hub", "Beta", 2.0)
        .with_edge("Hub", "Gamma", 2.5)
        .with_edge("Hub", "Delta", 1.5)
        .with_edge("Hub", "Epsilon", 2.0)
        .with_edge("Alpha", "Beta", 1.0)
        .with_edge("Gamma", "Delta", 1.2);

    // ── Row 4: Chord ───────────────────────────────────────────────────────
    let chord = ChordPlot::new()
        .with_matrix(vec![
            vec![0.0, 120.0, 80.0, 50.0],
            vec![120.0, 0.0, 95.0, 40.0],
            vec![80.0, 95.0, 0.0, 70.0],
            vec![50.0, 40.0, 70.0, 0.0],
        ])
        .with_labels(["CD4 T", "CD8 T", "NK", "B cell"])
        .with_legend("Cell types");

    // ── Row 4: Sankey ──────────────────────────────────────────────────────
    let sankey = SankeyPlot::new()
        .with_node_color("Reads", "#4e79a7")
        .with_node_color("Mapped", "#59a14f")
        .with_node_color("Unmapped", "#e15759")
        .with_node_color("Exonic", "#f28e2b")
        .with_node_color("Intronic", "#76b7b2")
        .with_link("Reads", "Mapped", 850.0)
        .with_link("Reads", "Unmapped", 150.0)
        .with_link("Mapped", "Exonic", 620.0)
        .with_link("Mapped", "Intronic", 230.0)
        .with_legend("RNA-seq flow");

    // ── Row 4: PhyloTree ───────────────────────────────────────────────────
    let phylo = PhyloTree::from_newick(
        "(((Homo:0.01,Pan:0.01):0.05,Gorilla:0.06):0.15,\
          ((Pongo:0.08,Hylobates:0.08):0.1,\
           ((Macaca:0.2,Papio:0.2):0.1,Colobus:0.3):0.05):0.1);",
    );

    // ── Row 4: Synteny ─────────────────────────────────────────────────────
    let mut synteny = SyntenyPlot::new().with_sequences([
        ("Chr 1", 248_956_422.0_f64),
        ("Chr 2", 242_193_529.0_f64),
        ("Chr 3", 198_295_559.0_f64),
        ("Chr 4", 190_214_555.0_f64),
        ("Chr 5", 181_538_259.0_f64),
        ("Chr 6", 170_805_979.0_f64),
    ]);
    let pairs: [(usize, usize); 5] = [(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)];
    let fwd = [
        (0.0_f64, 35e6_f64, 0.0_f64, 33e6_f64),
        (45e6, 83e6, 43e6, 80e6),
        (95e6, 128e6, 93e6, 125e6),
        (140e6, 165e6, 138e6, 163e6),
    ];
    for (s1, s2) in pairs {
        for (a, b, c, d) in fwd {
            synteny = synteny.with_block(s1, a, b, s2, c, d);
        }
    }
    synteny = synteny
        .with_inv_block(0, 37e6, 43e6, 1, 35e6, 41e6)
        .with_inv_block(2, 87e6, 93e6, 3, 85e6, 91e6);

    // ── Row 4: Brick ───────────────────────────────────────────────────────
    let seqs = [
        "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCAT",
        "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCATCAT",
        "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCAT",
        "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCATCATCAT",
        "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCAT",
        "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCATCAT",
        "CGGCGATCAGGCCGCACTCATCATCATCATCATCAT",
        "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCATCATCATCAT",
    ];
    let tmpl = BrickTemplate::new().dna();
    let brick = BrickPlot::new()
        .with_sequences(seqs)
        .with_names(["r1", "r2", "r3", "r4", "r5", "r6", "r7", "r8"])
        .with_template(tmpl.template)
        .with_x_offset(18.0);

    // ── Row 4: Treemap ─────────────────────────────────────────────────────
    let treemap = TreemapPlot::new()
        .with_children(
            "Technology",
            vec![
                TreemapNode::leaf("Software", 42.0),
                TreemapNode::leaf("Hardware", 28.0),
                TreemapNode::leaf("Cloud", 35.0),
            ],
        )
        .with_children(
            "Healthcare",
            vec![
                TreemapNode::leaf("Pharma", 31.0),
                TreemapNode::leaf("Biotech", 24.0),
                TreemapNode::leaf("Devices", 18.0),
            ],
        )
        .with_children(
            "Energy",
            vec![
                TreemapNode::leaf("Renewables", 22.0),
                TreemapNode::leaf("Oil & Gas", 19.0),
            ],
        );

    // ── Row 5: Density ─────────────────────────────────────────────────────
    let dens_a = DensityPlot::new()
        .with_data({
            let mut s = 55555u64;
            (0..80)
                .map(|_| {
                    let u: f64 = (0..6).map(|_| lcg(&mut s)).sum::<f64>() - 3.0;
                    3.0 + u * 0.8
                })
                .collect::<Vec<_>>()
        })
        .with_color("steelblue")
        .with_filled(true)
        .with_opacity(0.5)
        .with_legend("Low");
    let dens_b = DensityPlot::new()
        .with_data({
            let mut s = 66666u64;
            (0..80)
                .map(|_| {
                    let u: f64 = (0..6).map(|_| lcg(&mut s)).sum::<f64>() - 3.0;
                    6.0 + u * 0.8
                })
                .collect::<Vec<_>>()
        })
        .with_color("firebrick")
        .with_filled(true)
        .with_opacity(0.5)
        .with_legend("High");

    // ── Row 5: Ridgeline ───────────────────────────────────────────────────
    let ridgeline = {
        let mut s = 44444u64;
        ["Winter", "Spring", "Summer", "Autumn"]
            .iter()
            .enumerate()
            .fold(RidgelinePlot::new(), |r, (i, &season)| {
                let mean = 5.0 + i as f64 * 9.0;
                let data: Vec<f64> = (0..40)
                    .map(|_| {
                        let u: f64 = (0..6).map(|_| lcg(&mut s)).sum::<f64>() - 3.0;
                        mean + u * 3.5
                    })
                    .collect();
                r.with_group(season, data)
            })
    };

    // ── Row 5: Polar ───────────────────────────────────────────────────────
    let n_polar = 72usize;
    let theta_car: Vec<f64> = (0..n_polar)
        .map(|i| i as f64 * 360.0 / n_polar as f64)
        .collect();
    let r_car: Vec<f64> = theta_car
        .iter()
        .map(|&t| 1.0 + t.to_radians().cos())
        .collect();
    let theta_circ: Vec<f64> = (0..=n_polar)
        .map(|i| i as f64 * 360.0 / n_polar as f64)
        .collect();
    let r_circ: Vec<f64> = vec![1.0; theta_circ.len()];
    let polar = PolarPlot::new()
        .with_series_labeled(r_car, theta_car, "Cardioid", PolarMode::Line)
        .with_series_labeled(r_circ, theta_circ, "Unit circle", PolarMode::Line)
        .with_r_max(2.1)
        .with_r_grid_lines(4)
        .with_theta_divisions(12)
        .with_legend(true);

    // ── Row 5: Ternary ─────────────────────────────────────────────────────
    let ternary = {
        let mut s = 88888u64;
        let mut plot = TernaryPlot::new()
            .with_corner_labels("A", "B", "C")
            .with_grid_lines(4)
            .with_legend(true);
        for (centre, label) in [
            ([0.78_f64, 0.13, 0.09], "A-rich"),
            ([0.11_f64, 0.78, 0.11], "B-rich"),
            ([0.10_f64, 0.11, 0.79], "C-rich"),
        ] {
            for _ in 0..8usize {
                let da = (lcg(&mut s) - 0.5) * 0.08;
                let db = (lcg(&mut s) - 0.5) * 0.08;
                let a = (centre[0] + da).clamp(0.01, 0.98);
                let b = (centre[1] + db).clamp(0.01, 0.98);
                let c = (1.0 - a - b).clamp(0.01, 0.98);
                let total = a + b + c;
                plot = plot.with_point_group(a / total, b / total, c / total, label);
            }
        }
        plot
    };

    // ── Row 5: Forest ──────────────────────────────────────────────────────
    let forest = ForestPlot::new()
        .with_weighted_row("Smith 2018", 0.82, 0.55, 1.18, 3.2)
        .with_weighted_row("Jones 2019", 1.15, 0.72, 1.61, 4.1)
        .with_weighted_row("Lee 2020", 0.63, 0.31, 0.96, 2.8)
        .with_weighted_row("Patel 2021", 1.28, 0.89, 1.74, 3.9)
        .with_weighted_row("Garcia 2021", 0.74, 0.44, 1.08, 3.5)
        .with_weighted_row("Kim 2022", 1.03, 0.68, 1.45, 4.6)
        .with_weighted_row("Muller 2022", 0.91, 0.60, 1.26, 5.1)
        .with_weighted_row("Chen 2023", 1.09, 0.75, 1.51, 4.8)
        .with_row("Pooled", 0.94, 0.77, 1.11)
        .with_null_value(1.0)
        .with_show_null_line(true);

    // ── Row 5: ECDF ────────────────────────────────────────────────────────
    let ecdf = {
        let mut s = 12121u64;
        EcdfPlot::new()
            .with_data_colored(
                "Low dose",
                {
                    (0..50)
                        .map(|_| {
                            let u: f64 = (0..6).map(|_| lcg(&mut s)).sum::<f64>() - 3.0;
                            4.0 + u * 1.2
                        })
                        .collect::<Vec<_>>()
                },
                "steelblue",
            )
            .with_data_colored(
                "High dose",
                {
                    (0..50)
                        .map(|_| {
                            let u: f64 = (0..6).map(|_| lcg(&mut s)).sum::<f64>() - 3.0;
                            7.0 + u * 1.0
                        })
                        .collect::<Vec<_>>()
                },
                "firebrick",
            )
            .with_confidence_band()
    };

    // ── Row 6: ROC ─────────────────────────────────────────────────────────
    let roc = RocPlot::new()
        .with_group(
            RocGroup::new("Model A")
                .with_raw(logistic_dataset(150, 1.2, 0.5))
                .with_ci(true)
                .with_optimal_point(),
        )
        .with_group(
            RocGroup::new("Model B")
                .with_raw(logistic_dataset(150, 0.6, 0.5))
                .with_ci(true),
        )
        .with_legend("Classifier");

    // ── Row 6: PR ──────────────────────────────────────────────────────────
    let pr = PrPlot::new()
        .with_group(
            PrGroup::new("Model A")
                .with_raw(logistic_dataset(150, 1.2, 0.5))
                .with_color("steelblue"),
        )
        .with_group(
            PrGroup::new("Model B")
                .with_raw(logistic_dataset(150, 0.6, 0.5))
                .with_color("firebrick"),
        )
        .with_baseline(true);

    // ── Row 6: Survival ────────────────────────────────────────────────────
    let survival = SurvivalPlot::new()
        .with_group(
            "Stage I",
            vec![10.0, 14.0, 20.0, 26.0, 32.0, 36.0, 40.0, 44.0, 48.0, 52.0],
            vec![
                false, true, false, false, true, false, false, true, false, false,
            ],
        )
        .with_group(
            "Stage II",
            vec![5.0, 8.0, 12.0, 16.0, 20.0, 24.0, 28.0, 32.0, 36.0, 40.0],
            vec![
                true, true, false, true, false, true, false, false, true, false,
            ],
        )
        .with_group(
            "Stage III",
            vec![2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0],
            vec![
                true, true, true, false, true, true, false, true, false, true,
            ],
        )
        .with_pvalue_text("log-rank p < 0.001")
        .with_legend("Stage");

    // ── Row 6: DicePlot ────────────────────────────────────────────────────
    let dice = DicePlot::new(4)
        .with_category_labels(
            ["Lung", "Liver", "Brain", "Kidney"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
        )
        .with_points([
            ("miR-21", "Drug A", vec![0, 1, 3], None, None),
            ("miR-21", "Drug B", vec![1, 2], None, None),
            ("miR-155", "Drug A", vec![0, 2, 3], None, None),
            ("miR-155", "Drug B", vec![2], None, None),
            ("miR-34", "Drug A", vec![0, 1, 2], None, None),
            ("miR-34", "Drug B", vec![0, 3], None, None),
        ]);

    // ── Row 6: Raincloud ───────────────────────────────────────────────────
    let raincloud = RaincloudPlot::new()
        .with_group("Control", (0..20).map(|i| 4.0 + i as f64 * 0.15).collect())
        .with_group("Low dose", (0..20).map(|i| 5.5 + i as f64 * 0.18).collect())
        .with_group(
            "High dose",
            (0..20).map(|i| 7.2 + i as f64 * 0.12).collect(),
        );

    // ── Row 6: Lollipop ────────────────────────────────────────────────────
    let lollipop = LollipopPlot::new()
        .with_domain_opacity(0.0, 2.0, Some("Transactivation"), "steelblue", 0.35)
        .with_domain_opacity(2.0, 5.0, Some("DNA-binding"), "firebrick", 0.35)
        .with_domain_opacity(5.0, 7.0, Some("Tetramerization"), "forestgreen", 0.35)
        .with_colored_point(1.2, 4.0, "steelblue")
        .with_colored_point(2.5, 7.0, "firebrick")
        .with_colored_point(3.1, 3.0, "firebrick")
        .with_colored_point(3.8, 5.0, "firebrick")
        .with_colored_point(4.6, 6.0, "firebrick")
        .with_colored_point(5.5, 2.0, "forestgreen")
        .with_colored_point(6.2, 4.0, "forestgreen");

    // ── Row 7: JointPlot ───────────────────────────────────────────────────
    let joint = JointPlot::new()
        .with_group(
            "Group A",
            (0..30).map(|i| 2.0 + i as f64 * 0.08),
            (0..30).map(|i| 2.5 + (i as f64 * 0.2).sin() * 0.8 + i as f64 * 0.05),
            "steelblue",
        )
        .with_group(
            "Group B",
            (0..30).map(|i| 3.5 + i as f64 * 0.07),
            (0..30).map(|i| 4.0 + (i as f64 * 0.15).cos() * 0.6 + i as f64 * 0.03),
            "firebrick",
        )
        .with_x_label("Feature x")
        .with_y_label("Feature y");

    // ── Row 7: Waffle ──────────────────────────────────────────────────────
    let waffle = WafflePlot::new()
        .with_category("Coalition A", 45.0, "#4e79a7")
        .with_category("Coalition B", 38.0, "#e15759")
        .with_category("Coalition C", 12.0, "#59a14f")
        .with_category("Independent", 5.0, "#f28e2b")
        .with_grid(10, 10)
        .with_legend("Seat share");

    // ── Row 7: Horizon ─────────────────────────────────────────────────────
    let horizon = HorizonPlot::new()
        .with_series(
            "Alpha",
            (0..12).map(|i| i as f64),
            [
                1.2, 3.5, -0.8, 4.1, 2.3, -1.5, 3.8, 1.0, -2.2, 4.5, 0.7, -3.1_f64,
            ],
        )
        .with_series(
            "Beta",
            (0..12).map(|i| i as f64),
            [
                -0.5, 2.1, 4.3, -1.2, 3.0, 2.8, -2.5, 3.3, 1.5, -0.8, 3.7, 2.0_f64,
            ],
        )
        .with_series(
            "Gamma",
            (0..12).map(|i| i as f64),
            [
                2.8, 0.5, 3.2, -1.8, 1.0, -3.0, 2.2, 4.0, -0.5, 2.5, -1.3, 3.5_f64,
            ],
        )
        .with_n_bands(3);

    // ── Row 7: Pyramid ─────────────────────────────────────────────────────
    let pyramid = PopulationPyramid::new()
        .with_group("0–9", 9.2, 8.8)
        .with_group("10–19", 9.8, 9.4)
        .with_group("20–29", 10.5, 10.1)
        .with_group("30–39", 11.2, 10.9)
        .with_group("40–49", 10.8, 10.5)
        .with_group("50–59", 9.5, 9.8)
        .with_group("60–69", 7.8, 8.4)
        .with_group("70+", 5.1, 6.9)
        .with_left_label("Male")
        .with_right_label("Female");

    // ── Row 7: Mosaic ──────────────────────────────────────────────────────
    let mosaic = MosaicPlot::new()
        .with_cell("Placebo", "Improved", 25.0)
        .with_cell("Placebo", "No change", 40.0)
        .with_cell("Placebo", "Worsened", 15.0)
        .with_cell("Drug A", "Improved", 48.0)
        .with_cell("Drug A", "No change", 20.0)
        .with_cell("Drug A", "Worsened", 8.0)
        .with_cell("Drug B", "Improved", 38.0)
        .with_cell("Drug B", "No change", 28.0)
        .with_cell("Drug B", "Worsened", 12.0);

    // ── Row 7: Slope ───────────────────────────────────────────────────────
    let slope = SlopePlot::new()
        .with_point("North", 4.2, 6.8)
        .with_point("South", 5.9, 4.3)
        .with_point("East", 3.1, 5.7)
        .with_point("West", 7.4, 8.2)
        .with_point("Centre", 6.0, 5.5)
        .with_point("Coast", 2.8, 4.1)
        .with_before_label("2022")
        .with_after_label("2023")
        .with_direction_colors(true);

    // ── Row 8: Venn ────────────────────────────────────────────────────────
    let venn = VennPlot::new()
        .with_set_size("Proteomics", 120)
        .with_set_size("Genomics", 150)
        .with_set_size("Metabolomics", 90)
        .with_overlap(["Proteomics", "Genomics"], 45)
        .with_overlap(["Proteomics", "Metabolomics"], 28)
        .with_overlap(["Genomics", "Metabolomics"], 35)
        .with_overlap(["Proteomics", "Genomics", "Metabolomics"], 18)
        .with_percentages(false);

    // ── Row 8: Parallel ────────────────────────────────────────────────────
    let parallel = {
        let mut s = 54321u64;
        let mut plot = ParallelPlot::new()
            .with_axis_names(["Sepal L", "Sepal W", "Petal L", "Petal W"])
            .with_curved(true);
        for (name, means) in [
            ("Setosa", [5.0, 3.4, 1.5, 0.2_f64]),
            ("Versicolor", [5.9, 2.8, 4.3, 1.3_f64]),
            ("Virginica", [6.6, 3.0, 5.6, 2.0_f64]),
        ] {
            for _ in 0..15usize {
                let row: Vec<f64> = means
                    .iter()
                    .map(|&m| {
                        let u: f64 = (0..4).map(|_| lcg(&mut s)).sum::<f64>() - 2.0;
                        (m + u * 0.3).max(0.1)
                    })
                    .collect();
                plot = plot.with_row_group(name, row);
            }
        }
        plot
    };

    // ── Row 8: Radar ───────────────────────────────────────────────────────
    let radar = RadarPlot::new(["Speed", "Power", "Agility", "Stamina", "Technique"])
        .with_series_color([8.5, 7.0, 9.2, 6.5, 7.8], "Warrior", "steelblue")
        .with_series_color([5.0, 9.5, 4.5, 8.0, 9.8], "Mage", "firebrick")
        .with_series_color([9.0, 5.5, 8.8, 7.0, 6.0], "Rogue", "forestgreen")
        .with_filled(true)
        .with_grid_lines(5);

    // ── Row 8: Rose (12 sectors, 2 stacked wind-speed bands) ──────────────
    let rose = RosePlot::new()
        .with_stack(
            "Light (0–5 m/s)",
            [
                8.0_f64, 18.0, 14.0, 7.0, 5.0, 9.0, 16.0, 10.0, 6.0, 4.0, 7.0, 12.0,
            ],
        )
        .with_stack(
            "Strong (>5 m/s)",
            [
                4.0_f64, 12.0, 6.0, 2.0, 1.0, 3.0, 8.0, 5.0, 3.0, 2.0, 4.0, 7.0,
            ],
        )
        .with_compass_labels()
        .with_grid_lines(4)
        .with_legend("Wind speed");

    // ── Row 8: Sunburst ────────────────────────────────────────────────────
    let sunburst = SunburstPlot::new().with_children(
        "Life",
        vec![
            TreemapNode::new(
                "Animals",
                vec![
                    TreemapNode::leaf("Mammals", 42.0),
                    TreemapNode::leaf("Birds", 28.0),
                    TreemapNode::leaf("Reptiles", 18.0),
                ],
            ),
            TreemapNode::new(
                "Plants",
                vec![
                    TreemapNode::leaf("Flowering", 55.0),
                    TreemapNode::leaf("Ferns", 20.0),
                ],
            ),
        ],
    );

    // ── Row 8: Bump ────────────────────────────────────────────────────────
    let bump = BumpPlot::new()
        .with_series("Alpha", [2.0_f64, 1.0, 1.0, 2.0, 3.0])
        .with_series("Beta", [1.0_f64, 2.0, 3.0, 1.0, 1.0])
        .with_series("Gamma", [3.0_f64, 3.0, 2.0, 3.0, 2.0])
        .with_series("Delta", [4.0_f64, 5.0, 4.0, 4.0, 5.0])
        .with_series("Epsilon", [5.0_f64, 4.0, 5.0, 5.0, 4.0])
        .with_x_labels(["Q1", "Q2", "Q3", "Q4", "Q5"]);

    // ── Row 9: QQ ──────────────────────────────────────────────────────────
    let qq = {
        let mut s = 99999u64;
        QQPlot::new()
            .with_data_colored(
                "Normal",
                {
                    (0..60)
                        .map(|_| {
                            let u: f64 = (0..12).map(|_| lcg(&mut s)).sum::<f64>() - 6.0;
                            u * 2.0 + 10.0
                        })
                        .collect::<Vec<_>>()
                },
                "steelblue",
            )
            .with_data_colored(
                "Heavy-tailed",
                {
                    (0..60)
                        .map(|i| {
                            let t = (i as f64 / 59.0 - 0.5) * 10.0;
                            t * t.abs().sqrt().signum() * 1.5 + 10.0
                        })
                        .collect::<Vec<_>>()
                },
                "firebrick",
            )
            .with_ci_band()
    };

    // ── Row 9: Scatter3D ───────────────────────────────────────────────────
    let scatter3d = {
        let mut s = 31415u64;
        let sc_a = Scatter3DPlot::new()
            .with_data((0..30).map(|_| {
                let u: f64 = (0..4).map(|_| lcg(&mut s)).sum::<f64>() - 2.0;
                let v: f64 = (0..4).map(|_| lcg(&mut s)).sum::<f64>() - 2.0;
                let w: f64 = (0..4).map(|_| lcg(&mut s)).sum::<f64>() - 2.0;
                (u + 2.0, v + 2.0, w + 2.0)
            }))
            .with_color("steelblue")
            .with_legend("Cluster A");
        let sc_b = Scatter3DPlot::new()
            .with_data((0..30).map(|_| {
                let u: f64 = (0..4).map(|_| lcg(&mut s)).sum::<f64>() - 2.0;
                let v: f64 = (0..4).map(|_| lcg(&mut s)).sum::<f64>() - 2.0;
                let w: f64 = (0..4).map(|_| lcg(&mut s)).sum::<f64>() - 2.0;
                (u - 2.0, v - 2.0, w)
            }))
            .with_color("firebrick")
            .with_legend("Cluster B");
        (sc_a, sc_b)
    };

    // ── Row 9: Surface3D ───────────────────────────────────────────────────
    let surface3d = Surface3DPlot::new(vec![]).with_data_fn(
        |x: f64, y: f64| {
            let r = (x * x + y * y).sqrt().max(0.001);
            r.sin() / r
        },
        -6.0_f64..=6.0,
        -6.0_f64..=6.0,
        30,
        30,
    );

    // ── Row 9: Calendar (Jan–Jun 2025 — ~27 cols fits 600px cell) ──────────
    let calendar = {
        let mut s = 24680u64;
        let mut cal = CalendarPlot::new().with_date_range("2025-01-01", "2025-06-30");
        for month in 1..=6u32 {
            let days = match month {
                2 => 28,
                4 | 6 => 30,
                _ => 31,
            };
            for day in (1..=days).step_by(3) {
                let val = lcg(&mut s) * 10.0;
                let date = format!("2025-{:02}-{:02}", month, day);
                cal = cal.with_data(std::iter::once((date, val)));
            }
        }
        cal
    };

    // ── Row 9: Funnel ──────────────────────────────────────────────────────
    let funnel = FunnelPlot::new()
        .with_stage("Impressions", 50_000.0)
        .with_stage("Clicks", 8_500.0)
        .with_stage("Signups", 1_200.0)
        .with_stage("Trials", 380.0)
        .with_stage("Paid customers", 105.0);

    // ── Row 9: Gantt ───────────────────────────────────────────────────────
    let gantt = GanttPlot::new()
        .with_task_group("Discovery", "Research", 0.0, 3.0)
        .with_task_group("Discovery", "Prototyping", 2.0, 5.0)
        .with_task_group("Build", "Backend", 4.0, 9.0)
        .with_task_group_progress("Build", "Frontend", 5.0, 10.0, 0.5)
        .with_milestone_group("Build", "Code freeze", 10.0)
        .with_task_group("Ship", "QA", 9.0, 11.0)
        .with_milestone("Launch", 12.0)
        .with_now_line(7.0);

    // ── Row 10: TextPlot, Quiver, Horizontal modes ───────────────────────────

    let text = TextPlot::new()
        .with_title("TextPlot")
        .with_body(
            "**Bold**, *italic*, `code`.\n\nMath: $\\sigma^2 = \\frac{1}{n}\\sum x_i^2$\n\nHeadings, rules, and\nword-wrapped paragraphs.",
        );

    let quiver =
        QuiverPlot::from_function((-3.0, 3.0, 8), (-3.0, 3.0, 8), |x, y| (-y * 0.3, x * 0.3))
            .with_color("steelblue");

    let (bar_h, box_h, violin_h, raincloud_h) = {
        let simple_groups: Vec<(&str, Vec<f64>)> = group_data
            .iter()
            .take(3)
            .map(|(l, d)| (*l, d.clone()))
            .collect();

        let bar_h = BarPlot::new()
            .with_group("Control", vec![(5.2_f64, "steelblue")])
            .with_group("Low", vec![(6.8_f64, "steelblue")])
            .with_group("High", vec![(8.1_f64, "steelblue")])
            .with_horizontal(true);

        let box_h = simple_groups
            .iter()
            .fold(BoxPlot::new(), |b, (lbl, v)| b.with_group(*lbl, v.clone()))
            .with_horizontal(true);

        let violin_h = simple_groups
            .iter()
            .fold(ViolinPlot::new(), |v, (lbl, d)| {
                v.with_group(*lbl, d.clone())
            })
            .with_horizontal(true);

        let raincloud_h = simple_groups
            .iter()
            .fold(RaincloudPlot::new(), |r, (lbl, d)| {
                r.with_group(*lbl, d.clone())
            })
            .with_horizontal(true);

        (bar_h, box_h, violin_h, raincloud_h)
    };

    // ── Assemble 11×6 Figure (row-major, 66 cells) ───────────────────────────

    let hmap_row_labels: Vec<String> = genes.iter().map(|s| s.to_string()).collect();
    let hmap_col_labels: Vec<String> = genes.iter().map(|s| s.to_string()).collect();

    let all_plots: Vec<Vec<Plot>> = vec![
        // Row 0: Scatter, Line, Bar, Histogram, Histogram2D, Hexbin
        vec![
            Plot::Scatter(sc_a),
            Plot::Scatter(sc_b),
            Plot::Scatter(sc_c),
        ],
        vec![Plot::Line(line_a), Plot::Line(line_b), Plot::Line(line_c)],
        vec![Plot::Bar(bar)],
        vec![Plot::Histogram(histogram)],
        vec![Plot::Histogram2d(hist2d)],
        vec![Plot::Hexbin(hexbin)],
        // Row 1: Box, Violin, Strip, Waterfall, StackedArea, Streamgraph
        vec![Plot::Box(box_plot)],
        vec![Plot::Violin(violin)],
        vec![Plot::Strip(strip)],
        vec![Plot::Waterfall(waterfall)],
        vec![Plot::StackedArea(stacked_area)],
        vec![Plot::Streamgraph(streamgraph)],
        // Row 2: Pie, Series, Band, Heatmap, DotPlot, Clustermap
        vec![Plot::Pie(pie)],
        vec![Plot::Series(ser1), Plot::Series(ser2), Plot::Series(ser3)],
        vec![Plot::Line(band_line)],
        vec![Plot::Heatmap(hmap)],
        vec![Plot::DotPlot(dot)],
        vec![Plot::Clustermap(clustermap)],
        // Row 3: Volcano, Manhattan, Candlestick, Contour, UpSet, Network
        vec![Plot::Volcano(volcano)],
        vec![Plot::Manhattan(manhattan)],
        vec![Plot::Candlestick(candle)],
        vec![Plot::Contour(contour)],
        vec![Plot::UpSet(upset)],
        vec![Plot::Network(network)],
        // Row 4: Chord, Sankey, PhyloTree, Synteny, Brick, Treemap
        vec![Plot::Chord(chord)],
        vec![Plot::Sankey(sankey)],
        vec![Plot::PhyloTree(phylo)],
        vec![Plot::Synteny(synteny)],
        vec![Plot::Brick(brick)],
        vec![Plot::Treemap(treemap)],
        // Row 5: Density, Ridgeline, Polar, Ternary, Forest, ECDF
        vec![Plot::Density(dens_a), Plot::Density(dens_b)],
        vec![Plot::Ridgeline(ridgeline)],
        vec![Plot::Polar(polar)],
        vec![Plot::Ternary(ternary)],
        vec![Plot::Forest(forest)],
        vec![Plot::Ecdf(ecdf)],
        // Row 6: ROC, PR, Survival, DicePlot, Raincloud, Lollipop
        vec![Plot::Roc(roc)],
        vec![Plot::Pr(pr)],
        vec![Plot::Survival(survival)],
        vec![Plot::DicePlot(dice)],
        vec![Plot::Raincloud(raincloud)],
        vec![Plot::Lollipop(lollipop)],
        // Row 7: JointPlot, Waffle, Horizon, Pyramid, Mosaic, Slope
        vec![Plot::Joint(joint)],
        vec![Plot::Waffle(waffle)],
        vec![Plot::Horizon(horizon)],
        vec![Plot::Pyramid(pyramid)],
        vec![Plot::Mosaic(mosaic)],
        vec![Plot::Slope(slope)],
        // Row 8: Venn, Parallel, Radar, Rose, Sunburst, Bump
        vec![Plot::Venn(venn)],
        vec![Plot::Parallel(parallel)],
        vec![Plot::Radar(radar)],
        vec![Plot::Rose(rose)],
        vec![Plot::Sunburst(sunburst)],
        vec![Plot::Bump(bump)],
        // Row 9: QQ, Scatter3D, Surface3D, Calendar, Funnel, Gantt
        vec![Plot::QQ(qq)],
        vec![Plot::Scatter3D(scatter3d.0), Plot::Scatter3D(scatter3d.1)],
        vec![Plot::Surface3D(surface3d)],
        vec![Plot::Calendar(calendar)],
        vec![Plot::Funnel(funnel)],
        vec![Plot::Gantt(gantt)],
        // Row 10: TextPlot, Quiver, Horizontal Bar, Horizontal Box, Horizontal Violin, Horizontal Raincloud
        vec![Plot::Text(text)],
        vec![Plot::Quiver(quiver)],
        vec![Plot::Bar(bar_h)],
        vec![Plot::Box(box_h)],
        vec![Plot::Violin(violin_h)],
        vec![Plot::Raincloud(raincloud_h)],
    ];

    let layouts: Vec<Layout> = all_plots
        .iter()
        .enumerate()
        .map(|(i, cell)| {
            let base = Layout::auto_from_plots(cell);
            match i {
                // Row 0
                0 => base
                    .with_title("Scatter")
                    .with_x_label("x")
                    .with_y_label("y"),
                1 => base
                    .with_title("Line")
                    .with_x_label("x")
                    .with_y_label("sigmoid(x)"),
                2 => base.with_title("Bar").with_y_label("Value"),
                3 => base
                    .with_title("Histogram")
                    .with_x_label("Value")
                    .with_y_label("Count"),
                4 => base
                    .with_title("Histogram 2D")
                    .with_x_label("x")
                    .with_y_label("y"),
                5 => base
                    .with_title("Hexbin")
                    .with_x_label("x")
                    .with_y_label("y"),
                // Row 1
                6 => base.with_title("Box Plot").with_y_label("Value"),
                7 => base.with_title("Violin").with_y_label("Value"),
                8 => base.with_title("Strip").with_y_label("Value"),
                9 => base.with_title("Waterfall").with_y_label("Value (M)"),
                10 => base
                    .with_title("Stacked Area")
                    .with_x_label("Year")
                    .with_y_label("Count"),
                11 => base
                    .with_title("Streamgraph")
                    .with_x_label("Month")
                    .with_y_label("Frequency"),
                // Row 2
                12 => base.with_title("Pie Chart"),
                13 => base
                    .with_title("Series")
                    .with_x_label("Index")
                    .with_y_label("Amplitude"),
                14 => base
                    .with_title("Band (CI)")
                    .with_x_label("x")
                    .with_y_label("sin(x)"),
                15 => base
                    .with_title("Heatmap")
                    .with_x_categories(hmap_col_labels.clone())
                    .with_y_categories(hmap_row_labels.clone()),
                16 => base.with_title("Dot Plot"),
                17 => base.with_title("Clustermap"),
                // Row 3
                18 => base
                    .with_title("Volcano")
                    .with_x_label("log₂ FC")
                    .with_y_label("-log₁₀(p)"),
                19 => base.with_title("Manhattan").with_y_label("-log₁₀(p)"),
                20 => base.with_title("Candlestick").with_y_label("Price"),
                21 => base
                    .with_title("Contour")
                    .with_x_label("x")
                    .with_y_label("y"),
                22 => base.with_title("UpSet"),
                23 => base.with_title("Network"),
                // Row 4
                24 => base.with_title("Chord"),
                25 => base.with_title("Sankey"),
                26 => base.with_title("Phylogenetic Tree"),
                27 => base.with_title("Synteny"),
                28 => base.with_title("Brick (DNA)"),
                29 => base.with_title("Treemap"),
                // Row 5
                30 => base
                    .with_title("Density")
                    .with_x_label("Value")
                    .with_y_label("Density"),
                31 => base
                    .with_title("Ridgeline")
                    .with_x_label("Temperature (°C)"),
                32 => base.with_title("Polar"),
                33 => base.with_title("Ternary"),
                34 => base.with_title("Forest Plot").with_x_label("Odds Ratio"),
                35 => base
                    .with_title("ECDF")
                    .with_x_label("Value")
                    .with_y_label("Cumulative P"),
                // Row 6
                36 => base
                    .with_title("ROC Curve")
                    .with_x_label("False Positive Rate")
                    .with_y_label("True Positive Rate"),
                37 => base
                    .with_title("Precision-Recall")
                    .with_x_label("Recall")
                    .with_y_label("Precision"),
                38 => base
                    .with_title("Survival")
                    .with_x_label("Time (months)")
                    .with_y_label("Probability"),
                39 => base.with_title("Dice Plot"),
                40 => base.with_title("Raincloud").with_y_label("Expression"),
                41 => base
                    .with_title("Lollipop")
                    .with_x_label("Position")
                    .with_y_label("Frequency"),
                // Row 7
                42 => base
                    .with_title("Joint Plot")
                    .with_x_label("Feature x")
                    .with_y_label("Feature y"),
                43 => base.with_title("Waffle"),
                44 => base.with_title("Horizon"),
                45 => base
                    .with_title("Population Pyramid")
                    .with_x_label("Population (%)"),
                46 => base.with_title("Mosaic"),
                47 => base.with_title("Slope Chart"),
                // Row 8
                48 => base.with_title("Venn Diagram"),
                49 => base.with_title("Parallel Coordinates"),
                50 => base.with_title("Radar"),
                51 => base.with_title("Rose / Wind"),
                52 => base.with_title("Sunburst"),
                53 => base.with_title("Bump Chart").with_y_label("Rank"),
                // Row 9
                54 => base.with_title("Q-Q Plot"),
                55 => base.with_title("Scatter 3D"),
                56 => base.with_title("Surface 3D"),
                57 => base.with_title("Calendar Heatmap"),
                58 => base.with_title("Funnel").with_y_label("Stage"),
                59 => base.with_title("Gantt"),
                // Row 10
                60 => base.with_title("Text Plot"),
                61 => base
                    .with_title("Quiver")
                    .with_x_label("x")
                    .with_y_label("y"),
                62 => base.with_title("Horizontal Bar").with_x_label("Value"),
                63 => base.with_title("Horizontal Box").with_x_label("Value"),
                64 => base.with_title("Horizontal Violin").with_x_label("Value"),
                65 => base
                    .with_title("Horizontal Raincloud")
                    .with_x_label("Value"),
                _ => base,
            }
        })
        .collect();

    let fig = Figure::new(11, 6)
        .with_cell_size(600.0, 460.0)
        .with_plots(all_plots)
        .with_layouts(layouts);

    let scene = fig.render();
    let svg = SvgBackend.render_scene(&scene);
    let out = "docs/src/assets/overview";
    std::fs::create_dir_all(out).expect("could not create docs/src/assets/overview");
    std::fs::write(format!("{out}/all_plots_complex.svg"), &svg).unwrap();
    println!("Written to {out}/all_plots_complex.svg");

    let png = kuva::backend::png::PngBackend::new()
        .with_scale(3.0)
        .render_scene(&scene)
        .unwrap();
    std::fs::write(format!("{out}/all_plots_complex.png"), png).unwrap();
    println!("Written to {out}/all_plots_complex.png");

    let pdf = kuva::backend::pdf::PdfBackend::new()
        .render_scene(&scene)
        .expect("PDF rendering failed");
    std::fs::write(format!("{out}/all_plots_complex.pdf"), pdf).unwrap();
    println!("Written to {out}/all_plots_complex.pdf");
}
