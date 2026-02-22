/// Visual theme controlling all plot chrome colors.
#[derive(Debug, Clone)]
pub struct Theme {
    pub background: String,
    pub axis_color: String,
    pub grid_color: String,
    pub tick_color: String,
    pub text_color: String,
    pub legend_bg: String,
    pub legend_border: String,
    pub pie_leader: String,
    pub box_median: String,
    pub violin_border: String,
    pub colorbar_border: String,
    pub font_family: Option<String>,
    pub show_grid: bool,
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}

impl Theme {
    pub fn light() -> Self {
        Self {
            background: "white".into(),
            axis_color: "black".into(),
            grid_color: "#ccc".into(),
            tick_color: "black".into(),
            text_color: "black".into(),
            legend_bg: "white".into(),
            legend_border: "black".into(),
            pie_leader: "#666".into(),
            box_median: "white".into(),
            violin_border: "black".into(),
            colorbar_border: "black".into(),
            font_family: None,
            show_grid: true,
        }
    }

    pub fn dark() -> Self {
        Self {
            background: "#1e1e1e".into(),
            axis_color: "#cccccc".into(),
            grid_color: "#444444".into(),
            tick_color: "#cccccc".into(),
            text_color: "#e0e0e0".into(),
            legend_bg: "#2d2d2d".into(),
            legend_border: "#666666".into(),
            pie_leader: "#999999".into(),
            box_median: "#1e1e1e".into(),
            violin_border: "#cccccc".into(),
            colorbar_border: "#cccccc".into(),
            font_family: None,
            show_grid: true,
        }
    }

    pub fn minimal() -> Self {
        Self {
            background: "white".into(),
            axis_color: "black".into(),
            grid_color: "#e0e0e0".into(),
            tick_color: "black".into(),
            text_color: "black".into(),
            legend_bg: "white".into(),
            legend_border: "none".into(),
            pie_leader: "#333".into(),
            box_median: "white".into(),
            violin_border: "black".into(),
            colorbar_border: "black".into(),
            font_family: Some("serif".into()),
            show_grid: false,
        }
    }

    pub fn solarized() -> Self {
        Self {
            background: "#fdf6e3".into(),
            axis_color: "#586e75".into(),
            grid_color: "#eee8d5".into(),
            tick_color: "#586e75".into(),
            text_color: "#657b83".into(),
            legend_bg: "#fdf6e3".into(),
            legend_border: "#93a1a1".into(),
            pie_leader: "#93a1a1".into(),
            box_median: "#fdf6e3".into(),
            violin_border: "#586e75".into(),
            colorbar_border: "#586e75".into(),
            font_family: None,
            show_grid: true,
        }
    }
}
