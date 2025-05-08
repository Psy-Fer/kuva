// use crate::plot::types::BarX;

pub struct BarPlot {
    pub groups: Vec<BarGroup>,
    pub width: f64,
    pub legend_label: Option<String>,
}

pub struct BarGroup {
    pub label: String,
    pub bars: Vec<BarValue>,
}

pub struct BarValue {
    pub value: f64,
    pub color: String,
}

impl BarPlot {
    pub fn new() -> Self {
        Self {
            groups: vec![],
            width: 0.8,
            legend_label: None,
        }
    }

    pub fn with_group<T: Into<String>>(mut self, label: T, values: Vec<(f64, &str)>) -> Self {
        let bars = values
                        .into_iter()
                        .map(|(v, c)| BarValue {
                            value: v,
                            color: c.into(),
                            
                        })
                        .collect();

        self.groups.push(BarGroup {
                        label: label.into(),
                        bars,
                    });
        self
    }

    pub fn with_width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    pub fn with_bar<T: Into<String>>(mut self, label: T, value: f64) -> Self {
        let color = self.default_color();
        self.groups.push(BarGroup {
            label: label.into(),
            bars: vec![BarValue { value, color }],
        });
        self
    }

    pub fn with_bars<T: Into<String>>(mut self, data: Vec<(T, f64)>) -> Self {
        let color = self.default_color();
        for (label, value) in data.into_iter() {
            self.groups.push(BarGroup {
                label: label.into(),
                bars: vec![BarValue { value: value,
                                      color: color.clone()
                                    }],
            });
        }
        self
    }



    pub fn with_color<T: Into<String>>(mut self, color: T) -> Self {
        let c = color.into();
        for group in &mut self.groups {
            for bar in &mut group.bars {
                bar.color = c.clone();
            }
        }
        self
    }

    fn default_color(&self) -> String {
        "steelblue".into()
    }

    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }
}