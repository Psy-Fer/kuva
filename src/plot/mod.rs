pub mod scatter;
pub mod line;
pub mod bar;
pub mod histogram;
pub mod boxplot;
pub mod types;

pub use scatter::ScatterPlot;
pub use line::LinePlot;
pub use bar::BarPlot;
pub use histogram::Histogram;
pub use boxplot::{BoxPlot, BoxGroup};
