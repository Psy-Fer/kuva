use std::sync::Arc;

use crate::plot::scatter::{ScatterPlot, TrendLine};
use crate::plot::line::LinePlot;
use crate::plot::bar::BarPlot;
use crate::plot::histogram::Histogram;
use crate::plot::boxplot::BoxPlot;
use crate::plot::violin::ViolinPlot;
use crate::plot::brick::BrickPlot;

use crate::plot::{Heatmap, Histogram2D, PiePlot, SeriesPlot};
use crate::plot::band::BandPlot;
use crate::plot::waterfall::{WaterfallPlot, WaterfallKind};
use crate::plot::strip::StripPlot;
use crate::plot::volcano::VolcanoPlot;
use crate::plot::manhattan::ManhattanPlot;
use crate::plot::dotplot::DotPlot;
use crate::plot::upset::UpSetPlot;
use crate::plot::stacked_area::StackedAreaPlot;
use crate::plot::candlestick::CandlestickPlot;
use crate::plot::contour::ContourPlot;
use crate::plot::chord::ChordPlot;
use crate::plot::sankey::SankeyPlot;
use crate::plot::phylo::PhyloTree;
use crate::plot::synteny::SyntenyPlot;
use crate::plot::density::DensityPlot;
use crate::plot::ecdf::EcdfPlot;
use crate::plot::ridgeline::RidgelinePlot;
use crate::plot::polar::PolarPlot;
use crate::plot::ternary::TernaryPlot;
use crate::plot::diceplot::DicePlot;
use crate::plot::forest::ForestPlot;
use crate::plot::scatter3d::Scatter3DPlot;
use crate::plot::surface3d::Surface3DPlot;
use crate::plot::clustermap::Clustermap;
use crate::plot::jointplot::JointPlot;
use crate::plot::raincloud::RaincloudPlot;
use crate::plot::lollipop::LollipopPlot;
use crate::plot::survival::SurvivalPlot;
use crate::plot::roc::RocPlot;
use crate::plot::slope::SlopePlot;
use crate::plot::venn::VennPlot;
use crate::plot::parallel::ParallelPlot;
use crate::plot::mosaic::MosaicPlot;
use crate::plot::qq::QQPlot;
use crate::plot::network::NetworkPlot;
use crate::plot::streamgraph::StreamgraphPlot;
use crate::plot::radar::RadarPlot;
use crate::plot::hexbin::HexbinPlot;
use crate::plot::treemap::TreemapPlot;
use crate::plot::sunburst::SunburstPlot;
use crate::plot::bump::BumpPlot;
use crate::plot::funnel::FunnelPlot;
use crate::plot::rose::RosePlot;
use crate::plot::calendar::CalendarPlot;
use crate::plot::legend::ColorBarInfo;
use crate::render::render_utils;


pub enum Plot {
    Scatter(ScatterPlot),
    Line(LinePlot),
    Bar(BarPlot),
    Histogram(Histogram),
    Histogram2d(Histogram2D),
    Box(BoxPlot),
    Violin(ViolinPlot),
    Series(SeriesPlot),
    Pie(PiePlot),
    Heatmap(Heatmap),
    Brick(BrickPlot),
    Band(BandPlot),
    Waterfall(WaterfallPlot),
    Strip(StripPlot),
    Volcano(VolcanoPlot),
    Manhattan(ManhattanPlot),
    DotPlot(DotPlot),
    UpSet(UpSetPlot),
    StackedArea(StackedAreaPlot),
    Candlestick(CandlestickPlot),
    Contour(ContourPlot),
    Chord(ChordPlot),
    Sankey(SankeyPlot),
    PhyloTree(PhyloTree),
    Synteny(SyntenyPlot),
    Density(DensityPlot),
    Ridgeline(RidgelinePlot),
    Polar(PolarPlot),
    Ternary(TernaryPlot),
    DicePlot(DicePlot),
    Forest(ForestPlot),
    Scatter3D(Scatter3DPlot),
    Surface3D(Surface3DPlot),
    Clustermap(Clustermap),
    Joint(JointPlot),
    Raincloud(RaincloudPlot),
    Lollipop(LollipopPlot),
    Survival(SurvivalPlot),
    Roc(RocPlot),
    Slope(SlopePlot),
    Venn(VennPlot),
    Parallel(ParallelPlot),
    Mosaic(MosaicPlot),
    Ecdf(EcdfPlot),
    QQ(QQPlot),
    Network(NetworkPlot),
    Streamgraph(StreamgraphPlot),
    Radar(RadarPlot),
    Hexbin(HexbinPlot),
    Treemap(TreemapPlot),
    Sunburst(SunburstPlot),
    Bump(BumpPlot),
    Funnel(FunnelPlot),
    Rose(RosePlot),
    Calendar(CalendarPlot),
}

