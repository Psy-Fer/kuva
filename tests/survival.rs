mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::SurvivalPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use std::fs;

fn write_svg(name: &str, plots: Vec<Plot>, layout: Layout) -> String {
    fs::create_dir_all("test_outputs").unwrap();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output(format!("test_outputs/{name}.svg"), &svg).unwrap();
    assert!(svg.contains("<svg"));
    svg
}

fn sample_times() -> (Vec<f64>, Vec<bool>) {
    let times = vec![
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
        17.0, 18.0, 19.0, 20.0,
    ];
    let events = vec![
        true, true, false, true, true, false, true, false, true, false, true, false, true, false,
        false, true, false, true, false, true,
    ];
    (times, events)
}

#[test]
fn test_survival_basic() {
    let (times, events) = sample_times();
    let sp = SurvivalPlot::new().with_group("All patients", times, events);
    let plots = vec![Plot::Survival(sp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Kaplan-Meier")
        .with_x_label("Time (months)")
        .with_y_label("Survival probability");
    let svg = write_svg("survival_basic", plots, layout);
    // KM step function is a path
    assert!(svg.contains("<path"));
}

#[test]
fn test_survival_two_groups() {
    let sp = SurvivalPlot::new()
        .with_group(
            "Control",
            vec![2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0],
            vec![
                true, true, true, false, true, false, true, false, false, true,
            ],
        )
        .with_group(
            "Treatment",
            vec![3.0, 7.0, 9.0, 12.0, 15.0, 18.0, 20.0, 22.0, 24.0, 26.0],
            vec![
                true, false, true, false, true, false, false, true, false, false,
            ],
        )
        .with_legend("Group");
    let plots = vec![Plot::Survival(sp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Two-group KM")
        .with_x_label("Time (months)")
        .with_y_label("Survival probability");
    let svg = write_svg("survival_two_groups", plots, layout);
    assert!(svg.contains("Control"));
    assert!(svg.contains("Treatment"));
}

#[test]
fn test_survival_with_ci() {
    let (times, events) = sample_times();
    let sp = SurvivalPlot::new()
        .with_group("Patients", times, events)
        .with_ci(true)
        .with_ci_alpha(0.2);
    let plots = vec![Plot::Survival(sp)];
    let layout = Layout::auto_from_plots(&plots).with_title("KM with CI");
    write_svg("survival_ci", plots, layout);
}

#[test]
fn test_survival_no_censoring_marks() {
    let (times, events) = sample_times();
    let sp = SurvivalPlot::new()
        .with_group("Patients", times, events)
        .with_censoring(false);
    let plots = vec![Plot::Survival(sp)];
    let layout = Layout::auto_from_plots(&plots).with_title("No censoring marks");
    write_svg("survival_no_censoring", plots, layout);
}

#[test]
fn test_survival_pvalue_annotation() {
    let sp = SurvivalPlot::new()
        .with_group(
            "Control",
            vec![2.0, 4.0, 6.0, 9.0, 12.0],
            vec![true, true, false, true, false],
        )
        .with_group(
            "Treatment",
            vec![4.0, 8.0, 12.0, 16.0, 20.0],
            vec![true, false, true, false, false],
        )
        .with_pvalue_text("log-rank p = 0.023")
        .with_legend("Group");
    let plots = vec![Plot::Survival(sp)];
    let layout = Layout::auto_from_plots(&plots).with_title("KM with p-value");
    let svg = write_svg("survival_pvalue", plots, layout);
    assert!(svg.contains("p = 0.023"));
}

#[test]
fn test_survival_three_groups() {
    let sp = SurvivalPlot::new()
        .with_group(
            "Stage I",
            vec![8.0, 12.0, 18.0, 24.0, 30.0],
            vec![false, true, false, false, true],
        )
        .with_group(
            "Stage II",
            vec![5.0, 8.0, 12.0, 15.0, 20.0],
            vec![true, true, false, true, false],
        )
        .with_group(
            "Stage III",
            vec![2.0, 4.0, 6.0, 9.0, 12.0],
            vec![true, true, true, false, true],
        )
        .with_legend("Stage");
    let plots = vec![Plot::Survival(sp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Survival by stage")
        .with_x_label("Time (months)")
        .with_y_label("Overall survival");
    let svg = write_svg("survival_three_groups", plots, layout);
    assert!(svg.contains("Stage I"));
    assert!(svg.contains("Stage III"));
}

#[test]
fn test_survival_all_events() {
    // No censored observations
    let sp = SurvivalPlot::new().with_group(
        "All events",
        vec![1.0, 2.0, 3.0, 4.0, 5.0],
        vec![true, true, true, true, true],
    );
    let plots = vec![Plot::Survival(sp)];
    let layout = Layout::auto_from_plots(&plots).with_title("All events");
    write_svg("survival_all_events", plots, layout);
}

#[test]
fn test_survival_all_censored() {
    // All censored — flat curve at 1.0
    let sp = SurvivalPlot::new().with_group(
        "All censored",
        vec![5.0, 10.0, 15.0],
        vec![false, false, false],
    );
    let plots = vec![Plot::Survival(sp)];
    let layout = Layout::auto_from_plots(&plots).with_title("All censored");
    write_svg("survival_all_censored", plots, layout);
}

#[test]
fn test_survival_single_event() {
    let sp = SurvivalPlot::new().with_group(
        "One event",
        vec![5.0, 10.0, 15.0],
        vec![false, true, false],
    );
    assert!(Plot::Survival(sp).bounds().is_some());
}

#[test]
fn test_survival_empty_no_panic() {
    let sp = SurvivalPlot::new();
    assert!(Plot::Survival(sp).bounds().is_none());
}

#[test]
fn test_survival_colored_groups() {
    let sp = SurvivalPlot::new()
        .with_colored_group(
            "Control",
            vec![2.0, 5.0, 8.0, 12.0, 16.0],
            vec![true, false, true, false, true],
            "tomato",
        )
        .with_colored_group(
            "Treatment",
            vec![4.0, 8.0, 12.0, 18.0, 24.0],
            vec![true, false, false, true, false],
            "seagreen",
        )
        .with_ci(true)
        .with_legend("Group");
    let plots = vec![Plot::Survival(sp)];
    let layout = Layout::auto_from_plots(&plots).with_title("Colored groups + CI");
    write_svg("survival_colored_ci", plots, layout);
}

#[test]
fn test_survival_realistic_os() {
    // 40-patient simulated OS dataset, two arms
    let ctrl_times = vec![
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 15.0, 16.0, 18.0,
        20.0, 22.0, 24.0, 26.0,
    ];
    let ctrl_events = vec![
        true, true, true, false, true, true, false, true, false, true, true, false, true, false,
        false, true, false, true, false, false,
    ];
    let trt_times = vec![
        2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0, 26.0, 28.0, 30.0, 32.0,
        34.0, 36.0, 40.0, 42.0,
    ];
    let trt_events = vec![
        false, true, false, true, false, false, true, false, false, true, false, false, true,
        false, false, true, false, false, false, true,
    ];
    let sp = SurvivalPlot::new()
        .with_group("Control (n=20)", ctrl_times, ctrl_events)
        .with_group("Treatment (n=20)", trt_times, trt_events)
        .with_ci(true)
        .with_pvalue_text("log-rank p = 0.041")
        .with_legend("Arm");
    let plots = vec![Plot::Survival(sp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Overall Survival")
        .with_x_label("Time (months)")
        .with_y_label("Probability of survival")
        .with_width(520.0)
        .with_height(400.0);
    let svg = write_svg("survival_realistic_os", plots, layout);
    assert!(svg.contains("Control"));
    assert!(svg.contains("Treatment"));
    assert!(svg.contains("p = 0.041"));
}

// ── Clinical additions: risk table, median lines, log-rank p-value ──────────────

fn two_arms() -> SurvivalPlot {
    // Arm A tends to have earlier events than arm B (separated survival).
    let a_t = vec![2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0];
    let a_e = vec![true; 10];
    let b_t = vec![6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0];
    let b_e = vec![
        true, true, false, true, true, false, true, false, true, false,
    ];
    SurvivalPlot::new()
        .with_group("Arm A", a_t, a_e)
        .with_group("Arm B", b_t, b_e)
        .with_legend("Treatment")
}

#[test]
fn test_survival_risk_table() {
    let plot = two_arms().with_risk_table(true);
    let plots = vec![Plot::Survival(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Survival with risk table")
        .with_x_label("Time (months)")
        .with_y_label("Survival probability");
    let svg = write_svg("survival_risk_table", plots, layout);
    assert!(svg.contains("Number at risk"), "risk table title missing");
    // Both group labels appear in the table.
    assert!(svg.matches(">Arm A<").count() >= 1 && svg.matches(">Arm B<").count() >= 1);

    // Regression guard: the table sits below the plot area, so it must be emitted AFTER
    // the plot-area clip group closes — otherwise it renders but is clipped invisible.
    let title_pos = svg.find("Number at risk").unwrap();
    let before = &svg[..title_pos];
    let last_clip_open = before.rfind("clip-path=");
    let last_clip_close = before.rfind("</g>");
    assert!(
        last_clip_open.is_none() || last_clip_close > last_clip_open,
        "risk table must be drawn outside the plot-area clip (else it is clipped away)"
    );
}

#[test]
fn test_survival_median_lines() {
    let plot = two_arms().with_median_lines(true);
    let plots = vec![Plot::Survival(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Median survival lines");
    let svg = write_svg("survival_median_lines", plots, layout);
    // Median reference lines are dashed.
    assert!(
        svg.contains("stroke-dasharray=\"4,3\""),
        "median lines should be dashed"
    );
}

#[test]
fn test_survival_logrank_pvalue() {
    let plot = two_arms().with_logrank_pvalue(true);
    let plots = vec![Plot::Survival(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Log-rank p-value");
    let svg = write_svg("survival_logrank", plots, layout);
    assert!(
        svg.contains("log-rank p"),
        "computed log-rank annotation missing"
    );
}

#[test]
fn test_survival_all_clinical() {
    let plot = two_arms()
        .with_risk_table(true)
        .with_median_lines(true)
        .with_logrank_pvalue(true);
    let plots = vec![Plot::Survival(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Overall Survival")
        .with_x_label("Time (months)")
        .with_y_label("Survival probability");
    let svg = write_svg("survival_clinical", plots, layout);
    assert!(svg.contains("Number at risk"));
    assert!(svg.contains("log-rank p"));
    assert!(svg.contains("stroke-dasharray=\"4,3\""));
}
