use kuva::backend::svg::SvgBackend;
use kuva::backend::terminal::TerminalBackend;
use kuva::plot::{
    Clustermap, ColorMap, FunnelPlot, GanttPlot, Heatmap, NetworkPlot, PieLabelPosition, PiePlot,
    PopulationPyramid, RosePlot, SankeyPlot, StreamgraphPlot, SunburstColorMode, SunburstPlot,
    TreemapColorMode, TreemapNode, TreemapPlot,
};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use kuva::render::theme::Theme;

/// Extract all `fill="..."` attributes of `<text>` elements whose content matches `needle`.
fn text_fills_for(svg: &str, needle: &str) -> Vec<Option<String>> {
    let target = format!(">{needle}</text>");
    let mut out = Vec::new();
    for chunk in svg.split("<text").skip(1) {
        if let Some(end) = chunk.find("</text>") {
            let full = &chunk[..end + "</text>".len()];
            if full.ends_with(&target) {
                let tag = full.split('>').next().unwrap_or("");
                let fill = tag
                    .split("fill=\"")
                    .nth(1)
                    .and_then(|rest| rest.split('"').next())
                    .map(|s| s.to_string());
                out.push(fill);
            }
        }
    }
    out
}

fn text_fill_for(svg: &str, needle: &str) -> Option<String> {
    text_fills_for(svg, needle).into_iter().flatten().next()
}

