mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::venn::VennPlot;
use kuva::render::{layout::Layout, plots::Plot, render::render_multiple};
use std::fs;

fn write_venn(name: &str, venn: VennPlot, title: &str) -> String {
    let plots = vec![Plot::Venn(venn)];
    let layout = Layout::auto_from_plots(&plots).with_title(title);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    fs::create_dir_all("test_outputs").unwrap();
    common::write_test_output(format!("test_outputs/venn_{name}.svg"), &svg).unwrap();
    svg
}

fn circle_count(svg: &str) -> usize {
    svg.matches("<circle").count()
}

#[test]
fn test_venn_2set_basic() {
    let venn = VennPlot::new()
        .with_set_size("Set A", 100)
        .with_set_size("Set B", 80)
        .with_overlap(["Set A", "Set B"], 30);
    write_venn("2set_basic", venn, "2-Set Venn Diagram");
}

#[test]
fn test_venn_2set_proportional() {
    let venn = VennPlot::new()
        .with_set_size("Large", 200)
        .with_set_size("Small", 50)
        .with_overlap(["Large", "Small"], 20)
        .with_proportional(true)
        .with_loss(true);
    write_venn("2set_proportional", venn, "2-Set Proportional Venn");
}

#[test]
fn test_venn_3set_basic() {
    let venn = VennPlot::new()
        .with_set_size("A", 120)
        .with_set_size("B", 100)
        .with_set_size("C", 90)
        .with_overlap(["A", "B"], 35)
        .with_overlap(["A", "C"], 25)
        .with_overlap(["B", "C"], 30)
        .with_overlap(["A", "B", "C"], 10);
    write_venn("3set_basic", venn, "3-Set Venn Diagram");
}

#[test]
fn test_venn_3set_with_percentages() {
    let venn = VennPlot::new()
        .with_set_size("X", 150)
        .with_set_size("Y", 120)
        .with_set_size("Z", 100)
        .with_overlap(["X", "Y"], 45)
        .with_overlap(["X", "Z"], 30)
        .with_overlap(["Y", "Z"], 40)
        .with_overlap(["X", "Y", "Z"], 15)
        .with_counts(true)
        .with_percentages(true);
    write_venn("3set_percentages", venn, "3-Set Venn with Percentages");
}

#[test]
fn test_venn_3set_proportional() {
    let venn = VennPlot::new()
        .with_set_size("A", 300)
        .with_set_size("B", 150)
        .with_set_size("C", 200)
        .with_overlap(["A", "B"], 50)
        .with_overlap(["A", "C"], 80)
        .with_overlap(["B", "C"], 40)
        .with_overlap(["A", "B", "C"], 20)
        .with_proportional(true);
    write_venn("3set_proportional", venn, "3-Set Proportional Venn");
}

#[test]
fn test_venn_4set_basic() {
    let venn = VennPlot::new()
        .with_set_size("W", 200)
        .with_set_size("X", 180)
        .with_set_size("Y", 160)
        .with_set_size("Z", 140)
        .with_overlap(["W", "X"], 60)
        .with_overlap(["W", "Y"], 50)
        .with_overlap(["W", "Z"], 40)
        .with_overlap(["X", "Y"], 55)
        .with_overlap(["X", "Z"], 45)
        .with_overlap(["Y", "Z"], 35)
        .with_overlap(["W", "X", "Y"], 20)
        .with_overlap(["W", "X", "Z"], 15)
        .with_overlap(["W", "Y", "Z"], 12)
        .with_overlap(["X", "Y", "Z"], 18)
        .with_overlap(["W", "X", "Y", "Z"], 5)
        .with_legend("Sets");
    write_venn("4set_basic", venn, "4-Set Venn Diagram");
}

#[test]
fn test_venn_raw_elements() {
    let venn = VennPlot::new()
        .with_set("A", vec!["a", "b", "c", "d", "e"])
        .with_set("B", vec!["c", "d", "e", "f", "g"])
        .with_set("C", vec!["e", "f", "g", "h", "i"]);
    write_venn("raw_elements", venn, "Venn from Raw Elements");
}

#[test]
fn test_venn_gene_lists() {
    let deseq2 = vec!["BRCA1", "TP53", "MYC", "EGFR", "VEGFA", "CDKN2A", "KRAS"];
    let edger = vec!["TP53", "MYC", "KRAS", "PIK3CA", "PTEN", "RB1"];
    let limma = vec!["BRCA1", "MYC", "EGFR", "PIK3CA", "CDKN2A", "MDM2"];

    let venn = VennPlot::new()
        .with_set("DESeq2", deseq2.iter().map(|s| s.to_string()).collect())
        .with_set("edgeR", edger.iter().map(|s| s.to_string()).collect())
        .with_set("limma", limma.iter().map(|s| s.to_string()).collect())
        .with_percentages(true);

    write_venn("gene_lists", venn, "DE Gene List Overlap");
}

