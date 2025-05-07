use crate::plot::types::BarX;

pub struct BarPlot {
    pub data: Vec<(BarX, f64)>, // (variable x, height)
    pub color: String,
    pub bar_width: Option<f64>,
}

impl BarPlot {
    pub fn new() -> Self {
        Self {
            data: vec![],
            bar_width: None,
            color: "black".to_string(),
        }
    }

    pub fn with_numeric_data(mut self, data: Vec<(f64, f64)>) -> Self {
        self.data = data.into_iter().map(|(x, y)| (BarX::Numeric(x), y)).collect();
        self
    }

    pub fn with_data(self, data: Vec<(f64, f64)>) -> Self {
        self.with_numeric_data(data)
    }


    pub fn with_categories_and_values<T: Into<String>>(categories: Vec<T>, values: Vec<f64>) -> Self {
        let data = categories
            .into_iter()
            .zip(values.into_iter())
            .map(|(cat, val)| (BarX::Category(cat.into()), val))
            .collect();

        Self {
            data,
            color: "black".to_string(),
            bar_width: None,
        }
    }

    pub fn resolve_bar_categories(&self) -> (Vec<(f64, f64)>, Vec<String>) {
        let mut categories = vec![];
        let mut mapping = std::collections::HashMap::new();
        let mut resolved = vec![];
    
        for (x, y) in self.data.iter() {
            match x {
                BarX::Numeric(n) => resolved.push((*n, *y)),
                BarX::Category(label) => {
                    let ix = *mapping.entry(label.clone()).or_insert_with(|| {
                        let pos = categories.len() as f64 + 1.0;
                        categories.push(label.clone());
                        pos
                    });
                    resolved.push((ix, *y));
                }
            }
        }
    
        (resolved, categories)
    }
    
    pub fn with_bar_width(mut self, width: f64) -> Self {
        self.bar_width = Some(width);
        self
    }

    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }
}