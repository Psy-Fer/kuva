use std::collections::HashMap;



#[derive(Debug, Clone)]
pub struct BrickTemplate {
    pub template: HashMap<char, String>,
}


// add a way to auto generate colours from a string of letters
// this way, motifs from a STRIGAR can be given, A->Z and then assigned to a colour in legend
// make a legend....
impl BrickTemplate {
    pub fn new() -> Self {
        Self {
            template: HashMap::new(),
        }
    }

    pub fn dna(mut self) -> Self {
        self.template.insert('A', "#a4e9a4ff".into());
        self.template.insert('C', "#92d6f5ff".into());
        self.template.insert('G', "#9c9cecff".into());
        self.template.insert('T', "#f3a7f3ff".into());

        self
    }

    pub fn rna(mut self) -> Self {
        self.template.insert('A', "green".into());
        self.template.insert('C', "blue".into());
        self.template.insert('G', "orange".into());
        self.template.insert('U', "red".into());

        self
    }
}

#[derive(Debug, Clone)]
pub struct BrickPlot {
    pub sequences: Vec<String>, //DNA strings...could generalise later
    pub names: Vec<String>, // names matching each string
    pub template: Option<HashMap<char, String>>, // letter: colour - A: greeen
    pub show_values: bool, // show the letters or not
}

impl BrickPlot {
    pub fn new() -> Self {
        Self {
            sequences: vec![],
            names: vec![],
            template: Some(HashMap::new()),
            show_values: false,
        }
    }

    pub fn with_sequences<T, I>(mut self, sequences: I) -> Self 
    where 
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        self.sequences = sequences.into_iter().map(|x| x.into()).collect();

        self
    }

    pub fn with_names<T, I>(mut self, names: I) -> Self 
    where 
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        self.names = names.into_iter().map(|x| x.into()).collect();

        self
    }

    pub fn with_template(mut self, template: HashMap<char, String>) -> Self {
        self.template = Some(template);
        self
    }

    pub fn show_values(mut self) -> Self {
        self.show_values = true;
        self
    }
}
