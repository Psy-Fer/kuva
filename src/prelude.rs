//! Convenience re-exports for the most commonly used types.
//!
//! Add one line to your crate and everything you need for typical plot construction
//! is in scope:
//!
//! ```rust
//! use kuva::prelude::*;
//! ```

// ── Plot structs ─────────────────────────────────────────────────────────────
pub use crate::plot::{
    AnnotationTrack,
    BandPlot,
    BarPlot,
    Box3DConfig,
    BoxGroup,
    BoxPlot,
    BrickPlot,
    BumpPlot,
    BumpSeries,
    BumpTieBreak,
    CandleDataPoint,
    CandlestickPlot,
    ChordPlot,
    ChromSpan,
    Clustermap,
    ClustermapNorm,
    ColorMap,
    ContourPlot,
    CurveStyle as BumpCurveStyle,
    DensityPlot,
    DicePlot,
    DicePoint,
    DotPlot,
    DotPoint,
    EcdfGroup,
    EcdfPlot,
    ForestPlot,
    ForestRow,
    FunnelColorMode,
    FunnelOrientation,
    FunnelPlot,
    FunnelStage,
    GanttPlot,
    GanttTask,
    GenomeBuild,
    Heatmap,
    HexbinPlot,
    Histogram,
    Histogram2D,
    JointGroup,
    JointPlot,
    KMGroup,
    LabelStyle as VolcanoLabelStyle,
    LegendEntry,
    LegendPlot,
    // Legend
    LegendPosition,
    LegendShape,
    LinePlot,
    LineStyle,
    LollipopDomain,
    LollipopPlot,
    LollipopPoint,
    ManhattanPlot,
    ManhattanPoint,
    MarginalType,
    // Style / config types used when building plots
    MarkerShape,
    MosaicCell,
    MosaicPlot,
    NetworkEdge,
    NetworkLayout,
    NetworkNode,
    NetworkPlot,
    NodeShape,
    ParallelPlot,
    ParallelRow,
    PhyloNode,
    PhyloTree,
    PieLabelPosition,
    PiePlot,
    PieSlice,
    PolarMode,
    PolarPlot,
    PolarSeries,
    PopulationPyramid,
    PrGroup,
    PrPlot,
    PyramidMode,
    PyramidSeries,
    QQGroup,
    QQMode,
    QQPlot,
    QuiverArrow,
    QuiverPivot,
    QuiverPlot,
    RadarPlot,
    RadarReference,
    RadarSeries,
    RaincloudGroup,
    RaincloudPlot,
    RidgelineGroup,
    RidgelinePlot,
    RocGroup,
    RocPlot,
    RoseEncoding,
    RoseMode,
    RosePlot,
    RoseSeries,
    SankeyAlluvium,
    SankeyLink,
    SankeyLinkColor,
    SankeyNode,
    SankeyNodeColoring,
    SankeyNodeOrder,
    SankeyPlot,
    Scatter3DPlot,
    Scatter3DPoint,
    // Core
    ScatterPlot,
    SeriesPlot,
    SeriesStyle,
    SlopePlot,
    SlopePoint,
    SlopeValueFormat,
    StackedAreaPlot,
    Strand,
    StreamBaseline,
    StreamOrder,
    StreamgraphPlot,
    StripGroup,
    StripPlot,
    StripStyle,
    SunburstColorMode,
    SunburstPlot,
    Surface3DPlot,
    SurvivalPlot,
    SyntenyBlock,
    SyntenyPlot,
    SyntenySequence,
    TernaryPlot,
    TernaryPoint,
    TreeBranchStyle,
    TreeOrientation,
    TreemapColorMode,
    TreemapLayout,
    TreemapNode,
    TreemapPlot,
    UpSetIntersection,
    UpSetPlot,
    UpSetSort,
    VennOverlap,
    VennPlot,
    VennSet,
    View3D,
    ViolinGroup,
    ViolinPlot,
    VolcanoPlot,
    VolcanoPoint,
    WaterfallBar,
    WaterfallKind,
    WaterfallPlot,
    ZReduce,
};

// ── Plot enum ────────────────────────────────────────────────────────────────
pub use crate::render::plots::Plot;

// ── Layout & rendering ───────────────────────────────────────────────────────
pub use crate::render::layout::{AxisLine, Layout, TickAlign, TickFormat, TickPos};
pub use crate::render::render::{
    collect_legend_entries, render_bump, render_forest, render_funnel, render_gantt,
    render_jointplot, render_lollipop, render_mosaic, render_multiple, render_parallel,
    render_phylo_tree, render_pr, render_pyramid, render_roc, render_rose, render_sankey,
    render_slope, render_sunburst, render_survival, render_synteny, render_treemap, render_twin_y,
    render_venn,
};

// ── Figure (multi-plot grid) ─────────────────────────────────────────────────
pub use crate::render::figure::{
    Figure, FigureLegendPosition, LabelConfig, LabelStyle as PanelLabelStyle, SharedAxis,
};

// ── Theme & palette ──────────────────────────────────────────────────────────
pub use crate::render::palette::Palette;
pub use crate::render::theme::Theme;

// ── Annotations ──────────────────────────────────────────────────────────────
pub use crate::render::annotations::{ReferenceLine, ShadedRegion, TextAnnotation};

// ── Date / time axes ─────────────────────────────────────────────────────────
pub use crate::render::datetime::{ymd, ymd_hms, DateTimeAxis, DateUnit};

// ── One-shot render helpers ───────────────────────────────────────────────────
pub use crate::render_to_svg;

#[cfg(feature = "png")]
pub use crate::render_to_png;

#[cfg(feature = "pdf")]
pub use crate::render_to_pdf;

// ── Backends ─────────────────────────────────────────────────────────────────
pub use crate::backend::svg::SvgBackend;
pub use crate::backend::terminal::TerminalBackend;

#[cfg(feature = "png")]
pub use crate::backend::png::PngBackend;

#[cfg(feature = "pdf")]
pub use crate::backend::pdf::PdfBackend;
