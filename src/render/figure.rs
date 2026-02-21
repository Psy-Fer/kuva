use crate::render::layout::Layout;
use crate::render::plots::Plot;
use crate::render::render::{Primitive, Scene, TextAnchor, render_multiple};

#[derive(Debug, Clone)]
pub enum LabelStyle {
    Uppercase,
    Lowercase,
    Numeric,
    Custom(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct LabelConfig {
    pub style: LabelStyle,
    pub size: u32,
    pub bold: bool,
}

impl Default for LabelConfig {
    fn default() -> Self {
        Self {
            style: LabelStyle::Uppercase,
            size: 16,
            bold: true,
        }
    }
}

impl LabelConfig {
    fn label_for(&self, index: usize) -> String {
        match &self.style {
            LabelStyle::Uppercase => {
                let c = (b'A' + index as u8) as char;
                c.to_string()
            }
            LabelStyle::Lowercase => {
                let c = (b'a' + index as u8) as char;
                c.to_string()
            }
            LabelStyle::Numeric => {
                (index + 1).to_string()
            }
            LabelStyle::Custom(labels) => {
                labels.get(index).cloned().unwrap_or_default()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum SharedAxis {
    AllRows,
    AllColumns,
    Row(usize),
    Column(usize),
    RowSlice { row: usize, col_start: usize, col_end: usize },
    ColumnSlice { col: usize, row_start: usize, row_end: usize },
}

pub struct Figure {
    rows: usize,
    cols: usize,
    structure: Vec<Vec<usize>>,
    plots: Vec<Vec<Plot>>,
    layouts: Vec<Layout>,
    title: Option<String>,
    title_size: u32,
    labels: Option<LabelConfig>,
    shared_x: Vec<SharedAxis>,
    shared_y: Vec<SharedAxis>,
    spacing: f64,
    padding: f64,
    cell_width: f64,
    cell_height: f64,
}

impl Figure {
    pub fn new(rows: usize, cols: usize) -> Self {
        let structure: Vec<Vec<usize>> = (0..rows * cols).map(|i| vec![i]).collect();
        Self {
            rows,
            cols,
            structure,
            plots: Vec::new(),
            layouts: Vec::new(),
            title: None,
            title_size: 20,
            labels: None,
            shared_x: Vec::new(),
            shared_y: Vec::new(),
            spacing: 15.0,
            padding: 10.0,
            cell_width: 350.0,
            cell_height: 280.0,
        }
    }

    pub fn with_title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_title_size(mut self, size: u32) -> Self {
        self.title_size = size;
        self
    }

    pub fn with_structure(mut self, structure: Vec<Vec<usize>>) -> Self {
        self.structure = structure;
        self
    }

    pub fn with_plots(mut self, plots: Vec<Vec<Plot>>) -> Self {
        self.plots = plots;
        self
    }

    pub fn with_layouts(mut self, layouts: Vec<Layout>) -> Self {
        self.layouts = layouts;
        self
    }

    /// Add bold uppercase panel labels (A, B, C, ...)
    pub fn with_labels(mut self) -> Self {
        self.labels = Some(LabelConfig::default());
        self
    }

    pub fn with_labels_numeric(mut self) -> Self {
        self.labels = Some(LabelConfig {
            style: LabelStyle::Numeric,
            ..LabelConfig::default()
        });
        self
    }

    pub fn with_labels_lowercase(mut self) -> Self {
        self.labels = Some(LabelConfig {
            style: LabelStyle::Lowercase,
            ..LabelConfig::default()
        });
        self
    }

    pub fn with_labels_custom(mut self, labels: Vec<&str>, config: LabelConfig) -> Self {
        self.labels = Some(LabelConfig {
            style: LabelStyle::Custom(labels.into_iter().map(|s| s.to_string()).collect()),
            size: config.size,
            bold: config.bold,
        });
        self
    }

    /// Share X axis across all columns (blanket).
    pub fn with_shared_x_all(mut self) -> Self {
        self.shared_x.push(SharedAxis::AllColumns);
        self
    }

    /// Share Y axis across all rows (blanket).
    pub fn with_shared_y_all(mut self) -> Self {
        self.shared_y.push(SharedAxis::AllRows);
        self
    }

    /// Share X axis within a single column.
    pub fn with_shared_x(mut self, col: usize) -> Self {
        self.shared_x.push(SharedAxis::Column(col));
        self
    }

    /// Share Y axis within a single row.
    pub fn with_shared_y(mut self, row: usize) -> Self {
        self.shared_y.push(SharedAxis::Row(row));
        self
    }

    /// Share X axis within a column for a slice of rows.
    pub fn with_shared_x_slice(mut self, col: usize, row_start: usize, row_end: usize) -> Self {
        self.shared_x.push(SharedAxis::ColumnSlice { col, row_start, row_end });
        self
    }

    /// Share Y axis within a row for a slice of columns.
    pub fn with_shared_y_slice(mut self, row: usize, col_start: usize, col_end: usize) -> Self {
        self.shared_y.push(SharedAxis::RowSlice { row, col_start, col_end });
        self
    }

    pub fn with_spacing(mut self, px: f64) -> Self {
        self.spacing = px;
        self
    }

    pub fn with_padding(mut self, px: f64) -> Self {
        self.padding = px;
        self
    }

    pub fn with_cell_size(mut self, w: f64, h: f64) -> Self {
        self.cell_width = w;
        self.cell_height = h;
        self
    }

    pub fn render(self) -> Scene {
        let Figure {
            rows, cols, structure, mut plots, layouts: user_layouts,
            title, title_size, labels, shared_x, shared_y,
            spacing, padding, cell_width, cell_height,
        } = self;

        validate_structure(&structure, rows, cols);

        let figure_title_height = if title.is_some() { 30.0 } else { 0.0 };
        let total_width = cols as f64 * cell_width
            + (cols as f64 - 1.0) * spacing
            + 2.0 * padding;
        let total_height = rows as f64 * cell_height
            + (rows as f64 - 1.0) * spacing
            + 2.0 * padding
            + figure_title_height;

        let mut master = Scene::new(total_width, total_height);

        // Build a layout for each structure slot
        let mut layouts: Vec<Layout> = Vec::new();
        for i in 0..structure.len() {
            let layout = if i < user_layouts.len() {
                clone_layout(&user_layouts[i])
            } else if i < plots.len() && !plots[i].is_empty() {
                Layout::auto_from_plots(&plots[i])
            } else {
                Layout::new((0.0, 1.0), (0.0, 1.0))
            };
            layouts.push(layout);
        }

        // Apply shared axis rules
        apply_shared_axes(&structure, &shared_y, &shared_x, &mut layouts, rows, cols);

        // Pad plots with empty vecs so indexing is safe
        while plots.len() < structure.len() {
            plots.push(Vec::new());
        }

        for (i, group) in structure.iter().enumerate() {
            let rect = cell_rect(group, cols);
            let col_span = rect.3 - rect.1 + 1;
            let row_span = rect.2 - rect.0 + 1;

            let cell_x = padding + rect.1 as f64 * (cell_width + spacing);
            let cell_y = padding + figure_title_height + rect.0 as f64 * (cell_height + spacing);
            let cell_w = col_span as f64 * cell_width + (col_span as f64 - 1.0) * spacing;
            let cell_h = row_span as f64 * cell_height + (row_span as f64 - 1.0) * spacing;

            let slot_plots = std::mem::take(&mut plots[i]);

            if !slot_plots.is_empty() {
                let mut layout = clone_layout(&layouts[i]);
                layout.width = Some(cell_w);
                layout.height = Some(cell_h);

                let cell_scene = render_multiple(slot_plots, layout);

                master.add(Primitive::GroupStart {
                    transform: Some(format!("translate({cell_x},{cell_y})")),
                });
                for elem in cell_scene.elements {
                    master.add(elem);
                }
                master.add(Primitive::GroupEnd);
            }

            if let Some(ref config) = labels {
                let label = config.label_for(i);
                master.add(Primitive::Text {
                    x: cell_x + 8.0,
                    y: cell_y + config.size as f64 + 2.0,
                    content: label,
                    size: config.size,
                    anchor: TextAnchor::Start,
                    rotate: None,
                    bold: config.bold,
                });
            }
        }

        if let Some(title) = title {
            master.add(Primitive::Text {
                x: total_width / 2.0,
                y: 22.0,
                content: title,
                size: title_size,
                anchor: TextAnchor::Middle,
                rotate: None,
                bold: false,
            });
        }

        master
    }
}

/// Clone a Layout field by field.
fn clone_layout(l: &Layout) -> Layout {
    let mut new = Layout::new(l.x_range, l.y_range);
    new.width = l.width;
    new.height = l.height;
    new.ticks = l.ticks;
    new.show_grid = l.show_grid;
    new.x_label = l.x_label.clone();
    new.y_label = l.y_label.clone();
    new.title = l.title.clone();
    new.x_categories = l.x_categories.clone();
    new.y_categories = l.y_categories.clone();
    new.show_legend = l.show_legend;
    new.show_colorbar = l.show_colorbar;
    new.legend_position = l.legend_position;
    new.legend_width = l.legend_width;
    new.log_x = l.log_x;
    new.log_y = l.log_y;
    new.suppress_x_ticks = l.suppress_x_ticks;
    new.suppress_y_ticks = l.suppress_y_ticks;
    new
}

/// Returns (min_row, min_col, max_row, max_col) for a group of cell indices.
fn cell_rect(group: &[usize], cols: usize) -> (usize, usize, usize, usize) {
    let mut min_row = usize::MAX;
    let mut max_row = 0;
    let mut min_col = usize::MAX;
    let mut max_col = 0;
    for &idx in group {
        let row = idx / cols;
        let col = idx % cols;
        min_row = min_row.min(row);
        max_row = max_row.max(row);
        min_col = min_col.min(col);
        max_col = max_col.max(col);
    }
    (min_row, min_col, max_row, max_col)
}

fn validate_structure(structure: &[Vec<usize>], rows: usize, cols: usize) {
    let total_cells = rows * cols;
    let mut seen = vec![false; total_cells];

    for (group_idx, group) in structure.iter().enumerate() {
        assert!(!group.is_empty(), "Figure structure: group {group_idx} is empty");

        for &idx in group {
            assert!(
                idx < total_cells,
                "Figure structure: cell index {idx} out of bounds (grid is {rows}x{cols} = {total} cells)",
                total = total_cells,
            );
            assert!(
                !seen[idx],
                "Figure structure: cell index {idx} appears in multiple groups"
            );
            seen[idx] = true;
        }

        let (min_row, min_col, max_row, max_col) = cell_rect(group, cols);
        let expected_count = (max_row - min_row + 1) * (max_col - min_col + 1);
        assert!(
            group.len() == expected_count,
            "Figure structure: group {group_idx} is not a filled rectangle \
             (has {} cells, expected {expected_count} for rows {min_row}..={max_row}, cols {min_col}..={max_col})",
            group.len(),
        );

        for r in min_row..=max_row {
            for c in min_col..=max_col {
                let cell = r * cols + c;
                assert!(
                    group.contains(&cell),
                    "Figure structure: group {group_idx} missing cell {cell} \
                     (row {r}, col {c}) — groups must be filled rectangles"
                );
            }
        }
    }
}

fn subplot_grid_pos(structure: &[Vec<usize>], subplot_idx: usize, cols: usize) -> Option<(usize, usize)> {
    if subplot_idx >= structure.len() { return None; }
    let group = &structure[subplot_idx];
    if group.is_empty() { return None; }
    let (min_row, min_col, _, _) = cell_rect(group, cols);
    Some((min_row, min_col))
}

fn apply_shared_axes(
    structure: &[Vec<usize>],
    shared_y_rules: &[SharedAxis],
    shared_x_rules: &[SharedAxis],
    layouts: &mut [Layout],
    _rows: usize,
    cols: usize,
) {
    for rule in shared_y_rules {
        let groups = matching_groups_for_shared_y(structure, rule, cols);
        if groups.len() < 2 { continue; }

        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;
        for &idx in &groups {
            if idx < layouts.len() {
                y_min = y_min.min(layouts[idx].y_range.0);
                y_max = y_max.max(layouts[idx].y_range.1);
            }
        }

        let leftmost_col = groups.iter()
            .filter_map(|&idx| subplot_grid_pos(structure, idx, cols))
            .map(|(_, col)| col)
            .min()
            .unwrap_or(0);

        for &idx in &groups {
            if idx < layouts.len() {
                layouts[idx].y_range = (y_min, y_max);
                if let Some((_, col)) = subplot_grid_pos(structure, idx, cols) {
                    if col != leftmost_col {
                        layouts[idx].suppress_y_ticks = true;
                        layouts[idx].y_label = None;
                    }
                }
            }
        }
    }

    for rule in shared_x_rules {
        let groups = matching_groups_for_shared_x(structure, rule, cols);
        if groups.len() < 2 { continue; }

        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        for &idx in &groups {
            if idx < layouts.len() {
                x_min = x_min.min(layouts[idx].x_range.0);
                x_max = x_max.max(layouts[idx].x_range.1);
            }
        }

        let bottommost_row = groups.iter()
            .filter_map(|&idx| subplot_grid_pos(structure, idx, cols))
            .map(|(row, _)| row)
            .max()
            .unwrap_or(0);

        for &idx in &groups {
            if idx < layouts.len() {
                layouts[idx].x_range = (x_min, x_max);
                if let Some((row, _)) = subplot_grid_pos(structure, idx, cols) {
                    if row != bottommost_row {
                        layouts[idx].suppress_x_ticks = true;
                        layouts[idx].x_label = None;
                    }
                }
            }
        }
    }
}

fn matching_groups_for_shared_y(structure: &[Vec<usize>], rule: &SharedAxis, cols: usize) -> Vec<usize> {
    let mut result = Vec::new();
    for (idx, _) in structure.iter().enumerate() {
        if let Some((row, col)) = subplot_grid_pos(structure, idx, cols) {
            let matches = match rule {
                SharedAxis::AllRows => true,
                SharedAxis::Row(r) => row == *r,
                SharedAxis::RowSlice { row: r, col_start, col_end } => {
                    row == *r && col >= *col_start && col <= *col_end
                }
                _ => false,
            };
            if matches { result.push(idx); }
        }
    }
    result
}

fn matching_groups_for_shared_x(structure: &[Vec<usize>], rule: &SharedAxis, cols: usize) -> Vec<usize> {
    let mut result = Vec::new();
    for (idx, _) in structure.iter().enumerate() {
        if let Some((row, col)) = subplot_grid_pos(structure, idx, cols) {
            let matches = match rule {
                SharedAxis::AllColumns => true,
                SharedAxis::Column(c) => col == *c,
                SharedAxis::ColumnSlice { col: c, row_start, row_end } => {
                    col == *c && row >= *row_start && row <= *row_end
                }
                _ => false,
            };
            if matches { result.push(idx); }
        }
    }
    result
}
