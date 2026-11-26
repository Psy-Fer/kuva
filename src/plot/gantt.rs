/// A single task (or milestone) in a Gantt chart.
pub struct GanttTask {
    pub label: String,
    /// Start time (x-axis value). Equal to `end` for milestones.
    pub start: f64,
    /// End time (x-axis value). Equal to `start` for milestones.
    pub end: f64,
    /// Optional group / phase name. Tasks with the same group are placed together.
    pub group: Option<String>,
    /// Completion fraction in `[0, 1]`. Rendered as a darker fill inside the bar.
    pub progress: Option<f64>,
    /// Per-task color override.
    pub color: Option<String>,
    /// When `true`, rendered as a diamond instead of a bar.
    pub is_milestone: bool,
}

/// A rendered display row — either a collapsible group header or a task bar.
pub enum GanttDisplayRow {
    /// Group header row; drawn with a background band.
    GroupHeader(String),
    /// Index into [`GanttPlot::tasks`].
    Task(usize),
}

/// Builder for a Gantt chart.
///
/// Tasks are grouped by phase (optional). Within each phase, tasks appear in
/// insertion order. Milestones are rendered as diamonds. An optional "now"
/// line marks the current date/time. Progress fills show task completion.
///
/// # Example
///
/// ```rust,no_run
/// use kuva::prelude::*;
///
/// let gantt = GanttPlot::new()
///     .with_task_group("Design", "Wireframes",   0.0, 3.0)
///     .with_task_group("Design", "Prototyping",  2.0, 5.0)
///     .with_task_group("Dev",    "Backend API",  3.0, 8.0)
///     .with_task_group_progress("Dev", "Frontend", 4.0, 9.0, 0.4)
///     .with_milestone("Launch", 10.0)
///     .with_now_line(6.0);
///
/// let plots = vec![Plot::from(gantt)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("Project Plan")
///     .with_x_label("Week");
/// ```
pub struct GanttPlot {
    pub tasks: Vec<GanttTask>,
    /// Explicit group ordering. Groups not listed appear in insertion order after.
    pub group_order: Vec<String>,
    /// If `Some(v)`, draw a vertical dashed line at x = v.
    pub now_line: Option<f64>,
    /// Bar height as a fraction of row height. Default `0.6`.
    pub bar_height_frac: f64,
    /// Diamond half-size in pixels for milestones. Default `7.0`.
    pub milestone_size: f64,
    /// When `true`, task labels are drawn inside (or beside) bars. Default `true`.
    pub show_labels: bool,
    /// Minimum bar pixel width to attempt an inside label. Default `40.0`.
    pub label_min_width: f64,
    /// Default bar color when no group color or per-task color applies. Default `"steelblue"`.
    pub color: String,
    /// Background band color for group header rows. Default `"#ebebeb"`.
    pub group_bg: String,
    pub legend_label: Option<String>,
}

impl Default for GanttPlot {
    fn default() -> Self {
        Self::new()
    }
}

impl GanttPlot {
    pub fn new() -> Self {
        Self {
            tasks: vec![],
            group_order: vec![],
            now_line: None,
            bar_height_frac: 0.6,
            milestone_size: 7.0,
            show_labels: true,
            label_min_width: 40.0,
            color: "steelblue".into(),
            group_bg: "#ebebeb".into(),
            legend_label: None,
        }
    }

    /// Add an ungrouped task.
    pub fn with_task(
        mut self,
        label: impl Into<String>,
        start: impl Into<f64>,
        end: impl Into<f64>,
    ) -> Self {
        self.tasks.push(GanttTask {
            label: label.into(),
            start: start.into(),
            end: end.into(),
            group: None,
            progress: None,
            color: None,
            is_milestone: false,
        });
        self
    }

    /// Add a task belonging to a named group/phase.
    pub fn with_task_group(
        mut self,
        group: impl Into<String>,
        label: impl Into<String>,
        start: impl Into<f64>,
        end: impl Into<f64>,
    ) -> Self {
        self.tasks.push(GanttTask {
            label: label.into(),
            start: start.into(),
            end: end.into(),
            group: Some(group.into()),
            progress: None,
            color: None,
            is_milestone: false,
        });
        self
    }