#[test]
fn test_venn_precomputed_sizes() {
    let venn = VennPlot::new()
        .with_set_size("Group 1", 500)
        .with_set_size("Group 2", 400)
        .with_overlap(["Group 1", "Group 2"], 100)
        .with_counts(true)
        .with_percentages(true)
        .with_colors(["steelblue", "tomato"]);
    write_venn("precomputed_sizes", venn, "Pre-computed Size Venn");
}

#[test]
fn test_venn_legend() {
    let venn = VennPlot::new()
        .with_set_size("Control", 300)
        .with_set_size("Treatment A", 250)
        .with_set_size("Treatment B", 220)
        .with_overlap(["Control", "Treatment A"], 80)
        .with_overlap(["Control", "Treatment B"], 65)
        .with_overlap(["Treatment A", "Treatment B"], 90)
        .with_overlap(["Control", "Treatment A", "Treatment B"], 30)
        .with_legend("Experimental Groups");
    write_venn("legend", venn, "Venn Diagram with Legend");
}

#[test]
fn test_venn_2set_leader_lines() {
    let venn = VennPlot::new()
        .with_set_size("Set A", 100)
        .with_set_size("Set B", 80)
        .with_overlap(["Set A", "Set B"], 30)
        .with_leader_lines(true);
    write_venn("2set_leader_lines", venn, "2-Set Venn with Leader Lines");
}

#[test]
fn test_venn_3set_leader_lines() {
    let venn = VennPlot::new()
        .with_set_size("A", 120)
        .with_set_size("B", 100)
        .with_set_size("C", 90)
        .with_overlap(["A", "B"], 35)
        .with_overlap(["A", "C"], 25)
        .with_overlap(["B", "C"], 30)
        .with_overlap(["A", "B", "C"], 10)
        .with_leader_lines(true);
    write_venn("3set_leader_lines", venn, "3-Set Venn with Leader Lines");
}

#[test]
fn test_venn_4set_leader_lines() {
    let venn = VennPlot::new()
        .with_set_size("W", 200)
        .with_set_size("X", 180)
        .with_set_size("Y", 160)
        .with_set_size("Z", 140)
        .with_overlap(["W", "X"], 60)
        .with_overlap(["W", "Y"], 50)
        .with_overlap(["W", "Z"], 40)
        .with_overlap(["X", "Y"], 55)
        .with_overlap(["X", "Z"], 45)
        .with_overlap(["Y", "Z"], 35)
        .with_overlap(["W", "X", "Y"], 20)
        .with_overlap(["W", "X", "Z"], 15)
        .with_overlap(["W", "Y", "Z"], 12)
        .with_overlap(["X", "Y", "Z"], 18)
        .with_overlap(["W", "X", "Y", "Z"], 5)
        .with_leader_lines(true)
        .with_legend("Sets");
    write_venn("4set_leader_lines", venn, "4-Set Venn with Leader Lines");
}

// ── set_indicators tests ───────────────────────────────────────────────────────

#[test]
fn test_venn_set_indicators_on_by_default() {
    // Default: show_set_indicators=true — extra circle elements appear above each
    // region label.  A 3-set Venn has 7 regions; the dots add many <circle> elements
    // beyond the 3 shape circles.
    let venn = VennPlot::new()
        .with_set_size("A", 120)
        .with_set_size("B", 100)
        .with_set_size("C", 90)
        .with_overlap(["A", "B"], 35)
        .with_overlap(["A", "C"], 25)
        .with_overlap(["B", "C"], 30)
        .with_overlap(["A", "B", "C"], 10);
    let svg = write_venn("set_indicators_on", venn, "Set Indicators On (default)");
    // 3 shape circles + at least 7 indicator dots (one per region visible)
    assert!(
        circle_count(&svg) > 3,
        "Expected indicator dots in addition to shape circles"
    );
}

#[test]
fn test_venn_set_indicators_off() {
    // with_set_indicators(false): only the 3 shape circles should appear.
    let venn = VennPlot::new()
        .with_set_size("A", 120)
        .with_set_size("B", 100)
        .with_set_size("C", 90)
        .with_overlap(["A", "B"], 35)
        .with_overlap(["A", "C"], 25)
        .with_overlap(["B", "C"], 30)
        .with_overlap(["A", "B", "C"], 10)
        .with_set_indicators(false);
    let svg = write_venn("set_indicators_off", venn, "Set Indicators Off");
    assert_eq!(
        circle_count(&svg),
        3,
        "Only 3 shape circles expected when indicators are off"
    );
}

