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

pub mod legend;

// pub mod types;

pub use scatter::ScatterPlot;
pub use line::LinePlot;
pub use bar::BarPlot;
pub use histogram::Histogram;
pub use histogram2d::Histogram2D;
pub use boxplot::{BoxPlot, BoxGroup};
pub use violin::{ViolinPlot, ViolinGroup};
pub use pie::{PiePlot, PieSlice};
pub use series::{SeriesPlot, SeriesStyle};
pub use heatmap::{Heatmap, ColorMap};

pub use legend::Legend;