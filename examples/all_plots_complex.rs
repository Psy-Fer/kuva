//! Full-featured showcase of all 25 kuva plot types.
//! Each cell uses a larger dataset and includes a title, axis labels,
//! and a legend where applicable.
//!
//! Run with:
//!   cargo run --example all_plots_complex
//!
//! Output: examples/all_plots_complex.svg

use kuva::plot::{
    ScatterPlot, LinePlot, BarPlot, Histogram, Histogram2D,
    BoxPlot, ViolinPlot, StripPlot, WaterfallPlot, StackedAreaPlot,
    PiePlot, PieLabelPosition, SeriesPlot, Heatmap,
    DotPlot, VolcanoPlot, ManhattanPlot, CandlestickPlot, ContourPlot,
    UpSetPlot, ChordPlot, SankeyPlot, PhyloTree, SyntenyPlot, BrickPlot,
};
use kuva::plot::brick::BrickTemplate;
use kuva::render::plots::Plot;
use kuva::render::layout::Layout;
use kuva::render::figure::Figure;
use kuva::backend::svg::SvgBackend;

// ── Deterministic pseudo-random (LCG) ─────────────────────────────────────
fn lcg(seed: &mut u64) -> f64 {
    *seed = seed.wrapping_mul(6_364_136_223_846_793_005)
               .wrapping_add(1_442_695_040_888_963_407);
    (*seed >> 33) as f64 / 4_294_967_296.0 // [0, 1)
}

