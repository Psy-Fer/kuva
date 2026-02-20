use std::collections::HashMap;


fn canonical_rotation(s: &str) -> String {
    let n = s.len();
    if n == 0 { return String::new(); }
    let doubled = format!("{}{}", s, s);
    (0..n)
        .map(|i| &doubled[i..i + n])
        .min()
        .unwrap()
        .to_string()
}

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
        self.template.insert('A', "rgb(0,150,0)".into());
        self.template.insert('C', "rgb(0,0,255)".into());
        self.template.insert('G', "rgb(209,113,5)".into());
        self.template.insert('T', "rgb(255,0,0)".into());

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
    pub sequences: Vec<String>, //ordered DNA strings...could generalise later
    pub names: Vec<String>, // ordered names matching each string
    pub strigars: Option<Vec<(String, String)>>, // strigar strings (motifs, strigar) as they come from bladerunner, also changes the plot to strigar mode
    pub motifs: Option<HashMap<char, String>>, // motifs in the structure (A: CAG)
    pub strigar_exp: Option<Vec<String>>, // expanded strigar, so 3A-> AAA
    pub template: Option<HashMap<char, String>>, // letter: colour - A: greeen
    pub x_offset: f64, // offset for x axis zero, to set the start of area of interest
    pub x_offsets: Option<Vec<f64>>, // ordered list of zero offsets matching the same order as names and sequences
    pub motif_lengths: Option<HashMap<char, usize>>, // global letter → motif nucleotide length (for variable-width bricks)
    pub show_values: bool, // show the letters or not
}

impl BrickPlot {
    pub fn new() -> Self {
        Self {
            sequences: vec![],
            names: vec![],
            strigars: None,
            motifs: None,
            strigar_exp: None,
            template: Some(HashMap::new()),
            motif_lengths: None,
            x_offset: 0.0,
            x_offsets: None,
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

    pub fn with_strigars<T, U, I>(mut self, strigars: I) -> Self 
    where 
        I: IntoIterator<Item = (T, U)>,
        T: Into<String>,
        U: Into<String>,
    {
        self.strigars = Some(strigars.into_iter()
                                .map(|(motif, strigar)| (motif.into(), strigar.into()))
                                .collect());

        // Phase A: Parse each read's motif string into local_letter → kmer map
        let per_read_maps: Vec<HashMap<char, String>> = self.strigars.as_ref().unwrap().iter()
            .map(|(motif_str, _)| {
                motif_str.split(',')
                    .map(|pair| {
                        let parts: Vec<&str> = pair.split(':').collect();
                        (parts[1].chars().next().unwrap(), parts[0].to_string())
                    })
                    .collect()
            })
            .collect();

        // Phase B: Collect all kmers, canonicalize, count frequencies
        let mut canonical_freq: HashMap<String, usize> = HashMap::new();
        let mut rotation_freq: HashMap<String, HashMap<String, usize>> = HashMap::new();
        for read_map in &per_read_maps {
            for (_letter, kmer) in read_map {
                let canon = canonical_rotation(kmer);
                *canonical_freq.entry(canon.clone()).or_insert(0) += 1;
                *rotation_freq.entry(canon).or_default().entry(kmer.clone()).or_insert(0) += 1;
            }
        }

        // Phase C: Sort canonicals by frequency desc, assign global letters
        let mut sorted_canonicals: Vec<(String, usize)> = canonical_freq.into_iter().collect();
        sorted_canonicals.sort_by(|a, b| b.1.cmp(&a.1));

        let mut canonical_to_global: HashMap<String, char> = HashMap::new();
        let mut global_to_display: HashMap<char, String> = HashMap::new();
        let mut global_to_length: HashMap<char, usize> = HashMap::new();

        for (idx, (canon, _freq)) in sorted_canonicals.iter().enumerate() {
            let global_letter = (b'A' + idx as u8) as char;
            canonical_to_global.insert(canon.clone(), global_letter);

            // Pick the most-frequent original rotation as display label
            let rotations = rotation_freq.get(canon).unwrap();
            let display = rotations.iter().max_by_key(|(_, count)| *count).unwrap().0.clone();
            global_to_display.insert(global_letter, display.clone());
            global_to_length.insert(global_letter, display.len());
        }

        // Phase D: Remap each read's strigar to global letters and expand
        let mut expanded_strigars: Vec<String> = vec![];

        for (i, (_motif_str, strigar_str)) in self.strigars.as_ref().unwrap().iter().enumerate() {
            let read_map = &per_read_maps[i];

            // Build local_letter → global_letter mapping for this read
            let mut local_to_global: HashMap<char, char> = HashMap::new();
            for (local_letter, kmer) in read_map {
                let canon = canonical_rotation(kmer);
                let global = canonical_to_global[&canon];
                local_to_global.insert(*local_letter, global);
            }

            // Remap and expand: "10A1B4A" with A→X, B→Y → "XXXXXXXXXXYYYYY..."
            let expanded: String = strigar_str.split(char::is_alphabetic)
                .zip(strigar_str.matches(char::is_alphabetic))
                .map(|(num, ch)| {
                    let local = ch.chars().next().unwrap();
                    let global = local_to_global[&local];
                    global.to_string().repeat(num.parse::<usize>().unwrap())
                })
                .collect();

            expanded_strigars.push(expanded);
        }

        // Phase E: Auto-generate template colours
        let motif_colors: &[&str] = &[
            "rgb(31,119,180)",   // blue
            "rgb(255,127,14)",   // orange
            "rgb(44,160,44)",    // green
            "rgb(214,39,40)",    // red
            "rgb(148,103,189)",  // purple
            "rgb(140,86,75)",    // brown
            "rgb(227,119,194)",  // pink
            "rgb(127,127,127)",  // gray
            "rgb(188,189,34)",   // olive
            "rgb(23,190,207)",   // cyan
        ];
        let mut auto_template: HashMap<char, String> = HashMap::new();
        for (idx, (canon, _)) in sorted_canonicals.iter().enumerate() {
            let global_letter = canonical_to_global[canon];
            auto_template.insert(global_letter, motif_colors[idx % motif_colors.len()].to_string());
        }

        self.template = Some(auto_template);
        self.motifs = Some(global_to_display);
        self.strigar_exp = Some(expanded_strigars);
        self.motif_lengths = Some(global_to_length);

        self

    }

    pub fn with_template(mut self, template: HashMap<char, String>) -> Self {
        self.template = Some(template);
        self
    }

    // a global x_offset
    // TODO: do per read offset, so it's the repeat start site in the read
    pub fn with_x_offset(mut self, x_offset: f64) -> Self {
        self.x_offset = x_offset;
        self
    }

    pub fn show_values(mut self) -> Self {
        self.show_values = true;
        self
    }
}
