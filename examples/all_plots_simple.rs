//! Compact showcase of all 62 kuva plot types in a 11×6 Figure grid.
//! Each cell uses minimal inline data — see all_plots_complex for larger
//! datasets with axes, legends, and titles.
//!
//! Run with:
//!   cargo run --example all_plots_simple
//!
//! Output: docs/src/assets/overview/all_plots_simple.svg

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

fn main() {
    // ── Row 0: Scatter, Line, Bar, Histogram, Histogram2D, Hexbin ────────────

    // 0: Scatter
    let pts: Vec<(f64, f64)> = (0..30)
        .map(|i| (i as f64, (i as f64 * 0.3).sin()))
        .collect();
    let scatter = ScatterPlot::new().with_data(pts).with_color("steelblue");

    // 1: Line
    let pts: Vec<(f64, f64)> = (0..30)
        .map(|i| (i as f64, (i as f64 * 0.3).cos()))
        .collect();
    let line = LinePlot::new().with_data(pts).with_color("firebrick");

    // 2: Bar (5 categories)
    let bar = BarPlot::new()
        .with_group("A", vec![(4.0_f64, "steelblue")])
        .with_group("B", vec![(7.0_f64, "steelblue")])
        .with_group("C", vec![(3.0_f64, "steelblue")])
        .with_group("D", vec![(8.0_f64, "steelblue")])
        .with_group("E", vec![(5.0_f64, "steelblue")]);

    // 3: Histogram — bell-shaped
    let hist_data: Vec<f64> = [
        5.0, 5.0, 10.0, 10.0, 10.0, 10.0, 15.0, 15.0, 15.0, 15.0, 15.0, 15.0, 15.0, 20.0, 20.0,
        20.0, 20.0, 20.0, 20.0, 20.0, 20.0, 20.0, 20.0, 25.0, 25.0, 25.0, 25.0, 25.0, 25.0, 25.0,
        25.0, 25.0, 25.0, 25.0, 25.0, 30.0, 30.0, 30.0, 30.0, 30.0, 30.0, 30.0, 30.0, 35.0, 35.0,
        35.0, 35.0, 35.0, 40.0, 40.0,
    ]
    .to_vec();
    let histogram = Histogram::new()
        .with_data(hist_data)
        .with_range((0.0, 45.0))
        .with_bins(9);

    // 4: Histogram2D
    let h2d_data: Vec<(f64, f64)> = (0..40)
        .map(|i| (i as f64 * 0.5, i as f64 * 0.4 + 1.0))
        .collect();
    let hist2d = Histogram2D::new().with_data(h2d_data, (0.0, 20.0), (0.0, 20.0), 8, 8);

    // 5: Hexbin
    let hx: Vec<f64> = (0..60).map(|i| i as f64 * 0.1).collect();
    let hy: Vec<f64> = hx.iter().map(|&x| x.sin() + x * 0.2).collect();
    let hexbin = HexbinPlot::new().with_data(hx, hy).with_n_bins(8);

    // ── Row 1: Box, Violin, Strip, Waterfall, StackedArea, Streamgraph ───────

    let grp_a: Vec<f64> = (0..20).map(|i| 2.0 + i as f64 * 0.15).collect();
    let grp_b: Vec<f64> = (0..20).map(|i| 4.0 + i as f64 * 0.12).collect();
    let grp_c: Vec<f64> = (0..20).map(|i| 6.0 + i as f64 * 0.10).collect();

    // 6: Box
    let box_plot = BoxPlot::new()
        .with_group("A", grp_a.clone())
        .with_group("B", grp_b.clone())
        .with_group("C", grp_c.clone());

    // 7: Violin
    let violin = ViolinPlot::new()
        .with_group("A", grp_a.clone())
        .with_group("B", grp_b.clone())
        .with_group("C", grp_c.clone());

    // 8: Strip
    let strip = StripPlot::new()
        .with_group("A", grp_a.clone())
        .with_group("B", grp_b.clone())
        .with_group("C", grp_c.clone())
        .with_color("steelblue");

    // 9: Waterfall
    let waterfall = WaterfallPlot::new()
        .with_delta("Start", 100.0)
        .with_delta("Q1", 30.0)
        .with_delta("Q2", -20.0)
        .with_delta("Q3", 15.0)
        .with_delta("Q4", -10.0)
        .with_total("Total");

    // 10: StackedArea
    let stacked_area = StackedAreaPlot::new()
        .with_x([0.0_f64, 1.0, 2.0, 3.0])
        .with_series([10.0_f64, 15.0, 12.0, 18.0])
        .with_color("steelblue")
        .with_series([8.0_f64, 12.0, 9.0, 14.0])
        .with_color("firebrick")
        .with_series([5.0_f64, 8.0, 6.0, 10.0])
        .with_color("forestgreen");

    // 11: Streamgraph
    let streamgraph = StreamgraphPlot::new()
        .with_x([0.0_f64, 1.0, 2.0, 3.0, 4.0])
        .with_series([5.0_f64, 8.0, 6.0, 9.0, 7.0])
        .with_color("#4e79a7")
        .with_series([3.0_f64, 5.0, 8.0, 4.0, 6.0])
        .with_color("#f28e2b")
        .with_series([7.0_f64, 4.0, 5.0, 6.0, 8.0])
        .with_color("#59a14f");

    // ── Row 2: Pie, Series, Band, Heatmap, DotPlot, Clustermap ───────────────

    // 12: Pie
    let pie = PiePlot::new()
        .with_slice("A", 30.0, "steelblue")
        .with_slice("B", 20.0, "firebrick")
        .with_slice("C", 35.0, "forestgreen")
        .with_slice("D", 15.0, "orange")
        .with_label_position(PieLabelPosition::Inside);

    // 13: Series — three overlaid
    let series1 = SeriesPlot::new()
        .with_data([1.0_f64, 3.0, 2.0, 4.0, 3.0])
        .with_color("steelblue")
        .with_line_style();
    let series2 = SeriesPlot::new()
        .with_data([2.0_f64, 1.0, 3.0, 2.0, 4.0])
        .with_color("firebrick")
        .with_line_style();
    let series3 = SeriesPlot::new()
        .with_data([3.0_f64, 2.0, 1.0, 3.0, 2.0])
        .with_color("forestgreen")
        .with_line_style();

    // 14: Band
    let bx: Vec<(f64, f64)> = (0..10)
        .map(|i| {
            let x = i as f64;
            (x, x * 0.6 + 1.0)
        })
        .collect();
    let lo: Vec<f64> = bx.iter().map(|(_, y)| y - 1.0).collect();
    let hi: Vec<f64> = bx.iter().map(|(_, y)| y + 1.0).collect();
    let band_line = LinePlot::new()
        .with_data(bx)
        .with_color("steelblue")
        .with_band(lo, hi);

    // 15: Heatmap (4×4)
    let hmap = Heatmap::new().with_data(vec![
        vec![1.0_f64, 2.0, 3.0, 4.0],
        vec![2.0_f64, 4.0, 3.0, 1.0],
        vec![3.0_f64, 1.0, 4.0, 2.0],
        vec![4.0_f64, 3.0, 1.0, 2.0],
    ]);

    // 16: DotPlot (3×3)
    let dot = DotPlot::new().with_data([
        ("X", "A", 10.0_f64, 1.0_f64),
        ("X", "B", 15.0_f64, 2.0_f64),
        ("X", "C", 8.0_f64, 0.5_f64),
        ("Y", "A", 12.0_f64, 1.5_f64),
        ("Y", "B", 20.0_f64, 3.0_f64),
        ("Y", "C", 5.0_f64, 0.2_f64),
        ("Z", "A", 7.0_f64, 0.8_f64),
        ("Z", "B", 11.0_f64, 1.2_f64),
        ("Z", "C", 18.0_f64, 2.5_f64),
    ]);

    // 17: Clustermap (4×4 two-block)
    let clustermap = Clustermap::new()
        .with_data(vec![
            vec![0.9_f64, 0.8, 0.1, 0.2],
            vec![0.8_f64, 0.9, 0.2, 0.1],
            vec![0.1_f64, 0.2, 0.9, 0.8],
            vec![0.2_f64, 0.1, 0.8, 0.9],
        ])
        .with_row_labels(["r1", "r2", "r3", "r4"])
        .with_col_labels(["c1", "c2", "c3", "c4"]);

    // ── Row 3: Volcano, Manhattan, Candlestick, Contour, UpSet, Network ──────

    // 18: Volcano
    let volcano = VolcanoPlot::new()
        .with_point("GeneA", 2.5, 0.001)
        .with_point("GeneB", -2.1, 0.005)
        .with_point("GeneC", 3.0, 0.0001)
        .with_point("GeneD", 0.5, 0.3)
        .with_point("GeneE", -0.3, 0.5)
        .with_point("GeneF", 1.8, 0.02)
        .with_point("GeneG", -1.5, 0.04)
        .with_point("GeneH", 0.1, 0.8)
        .with_point("GeneI", -3.2, 0.0005)
        .with_point("GeneJ", 2.0, 0.01);

    // 19: Manhattan
    let manhattan = ManhattanPlot::new().with_data([
        ("chr1", 0.12_f64),
        ("chr1", 0.34),
        ("chr1", 0.07),
        ("chr1", 0.22),
        ("chr1", 0.51),
        ("chr1", 0.09),
        ("chr1", 0.41),
        ("chr1", 0.19),
        ("chr1", 0.63),
        ("chr1", 0.003),
        ("chr2", 0.28_f64),
        ("chr2", 0.46),
        ("chr2", 0.17),
        ("chr2", 0.38),
        ("chr2", 0.55),
        ("chr2", 0.24),
        ("chr2", 0.67),
        ("chr2", 0.11),
        ("chr2", 0.43),
        ("chr2", 0.31),
        ("chr3", 0.47_f64),
        ("chr3", 0.26),
        ("chr3", 0.59),
        ("chr3", 0.15),
        ("chr3", 0.33),
        ("chr3", 0.1e-6),
        ("chr3", 0.48),
        ("chr3", 0.21),
        ("chr3", 0.37),
        ("chr3", 0.44),
    ]);

    // 20: Candlestick
    let candle = CandlestickPlot::new()
        .with_candle("Mon", 100.0, 110.0, 98.0, 107.0)
        .with_candle("Tue", 107.0, 115.0, 105.0, 112.0)
        .with_candle("Wed", 112.0, 113.0, 104.0, 106.0)
        .with_candle("Thu", 106.0, 108.0, 100.0, 102.0)
        .with_candle("Fri", 102.0, 111.0, 101.0, 109.0);

    // 21: Contour
    let contour_pts: Vec<(f64, f64, f64)> = (0..5)
        .flat_map(|r| {
            (0..5).map(move |c| {
                let x = c as f64 * 2.0;
                let y = r as f64 * 2.0;
                (x, y, -((x - 4.0).powi(2) + (y - 4.0).powi(2)).sqrt())
            })
        })
        .collect();
    let contour = ContourPlot::new().with_points(contour_pts);

    // 22: UpSet
    let upset = UpSetPlot::new().with_data(
        ["A", "B", "C"],
        [30_usize, 25, 20],
        [(1u64, 15), (2u64, 12), (4u64, 10), (3u64, 7), (7u64, 4)],
    );

    // 23: Network
    let network = NetworkPlot::new()
        .with_edge("A", "B", 1.0)
        .with_edge("B", "C", 1.0)
        .with_edge("C", "D", 1.0)
        .with_edge("D", "A", 1.0)
        .with_edge("A", "C", 0.5);

    // ── Row 4: Chord, Sankey, PhyloTree, Synteny, Brick, Treemap ─────────────

    // 24: Chord
    let chord = ChordPlot::new()
        .with_matrix(vec![
            vec![0.0, 80.0, 60.0],
            vec![80.0, 0.0, 40.0],
            vec![60.0, 40.0, 0.0],
        ])
        .with_labels(["X", "Y", "Z"]);

    // 25: Sankey
    let sankey = SankeyPlot::new()
        .with_node_color("Source", "steelblue")
        .with_node_color("Mid A", "forestgreen")
        .with_node_color("Mid B", "firebrick")
        .with_node_color("Sink", "orange")
        .with_link("Source", "Mid A", 40.0)
        .with_link("Source", "Mid B", 30.0)
        .with_link("Mid A", "Sink", 40.0)
        .with_link("Mid B", "Sink", 30.0);

    // 26: PhyloTree
    let phylo = PhyloTree::from_newick("((A:1,B:1):1,(C:1,D:1):1);");

    // 27: Synteny
    let synteny = SyntenyPlot::new()
        .with_sequences([("Seq1", 1_000_000.0_f64), ("Seq2", 900_000.0_f64)])
        .with_block(0, 100_000.0, 400_000.0, 1, 150_000.0, 450_000.0)
        .with_inv_block(0, 500_000.0, 800_000.0, 1, 500_000.0, 800_000.0);

    // 28: Brick
    let tmpl = BrickTemplate::new().dna();
    let brick = BrickPlot::new()
        .with_sequences(["ACGTACGTACGT", "CGTACGTACGTA", "GTACGTACGTAC"])
        .with_names(["read_1", "read_2", "read_3"])
        .with_template(tmpl.template);

    // 29: Treemap
    let treemap = TreemapPlot::new()
        .with_children(
            "Group A",
            vec![
                TreemapNode::leaf("X1", 10.0),
                TreemapNode::leaf("X2", 8.0),
                TreemapNode::leaf("X3", 6.0),
            ],
        )
        .with_children(
            "Group B",
            vec![TreemapNode::leaf("Y1", 14.0), TreemapNode::leaf("Y2", 9.0)],
        );

    // ── Row 5: Density, Ridgeline, Polar, Ternary, Forest, ECDF ─────────────

    // 30: Density (two overlapping groups)
    let density_a = DensityPlot::new()
        .with_data([
            2.0_f64, 3.0, 3.5, 4.0, 4.0, 4.5, 5.0, 5.0, 5.5, 6.0, 6.0, 6.5, 7.0, 7.5, 8.0,
        ])
        .with_color("steelblue")
        .with_filled(true)
        .with_opacity(0.6);
    let density_b = DensityPlot::new()
        .with_data([
            4.0_f64, 5.0, 5.5, 6.0, 6.0, 6.5, 7.0, 7.0, 7.5, 8.0, 8.0, 8.5, 9.0, 9.5, 10.0,
        ])
        .with_color("firebrick")
        .with_filled(true)
        .with_opacity(0.6);

    // 31: Ridgeline
    let ridgeline = RidgelinePlot::new()
        .with_group("A", vec![1.0_f64, 2.0, 2.5, 3.0, 3.0, 3.5, 4.0, 4.5, 5.0])
        .with_group("B", vec![3.0_f64, 4.0, 4.5, 5.0, 5.0, 5.5, 6.0, 6.5, 7.0])
        .with_group("C", vec![5.0_f64, 6.0, 6.5, 7.0, 7.0, 7.5, 8.0, 8.5, 9.0]);

    // 32: Polar (cardioid)
    let n = 36usize;
    let theta: Vec<f64> = (0..n).map(|i| i as f64 * 360.0 / n as f64).collect();
    let r: Vec<f64> = theta.iter().map(|&t| 1.0 + t.to_radians().cos()).collect();
    let polar = PolarPlot::new().with_series_labeled(r, theta, "Cardioid", PolarMode::Line);

    // 33: Ternary
    let ternary = TernaryPlot::new()
        .with_point_group(0.80, 0.12, 0.08, "A-rich")
        .with_point_group(0.75, 0.15, 0.10, "A-rich")
        .with_point_group(0.12, 0.78, 0.10, "B-rich")
        .with_point_group(0.10, 0.80, 0.10, "B-rich")
        .with_point_group(0.10, 0.12, 0.78, "C-rich")
        .with_point_group(0.08, 0.10, 0.82, "C-rich");

    // 34: Forest
    let forest = ForestPlot::new()
        .with_row("Study A", 0.82, 0.55, 1.21)
        .with_row("Study B", 1.15, 0.73, 1.62)
        .with_row("Study C", 0.63, 0.31, 0.94)
        .with_row("Study D", 1.28, 0.90, 1.75)
        .with_row("Meta", 0.94, 0.74, 1.14);

    // 35: ECDF (2 groups)
    let ecdf = EcdfPlot::new()
        .with_data("A", [1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0])
        .with_data(
            "B",
            [2.5_f64, 3.5, 4.5, 5.5, 6.5, 7.5, 8.5, 9.5, 10.5, 11.5],
        );

    // ── Row 6: ROC, PR, Survival, DicePlot, Raincloud, Lollipop ──────────────

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

    // 36: ROC
    let roc = RocPlot::new()
        .with_group(RocGroup::new("Classifier").with_raw(logistic_dataset(60, 1.0, 0.5)));

    // 37: PR
    let pr =
        PrPlot::new().with_group(PrGroup::new("Model").with_raw(logistic_dataset(60, 1.0, 0.5)));

    // 38: Survival (2 groups)
    let survival = SurvivalPlot::new()
        .with_group(
            "Control",
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 7.0, 9.0, 11.0, 14.0, 18.0],
            vec![
                true, true, false, true, false, true, false, true, false, false,
            ],
        )
        .with_group(
            "Treatment",
            vec![2.0, 4.0, 6.0, 8.0, 12.0, 16.0, 20.0, 24.0, 28.0, 32.0],
            vec![
                false, true, false, true, false, false, true, false, false, false,
            ],
        );

    // 39: DicePlot
    let dice = DicePlot::new(4).with_points([
        ("X1", "G1", vec![0, 2], None, None),
        ("X2", "G1", vec![1, 3], None, None),
        ("X1", "G2", vec![0, 1], None, None),
        ("X2", "G2", vec![2, 3], None, None),
    ]);

    // 40: Raincloud
    let raincloud = RaincloudPlot::new()
        .with_group("A", (0..15).map(|i| 2.0 + i as f64 * 0.4).collect())
        .with_group("B", (0..15).map(|i| 4.5 + i as f64 * 0.3).collect());

    // 41: Lollipop
    let lollipop = LollipopPlot::new()
        .with_point(1.0, 3.5)
        .with_point(2.0, -1.2)
        .with_point(3.0, 4.1)
        .with_point(4.0, -2.3)
        .with_point(5.0, 1.8)
        .with_point(6.0, 3.0);

    // ── Row 7: JointPlot, Waffle, Horizon, Pyramid, Mosaic, Slope ────────────

    // 42: JointPlot
    let joint = JointPlot::new().with_xy(
        (0..20).map(|i| i as f64 * 0.3),
        (0..20).map(|i| (i as f64 * 0.3).sin() + i as f64 * 0.1),
    );

    // 43: Waffle (4 categories)
    let waffle = WafflePlot::new()
        .with_category("Alpha", 40.0, "steelblue")
        .with_category("Beta", 30.0, "firebrick")
        .with_category("Gamma", 20.0, "forestgreen")
        .with_category("Delta", 10.0, "orange");

    // 44: Horizon (2 series)
    let horizon = HorizonPlot::new()
        .with_series(
            "A",
            (0..10).map(|i| i as f64),
            [2.0_f64, 4.0, -1.0, 3.0, 5.0, -2.0, 1.0, 4.0, -3.0, 2.0],
        )
        .with_series(
            "B",
            (0..10).map(|i| i as f64),
            [-1.0_f64, 2.0, 4.0, -2.0, 3.0, 1.0, -3.0, 2.0, 4.0, -1.0],
        );

    // 45: Population Pyramid
    let pyramid = PopulationPyramid::new()
        .with_group("0–20", 12.0, 11.0)
        .with_group("21–40", 18.0, 17.0)
        .with_group("41–60", 15.0, 16.0)
        .with_group("61–80", 10.0, 12.0)
        .with_group("81+", 4.0, 6.0);

    // 46: Mosaic (3 columns × 2 rows)
    let mosaic = MosaicPlot::new()
        .with_cell("X", "a", 30.0)
        .with_cell("X", "b", 20.0)
        .with_cell("Y", "a", 15.0)
        .with_cell("Y", "b", 35.0)
        .with_cell("Z", "a", 25.0)
        .with_cell("Z", "b", 25.0);

    // 47: Slope (5 items)
    let slope = SlopePlot::new()
        .with_point("Alpha", 3.5, 5.2)
        .with_point("Beta", 6.1, 4.8)
        .with_point("Gamma", 2.0, 3.9)
        .with_point("Delta", 4.4, 6.0)
        .with_point("Epsilon", 1.8, 2.5);

    // ── Row 8: Venn, Parallel, Radar, Rose, Sunburst, Bump ───────────────────

    // 48: Venn (2 sets)
    let venn = VennPlot::new()
        .with_set_size("Set A", 40)
        .with_set_size("Set B", 35)
        .with_overlap(["Set A", "Set B"], 15);

    // 49: Parallel coordinates (4 axes, 2 groups)
    let parallel = ParallelPlot::new()
        .with_axis_names(["Speed", "Power", "Agility", "Stamina"])
        .with_row_group("A", vec![8.0, 6.0, 7.0, 5.0])
        .with_row_group("A", vec![7.0, 7.0, 6.0, 6.0])
        .with_row_group("A", vec![9.0, 5.0, 8.0, 4.0])
        .with_row_group("B", vec![4.0, 9.0, 5.0, 8.0])
        .with_row_group("B", vec![5.0, 8.0, 4.0, 9.0])
        .with_row_group("B", vec![3.0, 10.0, 3.0, 7.0]);

    // 50: Radar (4 axes, 2 series)
    let radar = RadarPlot::new(["Speed", "Power", "Agility", "Stamina"])
        .with_series_labeled([8.0, 6.0, 9.0, 5.0], "Hero")
        .with_series_labeled([5.0, 9.0, 4.0, 8.0], "Villain")
        .with_filled(true);

    // 51: Rose (12 sectors, 2 stacked wind-speed bands)
    let rose = RosePlot::new()
        .with_stack(
            "Calm (0–5)",
            [
                6.0_f64, 14.0, 10.0, 5.0, 4.0, 3.0, 8.0, 12.0, 7.0, 4.0, 3.0, 5.0,
            ],
        )
        .with_stack(
            "Strong (>5)",
            [
                3.0_f64, 8.0, 5.0, 2.0, 1.0, 2.0, 5.0, 7.0, 3.0, 2.0, 1.0, 3.0,
            ],
        )
        .with_compass_labels()
        .with_legend("Wind speed");

    // 52: Sunburst (2-level hierarchy)
    let sunburst = SunburstPlot::new()
        .with_children(
            "Group A",
            vec![TreemapNode::leaf("A1", 10.0), TreemapNode::leaf("A2", 8.0)],
        )
        .with_children(
            "Group B",
            vec![
                TreemapNode::leaf("B1", 6.0),
                TreemapNode::leaf("B2", 9.0),
                TreemapNode::leaf("B3", 5.0),
            ],
        );

    // 53: Bump (4 series, 5 time steps)
    let bump = BumpPlot::new()
        .with_series("Alpha", [1.0_f64, 2.0, 1.0, 3.0, 2.0])
        .with_series("Beta", [2.0_f64, 1.0, 3.0, 1.0, 1.0])
        .with_series("Gamma", [3.0_f64, 3.0, 2.0, 2.0, 3.0])
        .with_series("Delta", [4.0_f64, 4.0, 4.0, 4.0, 4.0])
        .with_x_labels(["T1", "T2", "T3", "T4", "T5"]);

    // ── Row 9: QQ, Scatter3D, Surface3D, Calendar, Funnel ────────────────────

    // 54: Q-Q plot
    let qq = QQPlot::new()
        .with_data(
            "Sample",
            [2.0_f64, 4.0, 5.0, 7.0, 8.0, 9.0, 11.0, 12.0, 14.0, 16.0],
        )
        .with_ci_band();

    // 55: Scatter3D (helix)
    let scatter3d = Scatter3DPlot::new()
        .with_data((0..20).map(|i| {
            let t = i as f64 * 0.3;
            (t.cos(), t.sin(), t * 0.2)
        }))
        .with_color("steelblue");

    // 56: Surface3D (Gaussian bump, 4×4 grid)
    let z_grid: Vec<Vec<f64>> = (0..4)
        .map(|r| {
            (0..4)
                .map(|c| {
                    let x = c as f64 - 1.5;
                    let y = r as f64 - 1.5;
                    (-(x * x + y * y)).exp()
                })
                .collect()
        })
        .collect();
    let surface3d = Surface3DPlot::new(z_grid);

    // 57: Calendar (Jan–Apr 2025 — ~18 cols fits the figure cell at default cell_size)
    let calendar = CalendarPlot::new()
        .with_date_range("2025-01-01", "2025-04-30")
        .with_data([
            ("2025-01-05", 3.0_f64),
            ("2025-01-12", 5.0),
            ("2025-01-19", 2.0),
            ("2025-02-02", 7.0),
            ("2025-02-09", 4.0),
            ("2025-02-16", 6.0),
            ("2025-03-02", 8.0),
            ("2025-03-09", 3.0),
            ("2025-03-16", 5.0),
            ("2025-04-06", 4.0),
            ("2025-04-13", 6.0),
            ("2025-04-20", 9.0),
        ]);

    // 58: Funnel (4 stages)
    let funnel = FunnelPlot::new()
        .with_stage("Visitors", 1000.0)
        .with_stage("Signups", 400.0)
        .with_stage("Trials", 150.0)
        .with_stage("Customers", 60.0);

    // 59: Gantt (3 groups)
    let gantt = GanttPlot::new()
        .with_task_group("Plan", "Scope", 0.0, 2.0)
        .with_task_group("Build", "Dev", 2.0, 6.0)
        .with_task_group("Build", "Test", 5.0, 7.0)
        .with_milestone("Launch", 7.0)
        .with_now_line(4.0);

    // ── Row 10: TextPlot, Quiver, Horizontal modes ───────────────────────────

    // 60: TextPlot
    let text = TextPlot::new()
        .with_title("TextPlot")
        .with_body("**Bold**, *italic*,\n`code`, and\n$\\alpha + \\beta$");

    // 61: QuiverPlot (rotational field)
    let quiver = QuiverPlot::from_function((-2.0, 2.0, 6), (-2.0, 2.0, 6), |x, y| {
        (-y * 0.4, x * 0.4)
    });

    // 62: Horizontal bar
    let bar_h = BarPlot::new()
        .with_group("A", vec![(4.0_f64, "steelblue")])
        .with_group("B", vec![(7.0_f64, "steelblue")])
        .with_group("C", vec![(3.0_f64, "steelblue")])
        .with_group("D", vec![(8.0_f64, "steelblue")])
        .with_group("E", vec![(5.0_f64, "steelblue")])
        .with_horizontal(true);

    // 63: Horizontal box
    let box_h = BoxPlot::new()
        .with_group("A", grp_a.clone())
        .with_group("B", grp_b.clone())
        .with_group("C", grp_c.clone())
        .with_horizontal(true);

    // 64: Horizontal violin
    let violin_h = ViolinPlot::new()
        .with_group("A", grp_a.clone())
        .with_group("B", grp_b.clone())
        .with_group("C", grp_c.clone())
        .with_horizontal(true);

    // 65: Horizontal raincloud
    let raincloud_h = RaincloudPlot::new()
        .with_group("A", (0..15).map(|i| 2.0 + i as f64 * 0.4).collect())
        .with_group("B", (0..15).map(|i| 4.5 + i as f64 * 0.3).collect())
        .with_horizontal(true);

    // ── Assemble 11×6 Figure (row-major, 66 cells) ───────────────────────────

    let all_plots: Vec<Vec<Plot>> = vec![
        // Row 0: Scatter, Line, Bar, Histogram, Histogram2D, Hexbin
        vec![Plot::Scatter(scatter)],
        vec![Plot::Line(line)],
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
        vec![
            Plot::Series(series1),
            Plot::Series(series2),
            Plot::Series(series3),
        ],
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
        vec![Plot::Density(density_a), Plot::Density(density_b)],
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
        vec![Plot::Scatter3D(scatter3d)],
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
        .map(|cell| Layout::auto_from_plots(cell))
        .collect();

    let fig = Figure::new(11, 6)
        .with_cell_size(500.0, 380.0)
        .with_plots(all_plots)
        .with_layouts(layouts);

    let scene = fig.render();
    let svg = SvgBackend.render_scene(&scene);
    let out = "docs/src/assets/overview";
    std::fs::create_dir_all(out).expect("could not create docs/src/assets/overview");
    std::fs::write(format!("{out}/all_plots_simple.svg"), &svg).unwrap();
    println!("Written to {out}/all_plots_simple.svg");
}