fn main() {
    // ── Row 0: Scatter ─────────────────────────────────────────────────────
    // Three clusters of 40 points each

    let mut seed = 12345u64;
    let mut scatter_pts: Vec<(f64, f64, &'static str)> = Vec::new();
    for (cx, cy, col) in [(2.0, 3.0, "steelblue"), (6.0, 6.0, "firebrick"), (10.0, 2.0, "forestgreen")] {
        for _ in 0..40 {
            let x = cx + (lcg(&mut seed) - 0.5) * 2.5;
            let y = cy + (lcg(&mut seed) - 0.5) * 2.5;
            scatter_pts.push((x, y, col));
        }
    }
    // Build three ScatterPlots (one per cluster)
    let sc_a = ScatterPlot::new()
        .with_data(scatter_pts.iter().take(40).map(|(x, y, _)| (*x, *y)).collect::<Vec<_>>())
        .with_color("steelblue")
        .with_legend("Group A");
    let sc_b = ScatterPlot::new()
        .with_data(scatter_pts.iter().skip(40).take(40).map(|(x, y, _)| (*x, *y)).collect::<Vec<_>>())
        .with_color("firebrick")
        .with_legend("Group B");
    let sc_c = ScatterPlot::new()
        .with_data(scatter_pts.iter().skip(80).map(|(x, y, _)| (*x, *y)).collect::<Vec<_>>())
        .with_color("forestgreen")
        .with_legend("Group C");

    // ── Row 0: Line ────────────────────────────────────────────────────────
    // Three sigmoid-like curves

    let line_x: Vec<f64> = (0..60).map(|i| i as f64 * 0.2).collect();
    let line_a = LinePlot::new()
        .with_data(line_x.iter().map(|&x| (x, 1.0 / (1.0 + (-(x - 3.0)).exp()))).collect::<Vec<_>>())
        .with_color("steelblue")
        .with_legend("Low");
    let line_b = LinePlot::new()
        .with_data(line_x.iter().map(|&x| (x, 1.0 / (1.0 + (-(x - 6.0)).exp()))).collect::<Vec<_>>())
        .with_color("firebrick")
        .with_legend("Mid");
    let line_c = LinePlot::new()
        .with_data(line_x.iter().map(|&x| (x, 1.0 / (1.0 + (-(x - 9.0)).exp()))).collect::<Vec<_>>())
        .with_color("forestgreen")
        .with_legend("High");

    // ── Row 0: Bar (grouped, 4 groups × 4 conditions) ─────────────────────
    let bar = BarPlot::new()
        .with_group("Control", vec![
            (4.2, "steelblue"), (3.8, "firebrick"), (5.1, "forestgreen"), (4.7, "orange"),
        ])
        .with_group("Drug A", vec![
            (6.1, "steelblue"), (5.9, "firebrick"), (7.2, "forestgreen"), (6.5, "orange"),
        ])
        .with_group("Drug B", vec![
            (3.5, "steelblue"), (4.1, "firebrick"), (3.2, "forestgreen"), (3.9, "orange"),
        ])
        .with_group("Combo", vec![
            (8.3, "steelblue"), (7.7, "firebrick"), (9.1, "forestgreen"), (8.0, "orange"),
        ])
        .with_legend(vec!["T1", "T2", "T3", "T4"]);

    // ── Row 0: Histogram (100 values, roughly normal) ─────────────────────
    let mut hist_vals: Vec<f64> = Vec::new();
    let mut s = 99991u64;
    for _ in 0..100 {
        // Box–Muller approximation via sum of 12 uniforms
        let u: f64 = (0..12).map(|_| lcg(&mut s)).sum::<f64>() - 6.0;
        hist_vals.push(u * 10.0 + 50.0); // mean=50, sd≈10
    }
    let h_min = hist_vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let h_max = hist_vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let histogram = Histogram::new()
        .with_data(hist_vals)
        .with_range((h_min.floor(), h_max.ceil()))
        .with_bins(12)
        .with_color("steelblue")
        .with_legend("Count");

    // ── Row 0: Histogram2D (bimodal: two Gaussian clusters, 20×20 bins) ─────
    let mut h2d_pts: Vec<(f64, f64)> = Vec::new();
    let mut s = 22222u64;
    // Cluster 1: centre (5, 8), sd ≈ 1.5
    for _ in 0..500 {
        let ux: f64 = (0..6).map(|_| lcg(&mut s)).sum::<f64>() - 3.0;
        let uy: f64 = (0..6).map(|_| lcg(&mut s)).sum::<f64>() - 3.0;
        h2d_pts.push((ux * 1.5 + 5.0, uy * 1.5 + 8.0));
    }
    // Cluster 2: centre (14, 10), sd ≈ 1.5
    for _ in 0..500 {
        let ux: f64 = (0..6).map(|_| lcg(&mut s)).sum::<f64>() - 3.0;
        let uy: f64 = (0..6).map(|_| lcg(&mut s)).sum::<f64>() - 3.0;
        h2d_pts.push((ux * 1.5 + 14.0, uy * 1.5 + 10.0));
    }
    let hist2d = Histogram2D::new()
        .with_data(h2d_pts, (0.0, 20.0), (0.0, 20.0), 40, 40);

    // ── Row 1: Box/Violin/Strip (5 groups × 30 values each) ───────────────
    let group_data: Vec<(&str, Vec<f64>)> = {
        let mut s = 33333u64;
        ["Ctrl", "Low", "Mid", "High", "Max"].iter().enumerate().map(|(gi, &label)| {
            let mean = 2.0 + gi as f64 * 1.5;
            let spread = 0.5 + gi as f64 * 0.1;
            let vals: Vec<f64> = (0..30).map(|_| {
                let u: f64 = (0..6).map(|_| lcg(&mut s)).sum::<f64>() - 3.0;
                mean + u * spread
            }).collect();
            (label, vals)
        }).collect()
    };

    let box_plot = group_data.iter().fold(BoxPlot::new(), |b, (lbl, v)| b.with_group(*lbl, v.clone()));
    let violin   = group_data.iter().fold(ViolinPlot::new(), |v, (lbl, d)| v.with_group(*lbl, d.clone()));
    let strip    = group_data.iter().fold(
        StripPlot::new().with_color("steelblue"),
        |st, (lbl, d)| st.with_group(*lbl, d.clone()),
    );

    // ── Row 1: Waterfall (8 items) ─────────────────────────────────────────
    let waterfall = WaterfallPlot::new()
        .with_delta("Revenue",      520.0)
        .with_delta("COGS",        -180.0)
        .with_delta("Gross Profit", 340.0)
        .with_delta("R&D",          -90.0)
        .with_delta("S&M",          -70.0)
        .with_delta("G&A",          -40.0)
        .with_delta("Other",         15.0)
        .with_total("EBIT")
        .with_connectors()
        .with_values();

    // ── Row 1: StackedArea (10 time steps × 4 series) ─────────────────────
    let sa_x: Vec<f64> = (0..10).map(|i| 2015.0 + i as f64).collect();
    let stacked_area = StackedAreaPlot::new()
        .with_x(sa_x)
        .with_series([120.0, 135.0, 148.0, 162.0, 175.0, 190.0, 205.0, 220.0, 238.0, 255.0])
        .with_color("#4e79a7").with_legend("Alpha")
        .with_series([85.0, 92.0, 98.0, 105.0, 112.0, 118.0, 125.0, 133.0, 141.0, 150.0])
        .with_color("#f28e2b").with_legend("Beta")
        .with_series([60.0, 65.0, 71.0, 78.0, 83.0, 90.0, 96.0, 103.0, 110.0, 118.0])
        .with_color("#59a14f").with_legend("Gamma")
        .with_series([40.0, 45.0, 50.0, 56.0, 62.0, 68.0, 75.0, 82.0, 89.0, 97.0])
        .with_color("#e15759").with_legend("Delta");

    // ── Row 2: Pie (6 slices, percentages) ────────────────────────────────
    let pie = PiePlot::new()
        .with_slice("Alpha",  32.0, "#4e79a7")
        .with_slice("Beta",   21.0, "#f28e2b")
        .with_slice("Gamma",  17.0, "#59a14f")
        .with_slice("Delta",  14.0, "#e15759")
        .with_slice("Epsilon", 9.0, "#76b7b2")
        .with_slice("Zeta",    7.0, "#edc948")
        .with_label_position(PieLabelPosition::Outside)
        .with_percent();

    // ── Row 2: Series (3 series × 80 points, line+point style) ───────────
    let ser_x: Vec<f64> = (0..80).map(|i| i as f64).collect();
    let ser1 = SeriesPlot::new()
        .with_data(ser_x.iter().map(|&x| (x * 0.1).sin()).collect::<Vec<_>>())
        .with_color("steelblue").with_line_point_style().with_legend("sin(0.1x)");
    let ser2 = SeriesPlot::new()
        .with_data(ser_x.iter().map(|&x| (x * 0.15).cos()).collect::<Vec<_>>())
        .with_color("firebrick").with_line_point_style().with_legend("cos(0.15x)");
    let ser3 = SeriesPlot::new()
        .with_data(ser_x.iter().map(|&x| (x * 0.1).sin() * (x * 0.05).cos()).collect::<Vec<_>>())
        .with_color("forestgreen").with_line_point_style().with_legend("product");

    // ── Row 2: Band (sinusoidal confidence band) ───────────────────────────
    let band_x: Vec<f64> = (0..50).map(|i| i as f64 * 0.2).collect();
    let band_y: Vec<f64> = band_x.iter().map(|&x| x.sin()).collect();
    let band_lo: Vec<f64> = band_y.iter().map(|&y| y - 0.4).collect();
    let band_hi: Vec<f64> = band_y.iter().map(|&y| y + 0.4).collect();
    let band_line = LinePlot::new()
        .with_data(band_x.iter().zip(band_y.iter()).map(|(&x, &y)| (x, y)).collect::<Vec<_>>())
        .with_color("steelblue")
        .with_band(band_lo, band_hi)
        .with_legend("sin(x) ± 0.4");

    // ── Row 2: Heatmap (6×6 correlation matrix with labels) ───────────────
    let genes = ["BRCA1", "TP53", "EGFR", "MYC", "KRAS", "PTEN"];
    let heat_matrix = vec![
        vec![ 1.00,  0.72,  0.31, -0.45,  0.18, -0.62],
        vec![ 0.72,  1.00,  0.55, -0.23,  0.41, -0.38],
        vec![ 0.31,  0.55,  1.00,  0.08,  0.63, -0.11],
        vec![-0.45, -0.23,  0.08,  1.00, -0.29,  0.74],
        vec![ 0.18,  0.41,  0.63, -0.29,  1.00,  0.05],
        vec![-0.62, -0.38, -0.11,  0.74,  0.05,  1.00],
    ];
    let hmap = Heatmap::new()
        .with_data(heat_matrix)
        .with_labels(
            genes.iter().map(|s| s.to_string()).collect(),
            genes.iter().map(|s| s.to_string()).collect(),
        );

    // ── Row 2: DotPlot (4×4 grid, pathway × cell-type) ───────────────────
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
    let dot_data: Vec<(&str, &str, f64, f64)> = pathways.iter().enumerate()
        .flat_map(|(pi, &path)| {
            cell_types.iter().enumerate().map(move |(ci, &ct)| {
                (path, ct, sizes[pi][ci], colors[pi][ci])
            })
        })
        .collect();
    let dot = DotPlot::new()
        .with_data(dot_data)
        .with_size_legend("% expressed")
        .with_colorbar("mean expr");

    // ── Row 3: Volcano (35 genes) ──────────────────────────────────────────
    let volcano_data = [
        ("BRCA1",   2.5,  0.001), ("TP53",    1.8,  0.01),  ("EGFR",    3.2,  0.0001),
        ("MYC",     1.5,  0.03),  ("KRAS",    2.1,  0.005), ("CDK4",    1.2,  0.04),
        ("PTEN",    2.8,  0.002), ("RB1",     1.9,  0.008), ("AKT1",    3.5,  0.00005),
        ("VEGFA",   2.3,  0.003), ("CDKN2A", -2.3,  0.002), ("SMAD4",  -1.9,  0.008),
        ("VHL",    -3.0,  0.0005),("CASP3",  -1.6,  0.04),  ("BCL2",   -2.7,  0.001),
        ("FAS",    -1.4,  0.035), ("PUMA",   -2.0,  0.007), ("BAX",    -1.7,  0.015),
        ("P21",    -3.2,  0.0002),("MDM2",   -2.5,  0.003), ("GAPDH",   0.3,  0.5),
        ("ACTB",   -0.5,  0.3),   ("TUBA1",   0.8,  0.1),   ("HIST1",  -0.2,  0.7),
        ("RPL5",    0.6,  0.2),   ("RPS6",   -0.9,  0.15),  ("EEF1A",   0.1,  0.8),
        ("HNRNPA", -0.7,  0.4),   ("SF3B1",   0.4,  0.6),   ("SRSF1",  -0.3,  0.9),
        ("GeneA",   1.5,  0.2),   ("GeneB",  -1.1,  0.07),  ("GeneC",   0.9,  0.12),
        ("GeneD",  -0.8,  0.08),  ("GeneE",   1.3,  0.18),
    ];
    let volcano = VolcanoPlot::new()
        .with_points(volcano_data)
        .with_label_top(5);

    // ── Row 3: Manhattan (sequential, 3 chroms × 80 SNPs each) ────────────
    let mut man_data: Vec<(&str, f64)> = Vec::new();
    let mut s = 77777u64;
    for (chrom, has_signal) in [("1", true), ("2", false), ("3", true)] {
        for i in 0..80usize {
            let r = lcg(&mut s);
            let p = if has_signal && i > 35 && i < 42 {
                r * 5e-9   // genome-wide significant cluster
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

    // ── Row 3: Candlestick (10 OHLC bars with volume) ─────────────────────
    let ohlc = [
        ("Jan", 98.0,  108.0, 95.0,  105.0),
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
    let candle = ohlc.iter().fold(CandlestickPlot::new(), |c, &(l, o, h, lo, cl)| {
        c.with_candle(l, o, h, lo, cl)
    });

    // ── Row 3: Contour (Gaussian field, filled) ────────────────────────────
    let n = 8usize;
    let coords: Vec<f64> = (0..n).map(|i| -2.0 + i as f64 / (n as f64 - 1.0) * 4.0).collect();
    let contour_pts: Vec<(f64, f64, f64)> = coords.iter().flat_map(|&y| {
        coords.iter().map(move |&x| {
            let z = (-(x * x + y * y) / 1.5).exp()
                  + 0.5 * (-(( x - 1.0).powi(2) + (y + 1.0).powi(2)) / 0.5).exp();
            (x, y, z)
        })
    }).collect();
    let contour = ContourPlot::new()
        .with_points(contour_pts)
        .with_filled()
        .with_n_levels(8)
        .with_legend("Density");

    // ── Row 3: UpSet (4 sets) ──────────────────────────────────────────────
    // A=1, B=2, C=4, D=8
    let upset = UpSetPlot::new().with_data(
        ["A", "B", "C", "D"],
        [60_usize, 50, 45, 35],
        [
            (1u64,  28), (2u64, 22), (4u64, 18), (8u64, 12),  // singletons
            (3u64,  15), (5u64, 11), (9u64,  9), (6u64,  8),  // pairs
            (12u64,  6), (7u64,  5), (14u64, 4), (15u64, 3),  // triples/quads
        ],
    );

    // ── Row 4: Chord (4×4 cell-type co-clustering) ────────────────────────
    let chord = ChordPlot::new()
        .with_matrix(vec![
            vec![  0.0, 120.0,  80.0,  50.0],
            vec![120.0,   0.0,  95.0,  40.0],
            vec![ 80.0,  95.0,   0.0,  70.0],
            vec![ 50.0,  40.0,  70.0,   0.0],
        ])
        .with_labels(["CD4 T", "CD8 T", "NK", "B cell"])
        .with_legend("Cell types");

    // ── Row 4: Sankey (5 nodes, source-color ribbons) ─────────────────────
    let sankey = SankeyPlot::new()
        .with_node_color("Reads",     "#4e79a7")
        .with_node_color("Mapped",    "#59a14f")
        .with_node_color("Unmapped",  "#e15759")
        .with_node_color("Exonic",    "#f28e2b")
        .with_node_color("Intronic",  "#76b7b2")
        .with_link("Reads",    "Mapped",   850.0)
        .with_link("Reads",    "Unmapped", 150.0)
        .with_link("Mapped",   "Exonic",   620.0)
        .with_link("Mapped",   "Intronic", 230.0)
        .with_legend("RNA-seq flow");

    // ── Row 4: PhyloTree (8 leaves, Newick) ───────────────────────────────
    let phylo = PhyloTree::from_newick(
        "(((Homo:0.01,Pan:0.01):0.05,Gorilla:0.06):0.15,\
          ((Pongo:0.08,Hylobates:0.08):0.1,\
           ((Macaca:0.2,Papio:0.2):0.1,Colobus:0.3):0.05):0.1);"
    );

    // ── Row 4: Synteny (6 chromosomes, many blocks) ────────────────────────
    let mut synteny = SyntenyPlot::new()
        .with_sequences([
            ("Chr 1", 248_956_422.0_f64),
            ("Chr 2", 242_193_529.0_f64),
            ("Chr 3", 198_295_559.0_f64),
            ("Chr 4", 190_214_555.0_f64),
            ("Chr 5", 181_538_259.0_f64),
            ("Chr 6", 170_805_979.0_f64),
        ]);
    // Four forward blocks across adjacent chromosome pairs
    let pairs: [(usize, usize); 5] = [(0,1),(1,2),(2,3),(3,4),(4,5)];
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

    // ── Row 4: Brick (8 reads, DNA mode) ──────────────────────────────────
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
        .with_names(["r1","r2","r3","r4","r5","r6","r7","r8"])
        .with_template(tmpl.template)
        .with_x_offset(18.0);

    // ── Assemble 5×5 Figure ────────────────────────────────────────────────

    let all_plots: Vec<Vec<Plot>> = vec![
        // Row 0
        vec![Plot::Scatter(sc_a), Plot::Scatter(sc_b), Plot::Scatter(sc_c)],
        vec![Plot::Line(line_a), Plot::Line(line_b), Plot::Line(line_c)],
        vec![Plot::Bar(bar)],
        vec![Plot::Histogram(histogram)],
        vec![Plot::Histogram2d(hist2d)],
        // Row 1
        vec![Plot::Box(box_plot)],
        vec![Plot::Violin(violin)],
        vec![Plot::Strip(strip)],
        vec![Plot::Waterfall(waterfall)],
        vec![Plot::StackedArea(stacked_area)],
        // Row 2
        vec![Plot::Pie(pie)],
        vec![Plot::Series(ser1), Plot::Series(ser2), Plot::Series(ser3)],
        vec![Plot::Line(band_line)],
        vec![Plot::Heatmap(hmap)],
        vec![Plot::DotPlot(dot)],
        // Row 3
        vec![Plot::Volcano(volcano)],
        vec![Plot::Manhattan(manhattan)],
        vec![Plot::Candlestick(candle)],
        vec![Plot::Contour(contour)],
        vec![Plot::UpSet(upset)],
        // Row 4
        vec![Plot::Chord(chord)],
        vec![Plot::Sankey(sankey)],
        vec![Plot::PhyloTree(phylo)],
        vec![Plot::Synteny(synteny)],
        vec![Plot::Brick(brick)],
    ];

    // Build one layout per cell: auto-compute ranges, then add metadata
    let hmap_row_labels: Vec<String> = genes.iter().map(|s| s.to_string()).collect();
    let hmap_col_labels: Vec<String> = genes.iter().map(|s| s.to_string()).collect();

    let layouts: Vec<Layout> = all_plots.iter().enumerate()
        .map(|(i, cell)| {
            let base = Layout::auto_from_plots(cell);
            match i {
                // Row 0
                0  => base.with_title("Scatter").with_x_label("x").with_y_label("y"),
                1  => base.with_title("Line").with_x_label("x").with_y_label("sigmoid(x)"),
                2  => base.with_title("Bar").with_y_label("Value"),
                3  => base.with_title("Histogram").with_x_label("Value").with_y_label("Count"),
                4  => base.with_title("Histogram 2D").with_x_label("x").with_y_label("y"),
                // Row 1
                5  => base.with_title("Box Plot").with_y_label("Value"),
                6  => base.with_title("Violin").with_y_label("Value"),
                7  => base.with_title("Strip").with_y_label("Value"),
                8  => base.with_title("Waterfall").with_y_label("Value (M)"),
                9  => base.with_title("Stacked Area").with_x_label("Year").with_y_label("Count"),
                // Row 2
                10 => base.with_title("Pie Chart"),
                11 => base.with_title("Series").with_x_label("Index").with_y_label("Amplitude"),
                12 => base.with_title("Band (CI)").with_x_label("x").with_y_label("sin(x)"),
                13 => base.with_title("Heatmap")
                           .with_x_categories(hmap_col_labels.clone())
                           .with_y_categories(hmap_row_labels.clone()),
                14 => base.with_title("Dot Plot"),
                // Row 3
                15 => base.with_title("Volcano").with_x_label("log₂ FC").with_y_label("-log₁₀(p)"),
                16 => base.with_title("Manhattan").with_y_label("-log₁₀(p)"),
                17 => base.with_title("Candlestick").with_y_label("Price"),
                18 => base.with_title("Contour").with_x_label("x").with_y_label("y"),
                19 => base.with_title("UpSet"),
                // Row 4
                20 => base.with_title("Chord"),
                21 => base.with_title("Sankey"),
                22 => base.with_title("Phylogenetic Tree"),
                23 => base.with_title("Synteny"),
                24 => base.with_title("Brick (DNA)"),
                _  => base,
            }
        })
        .collect();

    let fig = Figure::new(5, 5)
        .with_cell_size(600.0, 460.0)
        .with_plots(all_plots)
        .with_layouts(layouts);

    let scene = fig.render();
    let svg = SvgBackend.render_scene(&scene);
    let out = "docs/src/assets/overview";
    std::fs::create_dir_all(out).expect("could not create docs/src/assets/overview");
    std::fs::write(format!("{out}/all_plots_complex.svg"), &svg).unwrap();
    println!("Written to {out}/all_plots_complex.svg");

    let png = kuva::backend::png::PngBackend::new().with_scale(3.0).render_scene(&scene).unwrap();
    std::fs::write(format!("{out}/all_plots_complex.png"), png).unwrap();
    println!("Written to {out}/all_plots_complex.png");
}
