pub mod scatter;
pub mod line;
pub mod bar;
pub mod histogram;
pub mod boxplot;
pub mod violin;

pub mod legend;

// pub mod types;

pub use scatter::ScatterPlot;
pub use line::LinePlot;
pub use bar::BarPlot;
pub use histogram::Histogram;
pub use boxplot::{BoxPlot, BoxGroup};
pub use violin::{ViolinPlot, ViolinGroup};

pub use legend::Legend;