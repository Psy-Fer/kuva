pub mod scatter;
pub mod line;
pub mod bar;
pub mod histogram;
pub mod histogram2d;
pub mod boxplot;
pub mod violin;
pub mod pie;
pub mod series;
pub mod heatmap;
pub mod brick;
pub mod band;
pub mod waterfall;
pub mod strip;
pub mod volcano;
pub mod manhattan;
pub mod dotplot;
pub mod upset;
pub mod stacked_area;

pub mod legend;

// pub mod types;

pub use scatter::{ScatterPlot, MarkerShape};
pub use line::{LinePlot, LineStyle};
pub use bar::BarPlot;
pub use histogram::Histogram;
pub use histogram2d::Histogram2D;
pub use boxplot::{BoxPlot, BoxGroup};
pub use violin::{ViolinPlot, ViolinGroup};
pub use pie::{PiePlot, PieSlice, PieLabelPosition};
pub use series::{SeriesPlot, SeriesStyle};
pub use heatmap::{Heatmap, ColorMap};
pub use brick::BrickPlot;
pub use band::BandPlot;
pub use waterfall::{WaterfallPlot, WaterfallBar, WaterfallKind};
pub use strip::{StripPlot, StripGroup, StripStyle};
pub use volcano::{VolcanoPlot, VolcanoPoint, LabelStyle};
pub use manhattan::{ManhattanPlot, ManhattanPoint, ChromSpan, GenomeBuild};
pub use dotplot::{DotPlot, DotPoint};
pub use upset::{UpSetPlot, UpSetIntersection, UpSetSort};
pub use stacked_area::StackedAreaPlot;

pub use legend::{Legend, LegendEntry, LegendShape, LegendPosition};