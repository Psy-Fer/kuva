//! Compact showcase of all 25 kuva plot types in a single 5×5 Figure grid.
//! Each cell uses minimal inline data — click through to all_plots_complex
//! for larger datasets with axes, legends, and titles.
//!
//! Run with:
//!   cargo run --example all_plots_simple
//!
//! Output: examples/all_plots_simple.svg

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

fn main() {
    // ── Row 0: Scatter, Line, Bar, Histogram, Histogram2D ─────────────────

    // 0: Scatter
    let pts: Vec<(f64, f64)> = (0..30)
        .map(|i| (i as f64, (i as f64 * 0.3).sin()))
        .collect();
    let scatter = ScatterPlot::new()
        .with_data(pts)
        .with_color("steelblue");

    // 1: Line
    let pts: Vec<(f64, f64)> = (0..30)
        .map(|i| (i as f64, (i as f64 * 0.3).cos()))
        .collect();
    let line = LinePlot::new()
        .with_data(pts)
        .with_color("firebrick");

    // 2: Bar (5 categories, single series)
    let bar = BarPlot::new()
        .with_group("A", vec![(4.0_f64, "steelblue")])
        .with_group("B", vec![(7.0_f64, "steelblue")])
        .with_group("C", vec![(3.0_f64, "steelblue")])
        .with_group("D", vec![(8.0_f64, "steelblue")])
        .with_group("E", vec![(5.0_f64, "steelblue")]);

    // 3: Histogram — bell-shaped data so bars have unequal heights
    // 50 values approximating a normal distribution (2+4+7+10+12+8+5+2 = 50)
    let hist_data: Vec<f64> = [
        5.0, 5.0,
        10.0, 10.0, 10.0, 10.0,
        15.0, 15.0, 15.0, 15.0, 15.0, 15.0, 15.0,
        20.0, 20.0, 20.0, 20.0, 20.0, 20.0, 20.0, 20.0, 20.0, 20.0,
        25.0, 25.0, 25.0, 25.0, 25.0, 25.0, 25.0, 25.0, 25.0, 25.0, 25.0, 25.0,
        30.0, 30.0, 30.0, 30.0, 30.0, 30.0, 30.0, 30.0,
        35.0, 35.0, 35.0, 35.0, 35.0,
        40.0, 40.0,
    ].to_vec();
    let histogram = Histogram::new()
        .with_data(hist_data)
        .with_range((0.0, 45.0))
        .with_bins(9);

    // 4: Histogram2D (40 points along a diagonal band)
    let h2d_data: Vec<(f64, f64)> = (0..40)
        .map(|i| (i as f64 * 0.5, i as f64 * 0.4 + 1.0))
        .collect();
    let hist2d = Histogram2D::new()
        .with_data(h2d_data, (0.0, 20.0), (0.0, 20.0), 8, 8);

    // ── Row 1: Box, Violin, Strip, Waterfall, StackedArea ─────────────────

    // Shared group data for Box/Violin/Strip — linear progressions with spread
    let grp_a: Vec<f64> = (0..20).map(|i| 2.0 + i as f64 * 0.15).collect();
    let grp_b: Vec<f64> = (0..20).map(|i| 4.0 + i as f64 * 0.12).collect();
    let grp_c: Vec<f64> = (0..20).map(|i| 6.0 + i as f64 * 0.10).collect();

    // 5: Box
    let box_plot = BoxPlot::new()
        .with_group("A", grp_a.clone())
        .with_group("B", grp_b.clone())
        .with_group("C", grp_c.clone());

    // 6: Violin
    let violin = ViolinPlot::new()
        .with_group("A", grp_a.clone())
        .with_group("B", grp_b.clone())
        .with_group("C", grp_c.clone());

    // 7: Strip
    let strip = StripPlot::new()
        .with_group("A", grp_a)
        .with_group("B", grp_b)
        .with_group("C", grp_c)
        .with_color("steelblue");

    // 8: Waterfall
    let waterfall = WaterfallPlot::new()
        .with_delta("Start",  100.0)
        .with_delta("Q1",      30.0)
        .with_delta("Q2",     -20.0)
        .with_delta("Q3",      15.0)
        .with_delta("Q4",     -10.0)
        .with_total("Total");

    // 9: StackedArea (4 time steps × 3 series)
    let stacked_area = StackedAreaPlot::new()
        .with_x([0.0_f64, 1.0, 2.0, 3.0])
        .with_series([10.0_f64, 15.0, 12.0, 18.0])
        .with_color("steelblue")
        .with_legend("S1")
        .with_series([8.0_f64, 12.0, 9.0, 14.0])
        .with_color("firebrick")
        .with_legend("S2")
        .with_series([5.0_f64, 8.0, 6.0, 10.0])
        .with_color("forestgreen")
        .with_legend("S3");

    // ── Row 2: Pie, Series, Band, Heatmap, DotPlot ────────────────────────

    // 10: Pie (inside labels — avoids overflow inside a Figure cell)
    let pie = PiePlot::new()
        .with_slice("A", 30.0, "steelblue")
        .with_slice("B", 20.0, "firebrick")
        .with_slice("C", 35.0, "forestgreen")
        .with_slice("D", 15.0, "orange")
        .with_label_position(PieLabelPosition::Inside);

    // 11: Series — three overlaid series in the same cell
    let series1 = SeriesPlot::new()
        .with_data([1.0_f64, 3.0, 2.0, 4.0, 3.0])
        .with_color("steelblue")
        .with_line_style()
        .with_legend("S1");
    let series2 = SeriesPlot::new()
        .with_data([2.0_f64, 1.0, 3.0, 2.0, 4.0])
        .with_color("firebrick")
        .with_line_style()
        .with_legend("S2");
    let series3 = SeriesPlot::new()
        .with_data([3.0_f64, 2.0, 1.0, 3.0, 2.0])
        .with_color("forestgreen")
        .with_line_style()
        .with_legend("S3");

    // 12: Band — use LinePlot::with_band so the center line is visible
    let bx: Vec<(f64, f64)> = (0..10)
        .map(|i| { let x = i as f64; (x, x * 0.6 + 1.0) })
        .collect();
    let lo: Vec<f64> = bx.iter().map(|(_, y)| y - 1.0).collect();
    let hi: Vec<f64> = bx.iter().map(|(_, y)| y + 1.0).collect();
    let band_line = LinePlot::new()
        .with_data(bx)
        .with_color("steelblue")
        .with_band(lo, hi);

    // 13: Heatmap (4×4 matrix)
    let hmap = Heatmap::new().with_data(vec![
        vec![1.0_f64, 2.0, 3.0, 4.0],
        vec![2.0_f64, 4.0, 3.0, 1.0],
        vec![3.0_f64, 1.0, 4.0, 2.0],
        vec![4.0_f64, 3.0, 1.0, 2.0],
    ]);

    // 14: DotPlot (3×3 grid; size & color values)
    let dot = DotPlot::new().with_data([
        ("X", "A", 10.0_f64, 1.0_f64),
        ("X", "B", 15.0_f64, 2.0_f64),
        ("X", "C",  8.0_f64, 0.5_f64),
        ("Y", "A", 12.0_f64, 1.5_f64),
        ("Y", "B", 20.0_f64, 3.0_f64),
        ("Y", "C",  5.0_f64, 0.2_f64),
        ("Z", "A",  7.0_f64, 0.8_f64),
        ("Z", "B", 11.0_f64, 1.2_f64),
        ("Z", "C", 18.0_f64, 2.5_f64),
    ]);

    // ── Row 3: Volcano, Manhattan, Candlestick, Contour, UpSet ────────────

    // 15: Volcano (actual p-values, not -log10)
    let volcano = VolcanoPlot::new()
        .with_point("GeneA",  2.5,  0.001)
        .with_point("GeneB", -2.1,  0.005)
        .with_point("GeneC",  3.0,  0.0001)
        .with_point("GeneD",  0.5,  0.3)
        .with_point("GeneE", -0.3,  0.5)
        .with_point("GeneF",  1.8,  0.02)
        .with_point("GeneG", -1.5,  0.04)
        .with_point("GeneH",  0.1,  0.8)
        .with_point("GeneI", -3.2,  0.0005)
        .with_point("GeneJ",  2.0,  0.01);

    // 16: Manhattan — with_data takes (chrom, raw_pvalue) where pvalue ∈ (0,1)
    let manhattan = ManhattanPlot::new().with_data([
        // chr1 — one near-significant SNP
        ("chr1", 0.12_f64), ("chr1", 0.34), ("chr1", 0.07), ("chr1", 0.22),
        ("chr1", 0.51),     ("chr1", 0.09), ("chr1", 0.41), ("chr1", 0.19),
        ("chr1", 0.63),     ("chr1", 0.003),
        // chr2 — mostly uninformative
        ("chr2", 0.28_f64), ("chr2", 0.46), ("chr2", 0.17), ("chr2", 0.38),
        ("chr2", 0.55),     ("chr2", 0.24), ("chr2", 0.67), ("chr2", 0.11),
        ("chr2", 0.43),     ("chr2", 0.31),
        // chr3 — one significant hit
        ("chr3", 0.47_f64), ("chr3", 0.26), ("chr3", 0.59), ("chr3", 0.15),
        ("chr3", 0.33),     ("chr3", 0.1e-6), ("chr3", 0.48), ("chr3", 0.21),
        ("chr3", 0.37),     ("chr3", 0.44),
    ]);

    // 17: Candlestick (5 OHLC bars)
    let candle = CandlestickPlot::new()
        .with_candle("Mon", 100.0, 110.0,  98.0, 107.0)
        .with_candle("Tue", 107.0, 115.0, 105.0, 112.0)
        .with_candle("Wed", 112.0, 113.0, 104.0, 106.0)
        .with_candle("Thu", 106.0, 108.0, 100.0, 102.0)
        .with_candle("Fri", 102.0, 111.0, 101.0, 109.0);

    // 18: Contour (25 scattered points on a 5×5 grid; IDW interpolation inside)
    let contour_pts: Vec<(f64, f64, f64)> = (0..5)
        .flat_map(|r| {
            (0..5).map(move |c| {
                let x = c as f64 * 2.0;
                let y = r as f64 * 2.0;
                let z = -((x - 4.0).powi(2) + (y - 4.0).powi(2)).sqrt();
                (x, y, z)
            })
        })
        .collect();
    let contour = ContourPlot::new().with_points(contour_pts);

    // 19: UpSet (3 sets; bitmask encoding: A=1, B=2, C=4)
    let upset = UpSetPlot::new().with_data(
        ["A", "B", "C"],
        [30_usize, 25, 20],
        [(1u64, 15), (2u64, 12), (4u64, 10), (3u64, 7), (7u64, 4)],
    );

    // ── Row 4: Chord, Sankey, PhyloTree, Synteny, Brick ──────────────────

    // 20: Chord (3×3 symmetric matrix)
    let chord = ChordPlot::new()
        .with_matrix(vec![
            vec![ 0.0, 80.0, 60.0],
            vec![80.0,  0.0, 40.0],
            vec![60.0, 40.0,  0.0],
        ])
        .with_labels(["X", "Y", "Z"]);

    // 21: Sankey (source-color ribbons — default, cleaner for small cells)
    let sankey = SankeyPlot::new()
        .with_node_color("Source", "steelblue")
        .with_node_color("Mid A",  "forestgreen")
        .with_node_color("Mid B",  "firebrick")
        .with_node_color("Sink",   "orange")
        .with_link("Source", "Mid A", 40.0)
        .with_link("Source", "Mid B", 30.0)
        .with_link("Mid A",  "Sink",  40.0)
        .with_link("Mid B",  "Sink",  30.0);

    // 22: PhyloTree from Newick (4 leaves)
    let phylo = PhyloTree::from_newick("((A:1,B:1):1,(C:1,D:1):1);");

    // 23: Synteny (2 sequences, 1 forward + 1 inverted block)
    let synteny = SyntenyPlot::new()
        .with_sequences([("Seq1", 1_000_000.0_f64), ("Seq2", 900_000.0_f64)])
        .with_block(0, 100_000.0, 400_000.0, 1, 150_000.0, 450_000.0)
        .with_inv_block(0, 500_000.0, 800_000.0, 1, 500_000.0, 800_000.0);

    // 24: Brick (DNA sequence mode)
    let tmpl = BrickTemplate::new().dna();
    let brick = BrickPlot::new()
        .with_sequences(["ACGTACGTACGT", "CGTACGTACGTA", "GTACGTACGTAC"])
        .with_names(["read_1", "read_2", "read_3"])
        .with_template(tmpl.template);

    // ── Assemble 5×5 Figure (row-major, 25 cells) ─────────────────────────

    let all_plots: Vec<Vec<Plot>> = vec![
        // Row 0
        vec![Plot::Scatter(scatter)],
        vec![Plot::Line(line)],
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
        vec![Plot::Series(series1), Plot::Series(series2), Plot::Series(series3)],
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

    // Auto-compute one Layout per cell before moving all_plots into Figure
    let layouts: Vec<Layout> = all_plots.iter()
        .map(|cell| Layout::auto_from_plots(cell))
        .collect();

    let fig = Figure::new(5, 5)
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