    /// Add an ungrouped task with a progress fill (`0.0`–`1.0`).
    pub fn with_task_progress(
        mut self,
        label: impl Into<String>,
        start: impl Into<f64>,
        end: impl Into<f64>,
        progress: impl Into<f64>,
    ) -> Self {
        self.tasks.push(GanttTask {
            label: label.into(),
            start: start.into(),
            end: end.into(),
            group: None,
            progress: Some(progress.into().clamp(0.0, 1.0)),
            color: None,
            is_milestone: false,
        });
        self
    }

    /// Add a grouped task with a progress fill.
    pub fn with_task_group_progress(
        mut self,
        group: impl Into<String>,
        label: impl Into<String>,
        start: impl Into<f64>,
        end: impl Into<f64>,
        progress: impl Into<f64>,
    ) -> Self {
        self.tasks.push(GanttTask {
            label: label.into(),
            start: start.into(),
            end: end.into(),
            group: Some(group.into()),
            progress: Some(progress.into().clamp(0.0, 1.0)),
            color: None,
            is_milestone: false,
        });
        self
    }

    /// Add an ungrouped task with a per-task color override.
    pub fn with_colored_task(
        mut self,
        label: impl Into<String>,
        start: impl Into<f64>,
        end: impl Into<f64>,
        color: impl Into<String>,
    ) -> Self {
        self.tasks.push(GanttTask {
            label: label.into(),
            start: start.into(),
            end: end.into(),
            group: None,
            progress: None,
            color: Some(color.into()),
            is_milestone: false,
        });
        self
    }

    /// Add a milestone (diamond marker) with no group.
    pub fn with_milestone(mut self, label: impl Into<String>, at: impl Into<f64>) -> Self {
        let at = at.into();
        self.tasks.push(GanttTask {
            label: label.into(),
            start: at,
            end: at,
            group: None,
            progress: None,
            color: None,
            is_milestone: true,
        });
        self
    }

    /// Add a milestone belonging to a named group/phase.
    pub fn with_milestone_group(
        mut self,
        group: impl Into<String>,
        label: impl Into<String>,
        at: impl Into<f64>,
    ) -> Self {
        let at = at.into();
        self.tasks.push(GanttTask {
            label: label.into(),
            start: at,
            end: at,
            group: Some(group.into()),
            progress: None,
            color: None,
            is_milestone: true,
        });
        self
    }

    /// Set the x-value for the vertical "now" reference line.
    pub fn with_now_line(mut self, value: impl Into<f64>) -> Self {
        self.now_line = Some(value.into());
        self
    }

    /// Override the display order of groups. Unlisted groups follow in insertion order.
    pub fn with_group_order(mut self, groups: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.group_order = groups.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Set bar height as fraction of row height. Default `0.6`.
    pub fn with_bar_height(mut self, frac: f64) -> Self {
        self.bar_height_frac = frac.clamp(0.1, 1.0);
        self
    }

    /// Set milestone diamond half-size in pixels. Default `7.0`.
    pub fn with_milestone_size(mut self, size: f64) -> Self {
        self.milestone_size = size;
        self
    }

    /// Show or hide task labels. Default `true`.
    pub fn with_show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }

