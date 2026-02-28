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
    // Core
    ScatterPlot,
    LinePlot,
    BarPlot,
    Histogram,
    Histogram2D,
    BoxPlot,
    BoxGroup,
    ViolinPlot,
    ViolinGroup,
    PiePlot,
    PieSlice,
    SeriesPlot,
    Heatmap,
    BrickPlot,
    BandPlot,
    WaterfallPlot,
    WaterfallBar,
    WaterfallKind,
    StripPlot,
    StripGroup,
    StripStyle,
    VolcanoPlot,
    VolcanoPoint,
    ManhattanPlot,
    ManhattanPoint,
    ChromSpan,
    GenomeBuild,
    DotPlot,
    DotPoint,
    UpSetPlot,
    UpSetIntersection,
    UpSetSort,
    StackedAreaPlot,
    CandlestickPlot,
    CandleDataPoint,
    ContourPlot,
    ChordPlot,
    SankeyPlot,
    SankeyNode,
    SankeyLink,
    SankeyLinkColor,
    PhyloTree,
    PhyloNode,
    TreeOrientation,
    TreeBranchStyle,
    SyntenyPlot,
    SyntenyBlock,
    SyntenySequence,
    Strand,
    // Style / config types used when building plots
    MarkerShape,
    LineStyle,
    SeriesStyle,
    PieLabelPosition,
    ColorMap,
    LabelStyle as VolcanoLabelStyle,
    // Legend
    LegendPosition,
    LegendEntry,
    LegendShape,
};

// ── Plot enum ────────────────────────────────────────────────────────────────
pub use crate::render::plots::Plot;

// ── Layout & rendering ───────────────────────────────────────────────────────
pub use crate::render::layout::{Layout, TickFormat};
pub use crate::render::render::{
    render_multiple,
    render_twin_y,
    render_sankey,
    render_phylo_tree,
    render_synteny,
};

// ── Figure (multi-plot grid) ─────────────────────────────────────────────────
pub use crate::render::figure::{
    Figure,
    FigureLegendPosition,
    LabelConfig,
    LabelStyle as PanelLabelStyle,
    SharedAxis,
};

// ── Theme & palette ──────────────────────────────────────────────────────────
pub use crate::render::theme::Theme;
pub use crate::render::palette::Palette;

// ── Annotations ──────────────────────────────────────────────────────────────
pub use crate::render::annotations::{TextAnnotation, ReferenceLine, ShadedRegion};

// ── Date / time axes ─────────────────────────────────────────────────────────
pub use crate::render::datetime::{DateTimeAxis, DateUnit, ymd, ymd_hms};

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
