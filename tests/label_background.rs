//! Label backgrounds (issue #102): a semi-opaque rect drawn behind in-fill
//! value labels (Treemap, Sunburst, Mosaic, Funnel, Gantt) for readability
//! over busy fills or BW-mode hatch patterns. Off by default in color mode,
//! on by default in BW mode, overridable either way via
//! `Layout::with_label_background`.

mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::funnel::FunnelPlot;
use kuva::plot::mosaic::MosaicPlot;
use kuva::plot::sunburst::SunburstPlot;
use kuva::plot::treemap::{TreemapNode, TreemapPlot};
use kuva::plot::GanttPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

/// The marker left by `label_bg_rect` on every background it draws
/// (`Rect`/`Path` both serialize their `opacity: Some(0.82)` as this
/// `fill-opacity` attribute) — distinct from any other opacity used in these
/// renderers' bar/wedge/segment fills. Deliberately no closing quote: f64
/// roundtripping can print `0.82` as `0.8200000000000001`, so this only
/// anchors the prefix both forms share.
const BG_MARKER: &str = r#"fill-opacity="0.82"#;

fn svg_for(plots: Vec<Plot>, layout: Layout) -> String {
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

/// True if any `<text>` element is filled with the default light theme's
/// `text_color` ("black", serialized as `#000000`) — i.e. an in-fill label
/// switched off its usual white (chosen for contrast against the plot's own
/// fill) once a background rect made white-on-white unreadable.
fn svg_has_black_text(svg: &str) -> bool {
    let mut search_from = 0;
    while let Some(rel) = svg[search_from..].find("<text ") {
        let start = search_from + rel;
        let end = svg[start..].find('>').map_or(svg.len(), |e| start + e);
        if svg[start..end].contains(r##"fill="#000000""##) {
            return true;
        }
        search_from = end.max(start + 1);
    }
    false
}

fn treemap() -> TreemapPlot {
    TreemapPlot::new()
        .with_node(TreemapNode::leaf("Alpha", 40.0))
        .with_node(TreemapNode::leaf("Beta", 30.0))
        .with_node(TreemapNode::leaf("Gamma", 20.0))
}

fn sunburst() -> SunburstPlot {
    SunburstPlot::new().with_node(TreemapNode::new(
        "Root",
        vec![
            TreemapNode::leaf("A", 30.0),
            TreemapNode::leaf("B", 45.0),
            TreemapNode::leaf("C", 25.0),
        ],
    ))
}

fn mosaic() -> MosaicPlot {
    MosaicPlot::new()
        .with_cell("Unexposed", "Healthy", 500.0)
        .with_cell("Unexposed", "Severe", 20.0)
        .with_cell("Dosed", "Healthy", 100.0)
        .with_cell("Dosed", "Severe", 200.0)
}

fn funnel() -> FunnelPlot {
    FunnelPlot::new()
        .with_stage("Screened", 1200)
        .with_stage("Eligible", 800)
        .with_stage("Enrolled", 600)
}

fn gantt() -> GanttPlot {
    GanttPlot::new()
        .with_task("Task A", 0.0, 3.0)
        .with_task("Task B", 2.0, 6.0)
}

macro_rules! bg_tests {
    ($name:ident, $plot_variant:ident, $ctor:expr) => {
        mod $name {
            use super::*;

            #[test]
            fn off_by_default_in_color_mode() {
                let plots = vec![Plot::$plot_variant($ctor)];
                let layout = Layout::auto_from_plots(&plots).with_title("color");
                let svg = svg_for(plots, layout);
                assert!(
                    !svg.contains(BG_MARKER),
                    "expected no label background by default in color mode"
                );
            }

            #[test]
            fn on_by_default_in_bw_mode() {
                let plots = vec![Plot::$plot_variant($ctor)];
                let layout = Layout::auto_from_plots(&plots)
                    .with_title("bw")
                    .with_bw_mode();
                let svg = svg_for(plots, layout);
                common::write_test_output(
                    concat!("test_outputs/label_bg_", stringify!($name), ".svg"),
                    &svg,
                )
                .unwrap();
                assert!(
                    svg.contains(BG_MARKER),
                    "expected a label background by default in BW mode"
                );
                assert!(
                    svg_has_black_text(&svg),
                    "expected in-fill label text to switch to the theme's text_color \
                     (black, in the default light theme) once a background rect is drawn, \
                     instead of staying white-on-white"
                );
            }

            #[test]
            fn explicit_on_in_color_mode() {
                let plots = vec![Plot::$plot_variant($ctor)];
                let layout = Layout::auto_from_plots(&plots)
                    .with_title("forced-on")
                    .with_label_background(true);
                let svg = svg_for(plots, layout);
                assert!(
                    svg.contains(BG_MARKER),
                    "expected a label background when explicitly forced on"
                );
                assert!(
                    svg_has_black_text(&svg),
                    "expected in-fill label text to switch to black once forced on in color mode"
                );
            }

            #[test]
            fn explicit_off_in_bw_mode() {
                let plots = vec![Plot::$plot_variant($ctor)];
                let layout = Layout::auto_from_plots(&plots)
                    .with_title("forced-off")
                    .with_bw_mode()
                    .with_label_background(false);
                let svg = svg_for(plots, layout);
                assert!(
                    !svg.contains(BG_MARKER),
                    "expected no label background when explicitly forced off, even in BW mode"
                );
            }
        }
    };
}

bg_tests!(treemap_labels, Treemap, treemap());
bg_tests!(sunburst_labels, Sunburst, sunburst());
bg_tests!(mosaic_labels, Mosaic, mosaic());
bg_tests!(funnel_labels, Funnel, funnel());
bg_tests!(gantt_labels, Gantt, gantt());

/// `clone_layout` (src/render/figure.rs) must copy every `Layout` field,
/// `label_background` included — a `Figure` cell should honor the same
/// override a single-plot `Layout` would.
#[test]
fn figure_cell_inherits_label_background() {
    use kuva::render::figure::Figure;

    let plots_a = vec![Plot::Treemap(treemap())];
    let layout_a = Layout::auto_from_plots(&plots_a).with_label_background(true);
    let plots_b = vec![Plot::Gantt(gantt())];
    let layout_b = Layout::auto_from_plots(&plots_b);

    let figure = Figure::new(1, 2)
        .with_plots(vec![plots_a, plots_b])
        .with_layouts(vec![layout_a, layout_b]);
    let scene = figure.render();
    let svg = SvgBackend.render_scene(&scene);
    assert!(
        svg.contains(BG_MARKER),
        "expected the treemap cell's explicit with_label_background(true) to survive Figure layout cloning"
    );
}