    /// Set the default bar color (used when there are no groups). Default `"steelblue"`.
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = color.into();
        self
    }

    /// Set the group header row background color. Default `"#ebebeb"`.
    pub fn with_group_bg(mut self, color: impl Into<String>) -> Self {
        self.group_bg = color.into();
        self
    }

    /// Attach a legend label (shows a colored rect entry).
    pub fn with_legend(mut self, label: impl Into<String>) -> Self {
        self.legend_label = Some(label.into());
        self
    }

    /// Returns groups in effective display order.
    /// Named groups appear first (in group_order, then insertion order),
    /// ungrouped tasks last (represented as `None`).
    pub fn effective_group_order(&self) -> Vec<Option<String>> {
        // A set alongside the list keeps this linear in tasks plus groups.
        let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
        let mut ordered: Vec<Option<String>> = vec![];
        let named = self
            .group_order
            .iter()
            .chain(self.tasks.iter().filter_map(|t| t.group.as_ref()));
        for g in named {
            if seen.insert(g.as_str()) {
                ordered.push(Some(g.clone()));
            }
        }
        if self.tasks.iter().any(|t| t.group.is_none()) {
            ordered.push(None);
        }
        ordered
    }

    /// Returns display rows in top-to-bottom order.
    pub fn ordered_display_rows(&self) -> Vec<GanttDisplayRow> {
        let groups = self.effective_group_order();
        let has_groups = groups.iter().any(|g| g.is_some());
        // Task indices per group, in task order, built in one pass.
        let mut by_group: std::collections::HashMap<Option<&str>, Vec<usize>> =
            std::collections::HashMap::new();
        for (i, task) in self.tasks.iter().enumerate() {
            by_group.entry(task.group.as_deref()).or_default().push(i);
        }
        let mut rows = vec![];
        for group_key in &groups {
            if has_groups {
                if let Some(ref g) = group_key {
                    rows.push(GanttDisplayRow::GroupHeader(g.clone()));
                }
            }
            if let Some(tasks) = by_group.get(&group_key.as_deref()) {
                rows.extend(tasks.iter().map(|&i| GanttDisplayRow::Task(i)));
            }
        }
        rows
    }

    /// Row labels in top-to-bottom display order (used to build y_categories).
    pub fn row_labels(&self) -> Vec<String> {
        self.ordered_display_rows()
            .into_iter()
            .map(|r| match r {
                GanttDisplayRow::GroupHeader(g) => g,
                GanttDisplayRow::Task(i) => self.tasks[i].label.clone(),
            })
            .collect()
    }

    /// Compute x-axis bounds across all tasks and the now line.
    pub fn x_bounds(&self) -> Option<(f64, f64)> {
        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        for t in &self.tasks {
            x_min = x_min.min(t.start);
            x_max = x_max.max(t.end);
        }
        if let Some(now) = self.now_line {
            x_min = x_min.min(now);
            x_max = x_max.max(now);
        }
        if x_min.is_finite() {
            Some((x_min, x_max))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The scanning versions the set-based ones replaced, as (header, task
    /// index) pairs so they can be compared.
    fn order_ref(p: &GanttPlot) -> Vec<Option<String>> {
        let mut ordered: Vec<Option<String>> = vec![];
        for g in &p.group_order {
            if !ordered.contains(&Some(g.clone())) {
                ordered.push(Some(g.clone()));
            }
        }
        for task in &p.tasks {
            if let Some(ref g) = task.group {
                if !ordered.contains(&Some(g.clone())) {
                    ordered.push(Some(g.clone()));
                }
            }
        }
        if p.tasks.iter().any(|t| t.group.is_none()) {
            ordered.push(None);
        }
        ordered
    }

    fn rows_ref(p: &GanttPlot) -> Vec<(Option<String>, Option<usize>)> {
        let groups = order_ref(p);
        let has_groups = groups.iter().any(|g| g.is_some());
        let mut rows = vec![];
        for group_key in &groups {
            if has_groups {
                if let Some(ref g) = group_key {
                    rows.push((Some(g.clone()), None));
                }
            }
            for (i, task) in p.tasks.iter().enumerate() {
                let belongs = match group_key {
                    Some(g) => task.group.as_deref() == Some(g.as_str()),
                    None => task.group.is_none(),
                };
                if belongs {
                    rows.push((None, Some(i)));
                }
            }
        }
        rows
    }

    fn rows(p: &GanttPlot) -> Vec<(Option<String>, Option<usize>)> {
        p.ordered_display_rows()
            .into_iter()
            .map(|r| match r {
                GanttDisplayRow::GroupHeader(g) => (Some(g), None),
                GanttDisplayRow::Task(i) => (None, Some(i)),
            })
            .collect()
    }

    #[test]
    fn set_based_ordering_matches_the_scans() {
        let names = ["a", "b", "c", "d"];
        let mut state = 0x1234_5678_9abc_def1_u64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        for _ in 0..500 {
            let mut p = GanttPlot::new();
            for t in 0..(next() % 12) {
                let pick = next() % 5;
                p = if pick == 4 {
                    p.with_task(format!("t{t}"), 0.0, 1.0)
                } else {
                    p.with_task_group(names[pick as usize], format!("t{t}"), 0.0, 1.0)
                };
            }
            // Explicit order: some names, possibly repeated or unused.
            let order: Vec<&str> = (0..(next() % 4))
                .map(|_| names[(next() % 4) as usize])
                .collect();
            p = p.with_group_order(order);
            assert_eq!(p.effective_group_order(), order_ref(&p));
            assert_eq!(rows(&p), rows_ref(&p));
        }
    }
}
