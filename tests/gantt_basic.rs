mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::GanttPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

fn svg(plots: Vec<Plot>, layout: Layout) -> String {
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

fn gantt_svg(gp: GanttPlot) -> String {
    let plots = vec![Plot::Gantt(gp)];
    let layout = Layout::auto_from_plots(&plots);
    svg(plots, layout)
}

// ── Basic rendering ────────────────────────────────────────────────────────────

#[test]
fn test_gantt_ungrouped() {
    let gp = GanttPlot::new()
        .with_task("Task A", 0.0, 3.0)
        .with_task("Task B", 2.0, 6.0)
        .with_task("Task C", 5.0, 9.0);
    let plots = vec![Plot::Gantt(gp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Simple Gantt")
        .with_x_label("Week");
    let out = svg(plots, layout);
    assert!(out.contains("<svg"));
    assert!(out.contains("Simple Gantt"));
}

#[test]
fn test_gantt_grouped() {
    let gp = GanttPlot::new()
        .with_task_group("Phase 1", "Requirements", 0.0, 2.0)
        .with_task_group("Phase 1", "Design", 1.5, 4.0)
        .with_task_group("Phase 2", "Implementation", 3.5, 8.0)
        .with_task_group("Phase 2", "Testing", 7.0, 9.0);
    let out = gantt_svg(gp);
    assert!(out.contains("<svg"));
}

#[test]
fn test_gantt_milestone() {
    let gp = GanttPlot::new()
        .with_task_group("Design", "Wireframes", 0.0, 3.0)
        .with_milestone_group("Design", "Review", 3.0)
        .with_task_group("Dev", "Backend", 3.0, 8.0)
        .with_milestone("Launch", 10.0);
    let out = gantt_svg(gp);
    assert!(out.contains("<svg"));
}

#[test]
fn test_gantt_progress() {
    let gp = GanttPlot::new()
        .with_task_group_progress("Q1", "Feature A", 0.0, 4.0, 1.0)
        .with_task_group_progress("Q1", "Feature B", 1.0, 5.0, 0.6)
        .with_task_group("Q2", "Feature C", 4.0, 8.0)
        .with_now_line(3.5);
    let out = gantt_svg(gp);
    assert!(out.contains("<svg"));
}

#[test]
fn test_gantt_now_line() {
    let gp = GanttPlot::new()
        .with_task("Sprint 1", 0.0, 2.0)
        .with_task("Sprint 2", 2.0, 4.0)
        .with_task("Sprint 3", 4.0, 6.0)
        .with_now_line(3.0)
        .with_color("steelblue");
    let out = gantt_svg(gp);
    assert!(out.contains("<svg"));
}

#[test]
fn test_gantt_colored_task() {
    let gp = GanttPlot::new()
        .with_colored_task("Critical Task", 0.0, 5.0, "tomato")
        .with_task("Normal Task", 2.0, 6.0)
        .with_colored_task("Another Critical", 5.5, 8.0, "tomato");
    let out = gantt_svg(gp);
    assert!(out.contains("<svg"));
}

// ── group_order ────────────────────────────────────────────────────────────────

#[test]
fn test_gantt_group_order() {
    let gp = GanttPlot::new()
        .with_group_order(["Phase 2", "Phase 1"])
        .with_task_group("Phase 1", "Task A", 0.0, 3.0)
        .with_task_group("Phase 2", "Task B", 2.0, 5.0)
        .with_task_group("Phase 1", "Task C", 1.0, 4.0);
    let rows = gp.row_labels();
    // Phase 2 header should come before Phase 1 header
    let p2_pos = rows.iter().position(|r| r == "Phase 2").unwrap();
    let p1_pos = rows.iter().position(|r| r == "Phase 1").unwrap();
    assert!(p2_pos < p1_pos, "Phase 2 should appear before Phase 1");
}

#[test]
fn test_gantt_group_order_unlisted_at_end() {
    let gp = GanttPlot::new()
        .with_group_order(["Phase 1"])
        .with_task_group("Phase 2", "Task B", 2.0, 5.0)
        .with_task_group("Phase 1", "Task A", 0.0, 3.0);
    let rows = gp.row_labels();
    let p1_pos = rows.iter().position(|r| r == "Phase 1").unwrap();
    let p2_pos = rows.iter().position(|r| r == "Phase 2").unwrap();
    assert!(
        p1_pos < p2_pos,
        "Phase 1 (explicit order) before Phase 2 (insertion)"
    );
}

// ── Labels ────────────────────────────────────────────────────────────────────

#[test]
fn test_gantt_no_labels() {
    let gp = GanttPlot::new()
        .with_show_labels(false)
        .with_task("Hidden label", 0.0, 5.0);
    let out = gantt_svg(gp);
    assert!(out.contains("<svg"));
}

#[test]
fn test_gantt_milestone_label_not_clipped() {
    // Milestone at right edge — label must appear in SVG (right margin extended)
    let gp = GanttPlot::new()
        .with_task("Sprint 1", 0.0, 4.0)
        .with_milestone("Launch", 4.0);
    let plots = vec![Plot::Gantt(gp)];
    let layout = Layout::auto_from_plots(&plots);
    // gantt_right_annot_px should be non-zero when show_labels is true
    assert!(
        layout.gantt_right_annot_px > 0.0,
        "right margin should be reserved for milestone labels"
    );
    let out = svg(plots, layout);
    assert!(out.contains("Launch"));
}

#[test]
fn test_gantt_outside_label_long() {
    // Tasks that are very narrow relative to plot → outside labels
    let gp = GanttPlot::new()
        .with_task("Very Long Task Name", 0.0, 0.1) // narrow bar → outside label
        .with_task("Also Long Label Here", 5.0, 5.1)
        .with_task("Normal", 0.0, 10.0); // wide bar → inside label
    let plots = vec![Plot::Gantt(gp)];
    let layout = Layout::auto_from_plots(&plots);
    assert!(layout.gantt_right_annot_px > 0.0);
    let out = svg(plots, layout);
    assert!(out.contains("<svg"));
}

// ── x_bounds ──────────────────────────────────────────────────────────────────

#[test]
fn test_gantt_x_bounds_includes_now_line() {
    let gp = GanttPlot::new()
        .with_task("T", 2.0, 5.0)
        .with_now_line(10.0);
    let (x_min, x_max) = gp.x_bounds().unwrap();
    assert_eq!(x_min, 2.0);
    assert_eq!(x_max, 10.0);
}

#[test]
fn test_gantt_x_bounds_milestones() {
    let gp = GanttPlot::new()
        .with_task("T", 1.0, 5.0)
        .with_milestone("M", 8.0);
    let (x_min, x_max) = gp.x_bounds().unwrap();
    assert_eq!(x_min, 1.0);
    assert_eq!(x_max, 8.0);
}

#[test]
fn test_gantt_x_bounds_empty() {
    let gp = GanttPlot::new();
    assert!(gp.x_bounds().is_none());
}

// ── ordered_display_rows ──────────────────────────────────────────────────────

#[test]
fn test_gantt_row_labels_ungrouped() {
    let gp = GanttPlot::new()
        .with_task("Alpha", 0.0, 1.0)
        .with_task("Beta", 1.0, 2.0);
    let labels = gp.row_labels();
    // No groups → no group header rows; just the two task labels
    assert_eq!(labels, vec!["Alpha", "Beta"]);
}

#[test]
fn test_gantt_row_labels_grouped() {
    let gp = GanttPlot::new()
        .with_task_group("G1", "A", 0.0, 1.0)
        .with_task_group("G1", "B", 1.0, 2.0)
        .with_task_group("G2", "C", 2.0, 3.0);
    let labels = gp.row_labels();
    // Group headers interleaved: G1 header, A, B, G2 header, C
    assert_eq!(labels, vec!["G1", "A", "B", "G2", "C"]);
}

#[test]
fn test_gantt_ungrouped_tasks_after_groups() {
    let gp = GanttPlot::new()
        .with_task_group("Phase 1", "A", 0.0, 2.0)
        .with_task("Ungrouped", 3.0, 5.0);
    let labels = gp.row_labels();
    // Phase 1 header + A, then ungrouped (no header for None group)
    assert!(labels.contains(&"Phase 1".to_string()));
    assert!(labels.contains(&"A".to_string()));
    assert!(labels.contains(&"Ungrouped".to_string()));
    let g_pos = labels.iter().position(|l| l == "Phase 1").unwrap();
    let u_pos = labels.iter().position(|l| l == "Ungrouped").unwrap();
    assert!(g_pos < u_pos);
}

// ── Progress clamp ────────────────────────────────────────────────────────────

#[test]
fn test_gantt_progress_clamp() {
    let gp = GanttPlot::new()
        .with_task_progress("Over", 0.0, 5.0, 1.5) // > 1.0 → clamped to 1.0
        .with_task_progress("Under", 0.0, 5.0, -0.1); // < 0.0 → clamped to 0.0
    assert_eq!(gp.tasks[0].progress, Some(1.0));
    assert_eq!(gp.tasks[1].progress, Some(0.0));
}

// ── Showcase ──────────────────────────────────────────────────────────────────

#[test]
fn test_gantt_write_svg() {
    let gp = GanttPlot::new()
        .with_task_group("Design", "Wireframes", 0.0, 3.0)
        .with_task_group("Design", "Prototyping", 2.0, 5.0)
        .with_task_group("Dev", "Backend API", 3.0, 8.0)
        .with_task_group_progress("Dev", "Frontend", 4.0, 9.0, 0.4)
        .with_milestone_group("Dev", "Code Freeze", 9.0)
        .with_task_group("QA", "Testing", 8.5, 10.5)
        .with_milestone("Launch", 11.0)
        .with_now_line(6.0);
    let plots = vec![Plot::Gantt(gp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Project Roadmap")
        .with_x_label("Week")
        .with_width(700.0)
        .with_height(400.0);
    let out = svg(plots, layout);
    std::fs::create_dir_all("test_outputs").unwrap();
    common::write_test_output("test_outputs/gantt_roadmap.svg", &out).unwrap();
    assert!(out.contains("<svg"));
    assert!(out.contains("Launch"));
}

#[test]
fn test_gantt_clinical_timeline() {
    // Clinical trial timeline with months as x-axis
    let gp = GanttPlot::new()
        .with_task_group("Recruitment", "Screening", 0.0, 2.0)
        .with_task_group("Recruitment", "Enrollment", 1.0, 6.0)
        .with_task_group("Treatment", "Arm A", 6.0, 18.0)
        .with_task_group_progress("Treatment", "Arm B", 6.0, 18.0, 0.3)
        .with_task_group("Follow-up", "Safety", 18.0, 24.0)
        .with_task_group("Follow-up", "Efficacy", 18.0, 30.0)
        .with_milestone("Interim analysis", 12.0)
        .with_milestone("Primary endpoint", 24.0)
        .with_now_line(9.0);
    let plots = vec![Plot::Gantt(gp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Clinical Trial Timeline")
        .with_x_label("Month")
        .with_width(750.0)
        .with_height(380.0);
    let out = svg(plots, layout);
    std::fs::create_dir_all("test_outputs").unwrap();
    common::write_test_output("test_outputs/gantt_clinical.svg", &out).unwrap();
    assert!(out.contains("<svg"));
}

#[test]
fn test_gantt_software_sprints() {
    let gp = GanttPlot::new()
        .with_group_order(["Sprint 1", "Sprint 2", "Sprint 3"])
        .with_task_group_progress("Sprint 1", "Auth module", 0.0, 2.0, 1.0)
        .with_task_group_progress("Sprint 1", "User CRUD", 0.0, 2.0, 1.0)
        .with_task_group_progress("Sprint 2", "Dashboard", 2.0, 4.0, 0.8)
        .with_task_group_progress("Sprint 2", "API endpoints", 2.0, 4.0, 0.5)
        .with_task_group("Sprint 3", "Performance", 4.0, 6.0)
        .with_task_group("Sprint 3", "Testing & QA", 4.0, 6.0)
        .with_milestone("v1.0 release", 6.0)
        .with_now_line(3.2);
    let out = gantt_svg(gp);
    std::fs::create_dir_all("test_outputs").unwrap();
    common::write_test_output("test_outputs/gantt_sprints.svg", &out).unwrap();
    assert!(out.contains("<svg"));
}