#[test]
fn clustermap_and_heatmap_auto_contrast_viridis_light_and_dark() {
    // Clustermap with Viridis: 0.0 is dark purple (#440154 -> #ffffff),
    // 10.0 is bright yellow (#fde725 -> #000000).
    let cm = Clustermap::new()
        .with_data(vec![vec![0.0, 10.0], vec![10.0, 0.0]])
        .with_row_labels(["R1", "R2"])
        .with_col_labels(["C1", "C2"])
        .with_color_map(ColorMap::Viridis)
        .with_values();
    let plots = vec![Plot::Clustermap(cm)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert_eq!(
        text_fill_for(&svg, "0.00").as_deref(),
        Some("#ffffff"),
        "dark Viridis clustermap cell should get white text"
    );
    assert_eq!(
        text_fill_for(&svg, "10.00").as_deref(),
        Some("#000000"),
        "bright Viridis clustermap cell should get black text"
    );

    // Heatmap on Theme::dark(): bright yellow cell must still get black text
    // instead of light theme text_color (#e0e0e0).
    let hm = Heatmap::new()
        .with_data(vec![vec![0.0, 10.0], vec![5.0, 10.0]])
        .with_color_map(ColorMap::Viridis)
        .with_values();
    let plots = vec![Plot::Heatmap(hm)];
    let layout = Layout::auto_from_plots(&plots).with_theme(Theme::dark());
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert_eq!(
        text_fill_for(&svg, "0.00").as_deref(),
        Some("#ffffff"),
        "dark Viridis heatmap cell should get white text"
    );
    assert_eq!(
        text_fill_for(&svg, "10.00").as_deref(),
        Some("#000000"),
        "bright Viridis heatmap cell on Theme::dark() should get black text"
    );
}

#[test]
fn treemap_parent_and_leaf_tiles_auto_contrast() {
    let tm = TreemapPlot::new()
        .with_children(
            "Root",
            vec![
                TreemapNode::leaf("DarkLeaf", 25.0),
                TreemapNode::leaf("BrightLeaf", 100.0),
            ],
        )
        .with_color_mode(TreemapColorMode::ByValue(ColorMap::Viridis));
    let plots = vec![Plot::Treemap(tm)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    // Parent tile in ByValue mode is #e0e0e0 (light grey) -> must get black text.
    assert_eq!(
        text_fill_for(&svg, "Root").as_deref(),
        Some("#000000"),
        "light grey #e0e0e0 Treemap parent tile must get black text"
    );
    assert_eq!(
        text_fill_for(&svg, "BrightLeaf").as_deref(),
        Some("#000000"),
        "bright yellow Viridis leaf must get black text"
    );
    assert_eq!(
        text_fill_for(&svg, "DarkLeaf").as_deref(),
        Some("#ffffff"),
        "dark purple Viridis leaf must get white text"
    );
}

#[test]
fn pie_funnel_gantt_and_streamgraph_auto_contrast() {
    // Pie: dark navy slice -> white, bright yellow slice -> black.
    let pie = PiePlot::new()
        .with_slice("DarkSlice", 50.0, "#000080")
        .with_slice("LightSlice", 50.0, "#fde725")
        .with_label_position(PieLabelPosition::Inside);
    let plots = vec![Plot::Pie(pie)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert_eq!(text_fill_for(&svg, "DarkSlice").as_deref(), Some("#ffffff"));
    assert_eq!(
        text_fill_for(&svg, "LightSlice").as_deref(),
        Some("#000000")
    );

    // Funnel: dark vs light stage colors.
    let funnel = FunnelPlot::new()
        .with_stage_color("DarkStage", 100.0, "#000080")
        .with_stage_color("LightStage", 80.0, "#fde725")
        .with_show_values(true)
        .with_show_percents(false);
    let plots = vec![Plot::Funnel(funnel)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert_eq!(text_fill_for(&svg, "100").as_deref(), Some("#ffffff"));
    assert_eq!(text_fill_for(&svg, "80").as_deref(), Some("#000000"));

    // Gantt: dark vs light task bars (0.85 opacity composited over white).
    let gantt = GanttPlot::new()
        .with_colored_task("DarkTask", 0.0, 10.0, "#000080")
        .with_colored_task("LightTask", 0.0, 10.0, "#fde725");
    let plots = vec![Plot::Gantt(gantt)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert_eq!(text_fill_for(&svg, "DarkTask").as_deref(), Some("#ffffff"));
    assert_eq!(text_fill_for(&svg, "LightTask").as_deref(), Some("#000000"));

    // Streamgraph: named "yellow" (previously defaulted to white!) vs "navy".
    let sg = StreamgraphPlot::new()
        .with_x([0.0, 1.0, 2.0, 3.0])
        .with_series([20.0, 30.0, 30.0, 20.0])
        .with_color("yellow")
        .with_label("YellowStream")
        .with_series([20.0, 30.0, 30.0, 20.0])
        .with_color("navy")
        .with_label("NavyStream")
        .with_stream_labels(true);
    let plots = vec![Plot::Streamgraph(sg)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert_eq!(
        text_fill_for(&svg, "YellowStream").as_deref(),
        Some("#000000")
    );
    assert_eq!(
        text_fill_for(&svg, "NavyStream").as_deref(),
        Some("#ffffff")
    );
}

#[test]
fn bw_mode_heatmap_and_clustermap_keep_wcag_contrast() {
    // Regression: in bw_mode, label_background defaults to true, but Heatmap
    // and Clustermap draw solid greyscale cells without label_bg_rect — dark
    // cells (#000000) must still get white text, not black-on-black.
    let hm = Heatmap::new()
        .with_data(vec![vec![0.0, 10.0], vec![10.0, 0.0]])
        .with_values();
    let plots = vec![Plot::Heatmap(hm)];
    let layout = Layout::auto_from_plots(&plots).with_bw_mode();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert_eq!(
        text_fill_for(&svg, "0.00").as_deref(),
        Some("#000000"),
        "light grey bw_mode heatmap cell should get black text"
    );
    assert_eq!(
        text_fill_for(&svg, "10.00").as_deref(),
        Some("#ffffff"),
        "black bw_mode heatmap cell must get white text (not black-on-black)"
    );
}

#[test]
fn rose_sankey_sunburst_network_pyramid_and_funnel_dark_theme() {
    // RosePlot stacked mode: sector N ends in dark navy (40 -> white),
    // sector S ends in bright yellow (60 -> black).
    let mut rose = RosePlot::new()
        .with_x_labels(["N", "E", "S", "W"])
        .with_stack("Dark", [40.0, 40.0, 0.0, 0.0])
        .with_stack("Light", [0.0, 0.0, 60.0, 60.0])
        .with_grid(false)
        .with_show_values(true);
    rose.series[0].color = Some("#000080".to_string());
    rose.series[1].color = Some("#fde725".to_string());
    let plots = vec![Plot::Rose(rose)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert_eq!(text_fill_for(&svg, "40").as_deref(), Some("#ffffff"));
    assert_eq!(text_fill_for(&svg, "60").as_deref(), Some("#000000"));

    // SankeyPlot with flow labels on dark vs light opaque ribbons.
    let sankey = SankeyPlot::new()
        .with_node_color("DarkSrc", "#000080")
        .with_node_color("LightSrc", "#fde725")
        .with_link("DarkSrc", "Target", 60.0)
        .with_link("LightSrc", "Target", 40.0)
        .with_link_opacity(0.95)
        .with_flow_labels();
    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert_eq!(text_fill_for(&svg, "60").as_deref(), Some("#ffffff"));
    assert_eq!(text_fill_for(&svg, "40").as_deref(), Some("#000000"));

    // SunburstPlot: ByValue Viridis with dark vs bright leaf wedges.
    let sb = SunburstPlot::new()
        .with_children(
            "All",
            vec![
                TreemapNode::leaf("DarkArc", 40.0),
                TreemapNode::leaf("BrightArc", 100.0),
            ],
        )
        .with_color_mode(SunburstColorMode::ByValue(ColorMap::Viridis));
    let plots = vec![Plot::Sunburst(sb)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert_eq!(text_fill_for(&svg, "DarkArc").as_deref(), Some("#ffffff"));
    assert_eq!(text_fill_for(&svg, "BrightArc").as_deref(), Some("#000000"));

    // NetworkPlot with labels inside dark vs light nodes.
    let net = NetworkPlot::new()
        .with_node_color("DarkNode", "#000080")
        .with_node_color("LightNode", "#fde725")
        .with_edge("DarkNode", "LightNode", 1.0)
        .with_labels_inside();
    let plots = vec![Plot::Network(net)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert_eq!(text_fill_for(&svg, "DarkNode").as_deref(), Some("#ffffff"));
    assert_eq!(text_fill_for(&svg, "LightNode").as_deref(), Some("#000000"));

    // PopulationPyramid: dark left bars vs bright yellow right bars.
    let pyr = PopulationPyramid::new()
        .with_group("0-9", 80.0, 90.0)
        .with_left_color("#000080")
        .with_right_color("#fde725")
        .with_show_values(true);
    let plots = vec![Plot::Pyramid(pyr)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert_eq!(text_fill_for(&svg, "80").as_deref(), Some("#ffffff"));
    assert_eq!(text_fill_for(&svg, "90").as_deref(), Some("#000000"));

    // FunnelPlot on Theme::dark(): conversion-rate label over semi-transparent
    // dark connector must get white text (not hardcoded #555555).
    let funnel_dark = FunnelPlot::new()
        .with_stage_color("Top", 100.0, "#000080")
        .with_stage_color("Mid", 50.0, "#000080")
        .with_stage_gap(18.0)
        .with_show_conversion(true);
    let plots = vec![Plot::Funnel(funnel_dark)];
    let layout = Layout::auto_from_plots(&plots).with_theme(Theme::dark());
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert_eq!(
        text_fill_for(&svg, "50.0%").as_deref(),
        Some("#ffffff"),
        "dark-theme funnel conversion label should get white text"
    );

    // TerminalBackend smoke test: auto-contrast colors do not crash or leak into terminal output.
    let hm = Heatmap::new()
        .with_data(vec![vec![0.0, 10.0], vec![10.0, 0.0]])
        .with_values();
    let plots = vec![Plot::Heatmap(hm)];
    let layout = Layout::auto_from_plots(&plots);
    let term = TerminalBackend::new(40, 16).render_scene(&render_multiple(plots, layout));
    assert!(!term.is_empty());
}
