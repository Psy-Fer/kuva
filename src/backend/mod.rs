pub mod svg;

use crate::plot::scatter::ScatterPlot;

pub trait Backend {
    fn render_scatter(&self, plot: &ScatterPlot) -> String;
}