impl From<ScatterPlot>    for Plot { fn from(p: ScatterPlot)    -> Self { Plot::Scatter(p) } }
impl From<LinePlot>       for Plot { fn from(p: LinePlot)       -> Self { Plot::Line(p) } }
impl From<BarPlot>        for Plot { fn from(p: BarPlot)        -> Self { Plot::Bar(p) } }
impl From<Histogram>      for Plot { fn from(p: Histogram)      -> Self { Plot::Histogram(p) } }
impl From<Histogram2D>    for Plot { fn from(p: Histogram2D)    -> Self { Plot::Histogram2d(p) } }
impl From<BoxPlot>        for Plot { fn from(p: BoxPlot)        -> Self { Plot::Box(p) } }
impl From<ViolinPlot>     for Plot { fn from(p: ViolinPlot)     -> Self { Plot::Violin(p) } }
impl From<SeriesPlot>     for Plot { fn from(p: SeriesPlot)     -> Self { Plot::Series(p) } }
impl From<PiePlot>        for Plot { fn from(p: PiePlot)        -> Self { Plot::Pie(p) } }
impl From<Heatmap>        for Plot { fn from(p: Heatmap)        -> Self { Plot::Heatmap(p) } }
impl From<BrickPlot>      for Plot { fn from(p: BrickPlot)      -> Self { Plot::Brick(p) } }
impl From<BandPlot>       for Plot { fn from(p: BandPlot)       -> Self { Plot::Band(p) } }
impl From<WaterfallPlot>  for Plot { fn from(p: WaterfallPlot)  -> Self { Plot::Waterfall(p) } }
impl From<StripPlot>      for Plot { fn from(p: StripPlot)      -> Self { Plot::Strip(p) } }
impl From<VolcanoPlot>    for Plot { fn from(p: VolcanoPlot)    -> Self { Plot::Volcano(p) } }
impl From<ManhattanPlot>  for Plot { fn from(p: ManhattanPlot)  -> Self { Plot::Manhattan(p) } }
impl From<DotPlot>        for Plot { fn from(p: DotPlot)        -> Self { Plot::DotPlot(p) } }
impl From<UpSetPlot>      for Plot { fn from(p: UpSetPlot)      -> Self { Plot::UpSet(p) } }
impl From<StackedAreaPlot> for Plot { fn from(p: StackedAreaPlot) -> Self { Plot::StackedArea(p) } }
impl From<CandlestickPlot> for Plot { fn from(p: CandlestickPlot) -> Self { Plot::Candlestick(p) } }
impl From<ContourPlot>    for Plot { fn from(p: ContourPlot)    -> Self { Plot::Contour(p) } }
impl From<ChordPlot>      for Plot { fn from(p: ChordPlot)      -> Self { Plot::Chord(p) } }
impl From<SankeyPlot>     for Plot { fn from(p: SankeyPlot)     -> Self { Plot::Sankey(p) } }
impl From<PhyloTree>      for Plot { fn from(p: PhyloTree)      -> Self { Plot::PhyloTree(p) } }
impl From<SyntenyPlot>    for Plot { fn from(p: SyntenyPlot)    -> Self { Plot::Synteny(p) } }
impl From<DensityPlot>   for Plot { fn from(p: DensityPlot)   -> Self { Plot::Density(p) } }
impl From<RidgelinePlot> for Plot { fn from(p: RidgelinePlot) -> Self { Plot::Ridgeline(p) } }
impl From<PolarPlot>     for Plot { fn from(p: PolarPlot)     -> Self { Plot::Polar(p) } }
impl From<TernaryPlot>   for Plot { fn from(p: TernaryPlot)   -> Self { Plot::Ternary(p) } }
impl From<DicePlot>      for Plot { fn from(p: DicePlot)      -> Self { Plot::DicePlot(p) } }
impl From<ForestPlot>    for Plot { fn from(p: ForestPlot)    -> Self { Plot::Forest(p) } }
impl From<Scatter3DPlot> for Plot { fn from(p: Scatter3DPlot) -> Self { Plot::Scatter3D(p) } }
impl From<Surface3DPlot> for Plot { fn from(p: Surface3DPlot) -> Self { Plot::Surface3D(p) } }
impl From<Clustermap>   for Plot { fn from(p: Clustermap)   -> Self { Plot::Clustermap(p) } }
impl From<JointPlot>    for Plot { fn from(p: JointPlot)    -> Self { Plot::Joint(p) } }
impl From<RaincloudPlot> for Plot { fn from(p: RaincloudPlot) -> Self { Plot::Raincloud(p) } }
impl From<LollipopPlot>  for Plot { fn from(p: LollipopPlot)  -> Self { Plot::Lollipop(p) } }
impl From<SurvivalPlot>  for Plot { fn from(p: SurvivalPlot)  -> Self { Plot::Survival(p) } }
impl From<RocPlot>       for Plot { fn from(p: RocPlot)       -> Self { Plot::Roc(p) } }
impl From<SlopePlot>     for Plot { fn from(p: SlopePlot)     -> Self { Plot::Slope(p) } }
impl From<VennPlot>        for Plot { fn from(p: VennPlot)        -> Self { Plot::Venn(p) } }
impl From<ParallelPlot>    for Plot { fn from(p: ParallelPlot)    -> Self { Plot::Parallel(p) } }
impl From<MosaicPlot>      for Plot { fn from(p: MosaicPlot)      -> Self { Plot::Mosaic(p) } }
impl From<EcdfPlot>        for Plot { fn from(p: EcdfPlot)        -> Self { Plot::Ecdf(p) } }
impl From<QQPlot>          for Plot { fn from(p: QQPlot)          -> Self { Plot::QQ(p) } }
impl From<NetworkPlot>     for Plot { fn from(p: NetworkPlot)     -> Self { Plot::Network(p) } }
impl From<StreamgraphPlot> for Plot { fn from(p: StreamgraphPlot) -> Self { Plot::Streamgraph(p) } }
impl From<RadarPlot>       for Plot { fn from(p: RadarPlot)       -> Self { Plot::Radar(p) } }
impl From<HexbinPlot>      for Plot { fn from(p: HexbinPlot)      -> Self { Plot::Hexbin(p) } }
impl From<TreemapPlot>     for Plot { fn from(p: TreemapPlot)     -> Self { Plot::Treemap(p) } }
impl From<SunburstPlot>    for Plot { fn from(p: SunburstPlot)    -> Self { Plot::Sunburst(p) } }
impl From<BumpPlot>        for Plot { fn from(p: BumpPlot)        -> Self { Plot::Bump(p) } }
impl From<FunnelPlot>      for Plot { fn from(p: FunnelPlot)      -> Self { Plot::Funnel(p) } }
impl From<RosePlot>        for Plot { fn from(p: RosePlot)        -> Self { Plot::Rose(p) } }
impl From<CalendarPlot>    for Plot { fn from(p: CalendarPlot)    -> Self { Plot::Calendar(p) } }

use crate::plot::plot3d::DataRanges3D;
use crate::plot::colormap::ColorMap;

fn colorbar_from_z(cmap: &ColorMap, ranges: DataRanges3D, label: Option<String>) -> Option<ColorBarInfo> {
    let (z_min, z_max) = ranges.z;
    if !z_min.is_finite() || !z_max.is_finite() { return None; }
    let cmap = cmap.clone();
    Some(ColorBarInfo {
        map_fn: Arc::new(move |t| {
            let norm = (t - z_min) / (z_max - z_min + f64::EPSILON);
            cmap.map(norm.clamp(0.0, 1.0))
        }),
        min_value: z_min,
        max_value: z_max,
        label,
        tick_labels: None,
    })
}

fn bounds_from_2d<I>(points: I) -> Option<((f64, f64), (f64, f64))>
    where
        I: IntoIterator,
        I::Item: Into<(f64, f64)>,
    {
    let mut iter = points.into_iter().map(Into::into);
    let (x0, y0) = iter.next()?;
    let (mut x_min, mut x_max) = (x0, x0);
    let (mut y_min, mut y_max) = (y0, y0);
    for (x, y) in iter {
        x_min = x_min.min(x);
        x_max = x_max.max(x);
        y_min = y_min.min(y);
        y_max = y_max.max(y);
    }
    Some(((x_min, x_max), (y_min, y_max)))
}

fn _bounds_from_1d(points: &[f64]) -> Option<((f64, f64), (f64, f64))> {
    if points.is_empty() {
        return None;
    }
    let (mut min_val, mut max_val) = (points[0], points[0]);
    for i in points {
        min_val = min_val.min(*i);
        max_val = max_val.max(*i);
       
    }
    
    Some(((0.0f64, points.len() as f64), (min_val, max_val)))
}