#[test]
fn test_venn_set_indicators_2set() {
    // 2-set: 3 regions, each gets indicator dots.
    // Region A∩B gets 2 dots; A-only and B-only each get 1.  Total = 4 dots + 2 shapes = 6.
    let venn = VennPlot::new()
        .with_set_size("X", 80)
        .with_set_size("Y", 70)
        .with_overlap(["X", "Y"], 25);
    let svg = write_venn("set_indicators_2set", venn, "2-Set with Indicators");
    // 2 shapes + 4 indicator dots (1+1+2 for the three regions)
    assert!(
        circle_count(&svg) >= 6,
        "Expected 2 shape circles + 4 indicator dots"
    );
}

#[test]
fn test_venn_set_indicators_leader_lines_off() {
    // with_leader_lines + with_set_indicators(false): leader-line labels appear
    // but without colored dots (no circles beyond shapes in the 4-set leader areas).
    let venn = VennPlot::new()
        .with_set_size("W", 200)
        .with_set_size("X", 180)
        .with_set_size("Y", 160)
        .with_set_size("Z", 140)
        .with_overlap(["W", "X"], 60)
        .with_overlap(["W", "Y"], 50)
        .with_overlap(["W", "Z"], 40)
        .with_overlap(["X", "Y"], 55)
        .with_overlap(["X", "Z"], 45)
        .with_overlap(["Y", "Z"], 35)
        .with_overlap(["W", "X", "Y"], 20)
        .with_overlap(["W", "X", "Z"], 15)
        .with_overlap(["W", "Y", "Z"], 12)
        .with_overlap(["X", "Y", "Z"], 18)
        .with_overlap(["W", "X", "Y", "Z"], 5)
        .with_leader_lines(true)
        .with_set_indicators(false);
    let svg = write_venn(
        "set_indicators_leader_off",
        venn,
        "Leader Lines, No Indicators",
    );
    // 4-set shapes are paths (not circles), so circle count should be 0.
    assert_eq!(
        circle_count(&svg),
        0,
        "No circles expected: 4-set shapes are paths and indicators are off"
    );
}

// ── show_loss / stress tests ───────────────────────────────────────────────────

#[test]
fn test_venn_loss_2set_proportional() {
    // Proportional 2-set with show_loss=true must render a "Stress:" annotation.
    let venn = VennPlot::new()
        .with_set_size("Large", 500)
        .with_set_size("Small", 100)
        .with_overlap(["Large", "Small"], 40)
        .with_proportional(true)
        .with_loss(true);
    let svg = write_venn("loss_2set", venn, "2-Set Proportional with Stress");
    assert!(svg.contains("Layout stress"), "Stress box missing from SVG");
}

#[test]
fn test_venn_loss_3set_proportional() {
    // Proportional 3-set with show_loss=true.
    let venn = VennPlot::new()
        .with_set_size("A", 300)
        .with_set_size("B", 200)
        .with_set_size("C", 150)
        .with_overlap(["A", "B"], 80)
        .with_overlap(["A", "C"], 50)
        .with_overlap(["B", "C"], 40)
        .with_overlap(["A", "B", "C"], 20)
        .with_proportional(true)
        .with_loss(true);
    let svg = write_venn("loss_3set", venn, "3-Set Proportional with Stress");
    assert!(svg.contains("Layout stress"), "Stress box missing from SVG");
}

#[test]
fn test_venn_loss_not_shown_without_flag() {
    // Without with_loss(true), stress should not appear even in proportional mode.
    let venn = VennPlot::new()
        .with_set_size("A", 200)
        .with_set_size("B", 150)
        .with_overlap(["A", "B"], 50)
        .with_proportional(true);
    let svg = write_venn("loss_not_shown", venn, "Proportional, No Stress Label");
    assert!(
        !svg.contains("Layout stress"),
        "Stress box should not appear when with_loss is false"
    );
}

#[test]
fn test_venn_loss_perfect_circles() {
    // Equal circles with no overlap: both regions are exactly half the total,
    // giving a low stress score (but still > 0 due to the forced overlap in layout).
    let venn = VennPlot::new()
        .with_set_size("A", 100)
        .with_set_size("B", 100)
        .with_overlap(["A", "B"], 0)
        .with_proportional(true)
        .with_loss(true);
    let svg = write_venn(
        "loss_no_overlap",
        venn,
        "Equal Circles, No Overlap, with Stress",
    );
    assert!(svg.contains("Layout stress"), "Stress box missing");
}