impl Plot {
    /// Set the primary color for single-color plot types.
    /// Multi-element plots (Bar, Pie, Brick) and grid plots (Heatmap, Histogram2d) are skipped.
    pub fn set_color(&mut self, color: &str) {
        match self {
            Plot::Scatter(s) => s.color = color.into(),
            Plot::Line(l) => l.color = color.into(),
            Plot::Series(s) => s.color = color.into(),
            Plot::Histogram(h) => h.color = color.into(),
            Plot::Box(b) => b.color = color.into(),
            Plot::Violin(v) => v.color = color.into(),
            Plot::Band(b) => b.color = color.into(),
            Plot::Strip(s) => s.color = color.into(),
            Plot::Density(d) => d.color = color.into(),
            Plot::Forest(f) => f.color = color.into(),
            Plot::Scatter3D(s) => s.color = color.into(),
            Plot::Surface3D(s) => s.color = color.into(),
            Plot::Raincloud(r) => r.color = color.into(),
            Plot::Lollipop(l) => l.color = color.into(),
            Plot::Survival(s) => s.color = color.into(),
            Plot::Roc(r) => r.color = color.into(),
            Plot::Slope(s) => s.color = color.into(),
            Plot::Parallel(p) => p.color = color.into(),
            Plot::Ecdf(e) => e.color = color.into(),
            Plot::QQ(q) => q.color = color.into(),
            _ => {}  // multi-series plots (StackedArea, Streamgraph, etc.) skip palette auto-assign
        }
    }

    pub fn bounds(&self) -> Option<((f64, f64), (f64, f64))> {
        match self {
            
            Plot::Scatter(s) => {
                let ((mut x_min, mut x_max), (mut y_min, mut y_max)) = bounds_from_2d(&s.data)?;

                // Expand with error bars
                for point in &s.data {
                    let x_lo = point.x - point.x_err.map_or(0.0, |e| e.0);
                    let x_hi = point.x + point.x_err.map_or(0.0, |e| e.1);
                    let y_lo = point.y - point.y_err.map_or(0.0, |e| e.0);
                    let y_hi = point.y + point.y_err.map_or(0.0, |e| e.1);

                    x_min = x_min.min(x_lo);
                    x_max = x_max.max(x_hi);
                    y_min = y_min.min(y_lo);
                    y_max = y_max.max(y_hi);
                }

                // Expand for band
                if let Some(ref band) = s.band {
                    for &y in &band.y_lower { y_min = y_min.min(y); }
                    for &y in &band.y_upper { y_max = y_max.max(y); }
                }

                // Expand for trend line
                if let Some(trend) = s.trend {
                    let TrendLine::Linear = trend;
                        if let Some((slope, intercept, _)) = render_utils::linear_regression(&s.data) {
                            let y_start = slope * x_min + intercept;
                            let y_end = slope * x_max + intercept;

                            y_min = y_min.min(y_start).min(y_end);
                            y_max = y_max.max(y_start).max(y_end);
                        }
                }

                Some(((x_min, x_max), (y_min, y_max)))
            },
            Plot::Line(p) => {
                let ((x_min, x_max), (mut y_min, mut y_max)) = bounds_from_2d(&p.data)?;
                if let Some(ref band) = p.band {
                    for &y in &band.y_lower { y_min = y_min.min(y); }
                    for &y in &band.y_upper { y_max = y_max.max(y); }
                }
                Some(((x_min, x_max), (y_min, y_max)))
            }
            // Plot::Series(s) => bounds_from_1d(&s.values),
            Plot::Series(sp) => {
                if sp.values.is_empty() {
                    None
                } else {
                    let x_min = 0.0;
                    let x_max = sp.values.len() as f64 - 1.0;
            
                    let mut y_min = f64::INFINITY;
                    let mut y_max = f64::NEG_INFINITY;
            
                    for &v in &sp.values {
                        y_min = y_min.min(v);
                        y_max = y_max.max(v);
                    }
            
                    Some(((x_min, x_max), (y_min, y_max)))
                }
            }
            Plot::Bar(bp) => {

                if bp.groups.is_empty() {
                    None
                }
                else {
                    let x_min = 0.5;
                    let x_max = bp.groups.len() as f64 + 0.5;
                    let y_min = 0.0;

                    let mut y_max = f64::NEG_INFINITY;
                    if bp.stacked {
                        for group in &bp.groups {
                            let sum: f64 = group.bars.iter().map(|b| b.value).sum();
                            y_max = y_max.max(sum);
                        }
                    } else {
                        for group in &bp.groups {
                            for bar in &group.bars {
                                y_max = y_max.max(bar.value);
                            }
                        }
                    }

                    Some(((x_min, x_max), (y_min, y_max)))
                }
            }
            Plot::Histogram(h) => {
                // Precomputed path: derive bounds from edges and counts directly
                if let Some((edges, counts)) = &h.precomputed {
                    if edges.len() < 2 || counts.is_empty() { return None; }
                    let x_min = edges[0];
                    let x_max = *edges.last().unwrap();
                    let max_y = if h.normalize {
                        1.0
                    } else {
                        counts.iter().cloned().fold(0.0_f64, f64::max)
                    };
                    return Some(((x_min, x_max), (0.0, max_y)));
                }
                // Auto-binning path: use explicit range if set, else derive from data
                // (mirrors the fallback in the renderer so bounds() always returns a usable range)
                let range = h.range.unwrap_or_else(|| {
                    if h.data.is_empty() { return (0.0, 1.0); }
                    let min = h.data.iter().cloned().fold(f64::INFINITY, f64::min);
                    let max = h.data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                    (min, max)
                });
                let bins = h.bins;
                let bin_width = (range.1 - range.0) / bins as f64;

                let mut counts = vec![0usize; bins];
                for &value in &h.data {
                    if value < range.0 || value > range.1 {
                        continue;
                    }
                    let bin = ((value - range.0) / bin_width).floor() as usize;
                    let bin = if bin == bins { bin - 1 } else { bin };
                    counts[bin] += 1;
                }

                let max_y = if h.normalize {
                    1.0
                } else {
                    *counts.iter().max().unwrap_or(&1) as f64
                };

                Some((range, (0.0, max_y)))
            }
            Plot::Box(bp) => {
                if bp.groups.is_empty() {
                    None
                }
                else {
                    let x_min = 0.5;
                    let x_max = bp.groups.len() as f64 + 0.5;
            
                    let mut y_min = f64::INFINITY;
                    let mut y_max = f64::NEG_INFINITY;
                    for g in &bp.groups {
                        if g.values.is_empty() { continue; }
                        let mut vals = g.values.clone();
                        vals.sort_by(|a, b| a.total_cmp(b));
                        let q1 = render_utils::percentile(&vals, 25.0);
                        let q3 = render_utils::percentile(&vals, 75.0);
                        let iqr = q3 - q1;
                        let lo = q1 - 1.5 * iqr;
                        let hi = q3 + 1.5 * iqr;
                        y_min = y_min.min(lo);
                        y_max = y_max.max(hi);
                    }
            
                    Some(((x_min, x_max), (y_min, y_max)))
                }
            }
            Plot::Violin(vp) => {
                if vp.groups.is_empty() {
                    None
                } else {
                    let x_min = 0.5;
                    let x_max = vp.groups.len() as f64 + 0.5;

                    let mut y_min = f64::INFINITY;
                    let mut y_max = f64::NEG_INFINITY;

                    for group in &vp.groups {
                        if group.values.is_empty() { continue; }
                        let g_min = group.values.iter().cloned().fold(f64::INFINITY, f64::min);
                        let g_max = group.values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                        let h = vp.bandwidth.unwrap_or_else(|| render_utils::silverman_bandwidth(&group.values));
                        y_min = y_min.min(g_min - 3.0 * h);
                        y_max = y_max.max(g_max + 3.0 * h);
                    }

                    Some(((x_min, x_max), (y_min, y_max)))
                }
            }
            Plot::Pie(_) => {
                // Centered at (0.0, 0.0) and rendered to fit the layout box
                Some(((-1.0, 1.0), (-1.0, 1.0))) 
            }
            Plot::Heatmap(hm) => {
                let rows = hm.data.len();
                let cols = hm.data.first().map_or(0, |row| row.len());
                // Use 0.5-based bounds so integer positions (1, 2, ...) land on
                // cell centres, matching DotPlot convention and making the axis
                // label code (y_val = i + 1.0) align with cell midpoints.
                Some(((0.5, cols as f64 + 0.5), (0.5, rows as f64 + 0.5)))
            }
            Plot::Histogram2d(h2d) => {
                // Return the physical axis range so the layout is calibrated in
                // data coordinates, matching the physical coords used by the renderer.
                Some(((h2d.x_range.0, h2d.x_range.1), (h2d.y_range.0, h2d.y_range.1)))
            }
            Plot::Band(b) => {
                if b.x.is_empty() { return None; }
                let x_min = b.x.iter().cloned().fold(f64::INFINITY, f64::min);
                let x_max = b.x.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let y_min = b.y_lower.iter().cloned().fold(f64::INFINITY, f64::min);
                let y_max = b.y_upper.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                Some(((x_min, x_max), (y_min, y_max)))
            }
            Plot::Waterfall(wp) => {
                if wp.bars.is_empty() { return None; }
                let x_min = 0.5;
                let x_max = wp.bars.len() as f64 + 0.5;
                let mut running = 0.0_f64;
                let mut y_min = 0.0_f64;
                let mut y_max = 0.0_f64;
                for bar in &wp.bars {
                    match bar.kind {
                        WaterfallKind::Delta => {
                            let base = running;
                            running += bar.value;
                            y_min = y_min.min(base).min(running);
                            y_max = y_max.max(base).max(running);
                        }
                        WaterfallKind::Total => {
                            y_min = y_min.min(0.0).min(running);
                            y_max = y_max.max(0.0).max(running);
                        }
                        WaterfallKind::Difference { from, to } => {
                            y_min = y_min.min(from).min(to);
                            y_max = y_max.max(from).max(to);
                        }
                    }
                }
                Some(((x_min, x_max), (y_min, y_max)))
            }
            Plot::Strip(sp) => {
                if sp.groups.is_empty() { return None; }
                let x_min = 0.5;
                let x_max = sp.groups.len() as f64 + 0.5;
                let mut y_min = f64::INFINITY;
                let mut y_max = f64::NEG_INFINITY;
                for g in &sp.groups {
                    for &v in &g.values {
                        y_min = y_min.min(v);
                        y_max = y_max.max(v);
                    }
                }
                if y_min == f64::INFINITY { return None; }
                Some(((x_min, x_max), (y_min, y_max)))
            }
            Plot::Volcano(vp) => {
                if vp.points.is_empty() { return None; }
                let floor = vp.floor();
                let mut x_min = f64::INFINITY;
                let mut x_max = f64::NEG_INFINITY;
                let mut y_max = f64::NEG_INFINITY;
                for p in &vp.points {
                    x_min = x_min.min(p.log2fc);
                    x_max = x_max.max(p.log2fc);
                    let y = -(p.pvalue.max(floor)).log10();
                    y_max = y_max.max(y);
                }
                Some(((x_min, x_max), (0.0, y_max)))
            }
            Plot::Manhattan(mp) => {
                if mp.points.is_empty() { return None; }
                let floor = mp.floor();
                let x_min = mp.spans.iter().map(|s| s.x_start).fold(f64::INFINITY, f64::min);
                let x_max = mp.spans.iter().map(|s| s.x_end).fold(f64::NEG_INFINITY, f64::max);
                if !x_min.is_finite() { return None; }
                // Ensure genome-wide threshold is always visible
                let y_max = mp.points.iter()
                    .map(|p| -(p.pvalue.max(floor)).log10())
                    .fold(mp.genome_wide, f64::max);
                Some(((x_min, x_max), (0.0, y_max)))
            }
            Plot::DotPlot(dp) => {
                if dp.x_categories.is_empty() { return None; }
                Some(((0.5, dp.x_categories.len() as f64 + 0.5),
                      (0.5, dp.y_categories.len() as f64 + 0.5)))
            }
            Plot::DicePlot(dp) => {
                if dp.x_categories.is_empty() { return None; }
                Some(((0.5, dp.x_categories.len() as f64 + 0.5),
                      (0.5, dp.y_categories.len() as f64 + 0.5)))
            }
            Plot::UpSet(_) => {
                // Dummy bounds — UpSet renders in pixel space and ignores map_x/map_y.
                Some(((0.0, 1.0), (0.0, 1.0)))
            }
            Plot::StackedArea(sa) => {
                if sa.x.is_empty() || sa.series.is_empty() { return None; }
                let x_min = sa.x.iter().cloned().fold(f64::INFINITY, f64::min);
                let x_max = sa.x.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let n = sa.x.len();
                let y_max = if sa.normalized {
                    100.0
                } else {
                    (0..n)
                        .map(|i| sa.series.iter().map(|s| s.get(i).copied().unwrap_or(0.0)).sum::<f64>())
                        .fold(0.0_f64, f64::max)
                };
                Some(((x_min, x_max), (0.0, y_max)))
            }
            Plot::Candlestick(cp) => {
                if cp.candles.is_empty() { return None; }
                let continuous = cp.candles.iter().any(|c| c.x.is_some());
                let (x_min, x_max) = if continuous {
                    (
                        cp.candles.iter().filter_map(|c| c.x).fold(f64::INFINITY, f64::min),
                        cp.candles.iter().filter_map(|c| c.x).fold(f64::NEG_INFINITY, f64::max),
                    )
                } else {
                    (0.5, cp.candles.len() as f64 + 0.5)
                };
                let y_min = cp.candles.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
                let y_max = cp.candles.iter().map(|c| c.high).fold(f64::NEG_INFINITY, f64::max);
                Some(((x_min, x_max), (y_min, y_max)))
            }
            Plot::Contour(cp) => {
                if cp.z.is_empty() { return None; }
                let x_min = cp.x_coords.iter().cloned().fold(f64::INFINITY, f64::min);
                let x_max = cp.x_coords.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let y_min = cp.y_coords.iter().cloned().fold(f64::INFINITY, f64::min);
                let y_max = cp.y_coords.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                Some(((x_min, x_max), (y_min, y_max)))
            }
            Plot::Chord(_) => {
                // Rendered in pixel space; dummy bounds satisfy Layout::auto_from_plots.
                Some(((0.0, 1.0), (0.0, 1.0)))
            }
            Plot::Sankey(_) => {
                // Rendered in pixel space; dummy bounds satisfy Layout::auto_from_plots.
                Some(((0.0, 1.0), (0.0, 1.0)))
            }
            Plot::PhyloTree(_) => {
                // Rendered in pixel space; dummy bounds satisfy Layout::auto_from_plots.
                Some(((0.0, 1.0), (0.0, 1.0)))
            }
            Plot::Synteny(_) => {
                // Rendered in pixel space; dummy bounds satisfy Layout::auto_from_plots.
                Some(((0.0, 1.0), (0.0, 1.0)))
            }
            Plot::Density(dp) => {
                // Use precomputed curve if available
                if let Some((xs, ys)) = &dp.precomputed {
                    if xs.is_empty() { return None; }
                    let x_min = xs.iter().cloned().fold(f64::INFINITY, f64::min);
                    let x_max = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                    let y_max = ys.iter().cloned().fold(0.0_f64, f64::max);
                    return Some(((x_min, x_max), (0.0, y_max * 1.1)));
                }
                if dp.data.len() < 2 { return None; }
                let bw = dp.bandwidth.unwrap_or_else(|| render_utils::silverman_bandwidth(&dp.data));
                let data_min = dp.data.iter().cloned().fold(f64::INFINITY, f64::min);
                let data_max = dp.data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let x_min = dp.x_lo.unwrap_or(data_min - 3.0 * bw);
                let x_max = dp.x_hi.unwrap_or(data_max + 3.0 * bw);
                // Use the same KDE path as the renderer (including reflection) so
                // bounds() and the rendered curve agree on the peak y value.
                let n = dp.data.len() as f64;
                let norm = 1.0 / (n * bw * (2.0 * std::f64::consts::PI).sqrt());
                let curve = if dp.x_lo.is_some() || dp.x_hi.is_some() {
                    render_utils::simple_kde_reflect(&dp.data, bw, dp.kde_samples,
                        x_min, x_max, dp.x_lo.is_some(), dp.x_hi.is_some())
                } else {
                    render_utils::simple_kde(&dp.data, bw, dp.kde_samples)
                };
                let y_max_pdf = curve.iter()
                    .map(|(_, y)| y * norm)
                    .fold(0.0_f64, f64::max);
                Some(((x_min, x_max), (0.0, y_max_pdf * 1.1)))
            }
            Plot::Ridgeline(rp) => {
                if rp.groups.is_empty() { return None; }
                let n = rp.groups.len() as f64;
                let mut x_min = f64::INFINITY;
                let mut x_max = f64::NEG_INFINITY;
                for g in &rp.groups {
                    if g.values.is_empty() { continue; }
                    let bw = rp.bandwidth.unwrap_or_else(|| render_utils::silverman_bandwidth(&g.values));
                    let gmin = g.values.iter().cloned().fold(f64::INFINITY, f64::min);
                    let gmax = g.values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                    x_min = x_min.min(gmin - 3.0 * bw);
                    x_max = x_max.max(gmax + 3.0 * bw);
                }
                if !x_min.is_finite() { return None; }
                // y_max must leave room for the top ridge to extend (1+overlap)
                // data units above group 0's center (at y = n).  Half a unit of
                // additional padding keeps it off the very top of the plot area.
                Some(((x_min, x_max), (0.5, n + 1.5 + rp.overlap)))
            }
            Plot::Polar(_) => {
                // Rendered in pixel space; dummy bounds satisfy Layout::auto_from_plots.
                Some(((-1.0, 1.0), (-1.0, 1.0)))
            }
            Plot::Ternary(_) => {
                // Rendered in pixel space; dummy bounds satisfy Layout::auto_from_plots.
                Some(((-1.0, 1.0), (-1.0, 1.0)))
            }
            Plot::Brick(bp) => {
                let rows = if let Some(ref exp) = bp.strigar_exp {
                    exp.len()
                } else {
                    bp.sequences.len()
                };

                let n_rows = rows;

                // STR width for row i (in data units, excluding flanks).
                let str_width = |i: usize| -> f64 {
                    if let Some(ref exp) = bp.strigar_exp {
                        if let Some(ref ml) = bp.motif_lengths {
                            exp.get(i).map(|s| {
                                s.chars().map(|c| *ml.get(&c).unwrap_or(&1) as f64).sum::<f64>()
                            }).unwrap_or(0.0)
                        } else {
                            exp.get(i).map(|s| s.len() as f64).unwrap_or(0.0)
                        }
                    } else {
                        bp.sequences.get(i).map(|s| s.len() as f64).unwrap_or(0.0)
                    }
                };
                let left_len = |i: usize| -> f64 {
                    bp.left_flanks.as_ref()
                        .and_then(|f| f.get(i))
                        .map(|s| s.chars().count() as f64)
                        .unwrap_or(0.0)
                };
                let right_len = |i: usize| -> f64 {
                    bp.right_flanks.as_ref()
                        .and_then(|f| f.get(i))
                        .map(|s| s.chars().count() as f64)
                        .unwrap_or(0.0)
                };

                // For right-anchor, all trailing edges align at max(str_width + right_len).
                // The right-align shift per row is max_right - row_right, which moves shorter
                // rows rightward. x_lo / x_hi must account for this shift.
                use crate::plot::BrickAnchor;
                let right_edges: Vec<f64> = (0..n_rows).map(|i| str_width(i) + right_len(i)).collect();
                let max_right = right_edges.iter().cloned().fold(0.0_f64, f64::max);
                let ra_shift = |i: usize| -> f64 {
                    if bp.anchor == BrickAnchor::Right { max_right - right_edges[i] } else { 0.0 }
                };

                let row_base_off = |i: usize| -> f64 {
                    let per_row = if let Some(ref offsets) = bp.x_offsets {
                        offsets.get(i).copied().flatten().unwrap_or(bp.x_offset)
                    } else {
                        bp.x_offset
                    };
                    per_row + bp.x_origin
                };

                // x extent: from leftmost flank to rightmost trailing edge across all rows.
                let mut lo = f64::INFINITY;
                let mut hi = f64::NEG_INFINITY;
                for i in 0..n_rows {
                    let eff_off = row_base_off(i) - ra_shift(i);
                    lo = lo.min(-left_len(i) - eff_off);
                    hi = hi.max(str_width(i) + right_len(i) - eff_off);
                }
                if !lo.is_finite() { lo = 0.0; }
                if !hi.is_finite() { hi = 1.0; }

                Some(((lo, hi), (0.0, rows as f64)))
            }
            Plot::Forest(fp) => {
                if fp.rows.is_empty() { return None; }
                let n = fp.rows.len();
                let y_min = 0.5;
                let y_max = n as f64 + 0.5;
                let mut x_min = f64::INFINITY;
                let mut x_max = f64::NEG_INFINITY;
                for row in &fp.rows {
                    x_min = x_min.min(row.ci_lower);
                    x_max = x_max.max(row.ci_upper);
                }
                // Include null value in x range so the reference line is visible
                if let Some(nv) = fp.null_value {
                    x_min = x_min.min(nv);
                    x_max = x_max.max(nv);
                }
                if !x_min.is_finite() { return None; }
                Some(((x_min, x_max), (y_min, y_max)))
            }
            Plot::Scatter3D(_) | Plot::Surface3D(_) => {
                Some(((-1.0, 1.0), (-1.0, 1.0)))
            }
            // Pixel-space plot — returns dummy bounds so Layout gets a valid range.
            Plot::Clustermap(_) => Some(((0.0, 1.0), (0.0, 1.0))),
            // Pixel-space composite plot; layout supplied internally via render_multiple.
            Plot::Joint(_) => Some(((-1.0, 1.0), (-1.0, 1.0))),
            Plot::Raincloud(r) => {
                let n = r.groups.len();
                if n == 0 { return None; }
                let all_vals: Vec<f64> = r.groups.iter()
                    .flat_map(|g| g.values.iter().copied())
                    .collect();
                if all_vals.is_empty() { return None; }
                let y_min = all_vals.iter().cloned().fold(f64::INFINITY, f64::min);
                let y_max = all_vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let pad = (y_max - y_min) * 0.05 + 0.5;
                Some(((0.5, n as f64 + 0.5), (y_min - pad, y_max + pad)))
            }
            Plot::Survival(sp) => {
                if sp.groups.is_empty() { return None; }
                let t_max = sp.groups.iter()
                    .flat_map(|g| g.times.iter().copied())
                    .fold(0.0_f64, f64::max);
                if t_max <= 0.0 { return None; }
                // y from 0 to 1 with small padding; x from 0 to t_max with padding
                Some(((0.0, t_max), (0.0, 1.0)))
            }
            Plot::Lollipop(lp) => {
                if lp.points.is_empty() { return None; }
                let mut x_min = f64::INFINITY;
                let mut x_max = f64::NEG_INFINITY;
                let mut y_min = lp.baseline;
                let mut y_max = lp.baseline;
                for p in &lp.points {
                    x_min = x_min.min(p.x);
                    x_max = x_max.max(p.x);
                    y_min = y_min.min(p.y);
                    y_max = y_max.max(p.y);
                }
                for d in &lp.domains {
                    x_min = x_min.min(d.x_start);
                    x_max = x_max.max(d.x_end);
                }
                if !lp.domains.is_empty() {
                    y_min = y_min.min(lp.baseline - lp.domain_height);
                }
                if !x_min.is_finite() { return None; }
                Some(((x_min, x_max), (y_min, y_max)))
            }
            Plot::Roc(_) => Some(((0.0, 1.0), (0.0, 1.0))),
            Plot::Slope(s) => {
                let n = s.points.len();
                if n == 0 { return None; }
                let mut x_min = f64::INFINITY;
                let mut x_max = f64::NEG_INFINITY;
                for p in &s.points {
                    x_min = x_min.min(p.before).min(p.after);
                    x_max = x_max.max(p.before).max(p.after);
                }
                if !x_min.is_finite() { return None; }
                let pad = (x_max - x_min) * 0.08 + 1e-9;
                Some(((x_min - pad, x_max + pad), (0.5, n as f64 + 0.5)))
            }
            // Pixel-space plot — dummy bounds so auto_from_plots sees it
            Plot::Venn(_) => Some(((-1.0, 1.0), (-1.0, 1.0))),
            // Pixel-space plot — dummy bounds so auto_from_plots sees it
            Plot::Parallel(_) => Some(((-1.0, 1.0), (-1.0, 1.0))),
            // Pixel-space plot — no axis bounds needed
            Plot::Mosaic(_) => None,
            Plot::Ecdf(ep) => {
                if ep.groups.is_empty() { return None; }
                let mut x_min = f64::INFINITY;
                let mut x_max = f64::NEG_INFINITY;
                for group in &ep.groups {
                    for &v in &group.data {
                        x_min = x_min.min(v);
                        x_max = x_max.max(v);
                    }
                }
                if !x_min.is_finite() { return None; }
                Some(((x_min, x_max), (0.0, 1.0)))
            }
            Plot::QQ(qp) => {
                use crate::plot::qq::QQMode;
                use crate::render::render_utils::probit;
                if qp.groups.is_empty() { return None; }
                match qp.mode {
                    QQMode::Normal => {
                        let n_max = qp.groups.iter().map(|g| g.data.len()).max().unwrap_or(0);
                        if n_max == 0 { return None; }
                        let th_min = probit(0.5 / n_max as f64);
                        let th_max = probit(1.0 - 0.5 / n_max as f64);
                        let mut y_min = f64::INFINITY;
                        let mut y_max = f64::NEG_INFINITY;
                        for g in &qp.groups {
                            for &v in &g.data {
                                y_min = y_min.min(v);
                                y_max = y_max.max(v);
                            }
                        }
                        if !y_min.is_finite() { return None; }
                        Some(((th_min, th_max), (y_min, y_max)))
                    }
                    QQMode::Genomic => {
                        let n_max = qp.groups.iter().map(|g| g.data.len()).max().unwrap_or(0);
                        if n_max == 0 { return None; }
                        let x_max = (2.0 * n_max as f64).log10();
                        let mut y_max: f64 = 0.0;
                        for g in &qp.groups {
                            for &p in &g.data {
                                if p > 0.0 && p <= 1.0 {
                                    y_max = y_max.max(-p.log10());
                                }
                            }
                        }
                        Some(((0.0, x_max), (0.0, y_max)))
                    }
                }
            }
            Plot::Hexbin(hb) => {
                if hb.x.is_empty() { return None; }
                let x0 = hb.x_range.map(|(lo, _)| lo).unwrap_or_else(|| hb.x.iter().cloned().fold(f64::INFINITY, f64::min));
                let x1 = hb.x_range.map(|(_, hi)| hi).unwrap_or_else(|| hb.x.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
                let y0 = hb.y_range.map(|(lo, _)| lo).unwrap_or_else(|| hb.y.iter().cloned().fold(f64::INFINITY, f64::min));
                let y1 = hb.y_range.map(|(_, hi)| hi).unwrap_or_else(|| hb.y.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
                Some(((x0, x1), (y0, y1)))
            }
            // Pixel-space plots — dummy bounds so auto_from_plots sees them
            Plot::Treemap(_)  => Some(((-1.0, 1.0), (-1.0, 1.0))),
            Plot::Sunburst(_) => Some(((-1.0, 1.0), (-1.0, 1.0))),
            Plot::Funnel(_)   => Some(((-1.0, 1.0), (-1.0, 1.0))),
            Plot::Rose(_)     => Some(((-1.0, 1.0), (-1.0, 1.0))),
            Plot::Calendar(_) => Some(((-1.0, 1.0), (-1.0, 1.0))),
            Plot::Bump(bp) => {
                let n = bp.total_series_count();
                let n_time = bp.n_time_points();
                if n == 0 || n_time == 0 {
                    Some(((0.5, 1.5), (0.5, 1.5)))
                } else {
                    Some(((0.5, n_time as f64 + 0.5), (0.5, n as f64 + 0.5)))
                }
            }
            // Rendered in pixel space; dummy bounds satisfy Layout::auto_from_plots.
            Plot::Network(_) => Some(((0.0, 1.0), (0.0, 1.0))),
            Plot::Radar(_) => Some(((0.0, 1.0), (0.0, 1.0))),
            Plot::Streamgraph(sg) => {
                let geom = sg.compute_geometry()?;
                let x_min = sg.x.iter().cloned().fold(f64::INFINITY, f64::min);
                let x_max = sg.x.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let y_min = geom.baseline.iter().cloned().fold(f64::INFINITY, f64::min);
                let y_max = geom.uppers.last()
                    .map(|u| u.iter().cloned().fold(f64::NEG_INFINITY, f64::max))
                    .unwrap_or(0.0);
                Some(((x_min, x_max), (y_min, y_max)))
            }
        }
    }

    /// Rough upper-bound on the number of SVG primitives this plot will emit.
    /// Used to pre-allocate the Scene elements vector and avoid repeated reallocs.
    pub fn estimated_primitives(&self) -> usize {
        match self {
            Plot::Scatter(s) => {
                let n = s.data.len();
                let err = if s.data.iter().any(|p| p.x_err.is_some() || p.y_err.is_some()) { n * 3 } else { 0 };
                n + err + 10
            }
            Plot::Line(l) => l.data.len() / 10 + 10,
            Plot::Series(s) => s.values.len() / 10 + 10,
            Plot::Manhattan(m) => m.points.len() + m.spans.len() * 2 + 30,
            Plot::Heatmap(h) => {
                let cells: usize = h.data.iter().map(|r| r.len()).sum();
                (if h.show_values { cells * 2 } else { cells }) + 10
            }
            Plot::Histogram2d(h) => h.bins.iter().map(|r| r.len()).sum::<usize>() + 10,
            Plot::Violin(v) => v.groups.len() * 20 + 10,
            Plot::Bar(b) => b.groups.iter().map(|g| g.bars.len()).sum::<usize>() * 2 + 10,
            Plot::Histogram(h) => h.bins * 2 + 10,
            Plot::Brick(b) => {
                let rows = if b.strigar_exp.is_some() { b.strigar_exp.as_ref().map_or(0, |e| e.len()) } else { b.sequences.len() };
                let avg_cols = b.sequences.first().map_or(10, |s| s.len());
                rows * avg_cols + 10
            }
            Plot::Forest(f) => f.rows.len() * 4 + 5,
            Plot::Scatter3D(s) => s.data.len() + 70,
            Plot::Surface3D(s) => {
                let n = s.nrows().saturating_sub(1) * s.ncols().saturating_sub(1);
                n + 70
            }
            Plot::Clustermap(c) => {
                let cells: usize = c.data.iter().map(|r| r.len()).sum();
                cells + 500
            }
            Plot::Joint(jp) => {
                jp.groups.iter().map(|g| g.scatter.data.len() * 3 + 200).sum::<usize>() + 100
            }
            Plot::Raincloud(r) => {
                let total_pts: usize = r.groups.iter().map(|g| g.values.len()).sum();
                r.groups.len() * 30 + total_pts + 10
            }
            Plot::Survival(sp) => sp.groups.iter().map(|g| g.times.len() * 3 + 20).sum(),
            Plot::Lollipop(lp) => lp.points.len() * 2 + lp.domains.len() * 2 + 5,
            Plot::Roc(r) => r.groups.iter().map(|g| {
                g.raw_predictions.as_ref().map(|p| p.len()).unwrap_or(100) * 2 + 50
            }).sum::<usize>() + 10,
            Plot::Slope(s) => s.points.len() * 5 + 10,
            Plot::Venn(v) => v.sets.len() * 10 + 50,
            Plot::Parallel(p) => p.rows.len() + p.axis_names.len() * 10 + 50,
            Plot::Mosaic(mp) => {
                let nc = mp.effective_col_order().len();
                let nr = mp.effective_row_order().len();
                nc * nr * 2 + nc + nr + 30
            }
            Plot::Ecdf(ep) => {
                let n: usize = ep.groups.iter().map(|g| g.data.len()).sum();
                let band = if ep.show_confidence_band { n * 4 } else { 0 };
                let rug = if ep.show_rug { n } else { 0 };
                ep.groups.len() * 2 + n * 2 + band + rug + 20
            }
            Plot::QQ(qp) => {
                let n: usize = qp.groups.iter().map(|g| g.data.len()).sum();
                let band = if qp.show_ci_band { n * 4 } else { 0 };
                qp.groups.len() * 2 + n + band + 20
            }
            Plot::Network(n) => n.nodes.len() * 2 + n.edges.len() * 3 + 20,
            Plot::Radar(r) => r.series.len() * (r.axes.len() + 2) + r.grid_lines * r.axes.len() + 30,
            Plot::Streamgraph(sg) => {
                sg.series.len() * (sg.x.len() * 3 + 2) + 20
            }
            Plot::Hexbin(hb) => hb.n_bins * hb.n_bins / 2,
            Plot::Treemap(tm) => tm.node_count() * 3 + 10,
            Plot::Sunburst(sb) => sb.node_count() * 2 + 10,
            Plot::Bump(bp) => bp.total_series_count() * bp.n_time_points() * 3 + 20,
            Plot::Funnel(fp) => fp.stage_count() * 6 + 20,
            Plot::Rose(rp) => rp.n_sectors() * rp.series.len().max(1) * 2 + rp.grid_lines * 2 + rp.n_sectors() + 20,
            Plot::Calendar(cp) => cp.data.len() + 100,
            _ => 100,
        }
    }

    pub fn colorbar_info(&self) -> Option<ColorBarInfo> {
        match self {
            Plot::Heatmap(hm) => {
                let flat: Vec<f64> = hm.data.iter().flatten().cloned().collect();
                let min = flat.iter().cloned().fold(f64::INFINITY, f64::min);
                let max = flat.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let cmap = hm.color_map.clone();
                Some(ColorBarInfo {
                    map_fn: Arc::new(move |t| {
                        let norm = (t - min) / (max - min + f64::EPSILON);
                        cmap.map(norm.clamp(0.0, 1.0))
                    }),
                    min_value: min,
                    max_value: max,
                    label: None,
                    tick_labels: None,
                })
            }
            Plot::Histogram2d(h2d) => {
                let max_count = h2d.bins.iter().flatten().copied().max().unwrap_or(1) as f64;
                let cmap = h2d.color_map.clone();
                let log_scale = h2d.log_count;
                if log_scale {
                    // Colorbar in log₁₀ space: ticks at integer powers of 10 labelled
                    // with the actual count value so users can read off "this color = N cells".
                    let log_max = (max_count + 1.0).log10();
                    let tick_labels: Vec<(f64, String)> = {
                        let mut v = vec![(0.0_f64, "0".to_string())];
                        let mut k = 0u32;
                        loop {
                            let count = 10_f64.powi(k as i32);
                            if count > max_count { break; }
                            let pos = (count + 1.0).log10();
                            v.push((pos, format!("{}", count as u64)));
                            k += 1;
                        }
                        // Always include max_count at the top
                        v.push((log_max, format!("{}", max_count as u64)));
                        v.dedup_by(|a, b| (a.0 - b.0).abs() < 1e-9);
                        v
                    };
                    Some(ColorBarInfo {
                        map_fn: Arc::new(move |t| {
                            // t is a log₁₀ value in [0, log_max]
                            cmap.map((t / log_max).clamp(0.0, 1.0))
                        }),
                        min_value: 0.0,
                        max_value: log_max,
                        label: Some("log\u{2081}\u{2080}(Count + 1)".to_string()),
                        tick_labels: Some(tick_labels),
                    })
                } else {
                    Some(ColorBarInfo {
                        map_fn: Arc::new(move |t| {
                            cmap.map((t / max_count).clamp(0.0, 1.0))
                        }),
                        min_value: 0.0,
                        max_value: max_count,
                        label: Some("Count".to_string()),
                        tick_labels: None,
                    })
                }
            }
            Plot::DotPlot(dp) => {
                let label = dp.color_legend_label.clone()?;
                let (min, max) = dp.color_range.unwrap_or_else(|| dp.color_extent());
                let cmap = dp.color_map.clone();
                Some(ColorBarInfo {
                    map_fn: Arc::new(move |t| {
                        let norm = (t - min) / (max - min + f64::EPSILON);
                        cmap.map(norm.clamp(0.0, 1.0))
                    }),
                    min_value: min,
                    max_value: max,
                    label: Some(label),
                    tick_labels: None,
                })
            }
            Plot::DicePlot(dp) => {
                let label = dp.fill_legend_label.clone()?;
                let (min, max) = dp.fill_range.unwrap_or_else(|| dp.fill_extent());
                let cmap = dp.color_map.clone();
                Some(ColorBarInfo {
                    map_fn: Arc::new(move |t| {
                        let norm = (t - min) / (max - min + f64::EPSILON);
                        cmap.map(norm.clamp(0.0, 1.0))
                    }),
                    min_value: min,
                    max_value: max,
                    label: Some(label),
                    tick_labels: None,
                })
            }
            Plot::Contour(cp) => {
                if !cp.filled { return None; }
                let (z_min, z_max) = cp.z_range();
                if !z_min.is_finite() || !z_max.is_finite() { return None; }
                let cmap = cp.color_map.clone();
                let label = cp.legend_label.clone();
                Some(ColorBarInfo {
                    map_fn: Arc::new(move |t| {
                        let norm = (t - z_min) / (z_max - z_min + f64::EPSILON);
                        cmap.map(norm.clamp(0.0, 1.0))
                    }),
                    min_value: z_min,
                    max_value: z_max,
                    label,
                    tick_labels: None,
                })
            }
            Plot::Clustermap(cm) => {
                let flat: Vec<f64> = cm.data.iter().flatten().cloned().collect();
                if flat.is_empty() { return None; }
                let min = flat.iter().cloned().fold(f64::INFINITY, f64::min);
                let max = flat.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let cmap = cm.color_map.clone();
                Some(ColorBarInfo {
                    map_fn: Arc::new(move |t| {
                        let norm = (t - min) / (max - min + f64::EPSILON);
                        cmap.map(norm.clamp(0.0, 1.0))
                    }),
                    min_value: min,
                    max_value: max,
                    label: cm.legend_label.clone(),
                    tick_labels: None,
                })
            }
            Plot::Surface3D(s) => colorbar_from_z(s.z_colormap.as_ref()?, s.data_ranges()?, s.box3d.z_label.clone()),
            Plot::Scatter3D(s) => colorbar_from_z(s.z_colormap.as_ref()?, s.data_ranges()?, s.box3d.z_label.clone()),
            // Hexbin draws its own colorbar inside add_hexbin (values are only known
            // after binning).  Return None here so the generic colorbar loop in
            // render_multiple does not attempt to draw a second, placeholder bar.
            // The layout margin is set via the explicit has_colorbar check in layout.rs.
            _ => None,
        }
    }
}