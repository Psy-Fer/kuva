/// Strand orientation of a collinear block.
#[derive(Debug, Clone, PartialEq)]
pub enum Strand {
    Forward,
    Reverse,
}

/// A labelled sequence represented as a horizontal bar.
#[derive(Debug, Clone)]
pub struct SyntenySequence {
    pub label:  String,
    pub length: f64,
    pub color:  Option<String>,
}

/// A collinear block connecting a region on seq1 to a region on seq2.
#[derive(Debug, Clone)]
pub struct SyntenyBlock {
    pub seq1:   usize,
    pub start1: f64,
    pub end1:   f64,
    pub seq2:   usize,
    pub start2: f64,
    pub end2:   f64,
    pub strand: Strand,
    pub color:  Option<String>,
}

/// A synteny plot: horizontal sequence bars connected by ribbon polygons.
#[derive(Debug, Clone)]
pub struct SyntenyPlot {
    pub sequences:     Vec<SyntenySequence>,
    pub blocks:        Vec<SyntenyBlock>,
    /// Pixel height of each sequence bar (default 18.0).
    pub bar_height:    f64,
    /// Ribbon fill-opacity (default 0.65).
    pub block_opacity: f64,
    /// false = per-sequence scale (each bar fills full width); true = shared ruler.
    pub shared_scale:  bool,
    pub legend_label:  Option<String>,
}

impl Default for SyntenyPlot {
    fn default() -> Self { Self::new() }
}

impl SyntenyPlot {
    pub fn new() -> Self {
        Self {
            sequences:     vec![],
            blocks:        vec![],
            bar_height:    18.0,
            block_opacity: 0.65,
            shared_scale:  false,
            legend_label:  None,
        }
    }

    /// Add sequences from (label, length) pairs.
    pub fn with_sequences<S, L>(mut self, seqs: impl IntoIterator<Item = (S, L)>) -> Self
    where
        S: Into<String>,
        L: Into<f64>,
    {
        for (label, length) in seqs {
            self.sequences.push(SyntenySequence {
                label:  label.into(),
                length: length.into(),
                color:  None,
            });
        }
        self
    }

    /// Override bar colors (parallel to sequences).
    pub fn with_sequence_colors<C: Into<String>>(mut self, colors: impl IntoIterator<Item = C>) -> Self {
        for (seq, color) in self.sequences.iter_mut().zip(colors) {
            seq.color = Some(color.into());
        }
        self
    }

    /// Add a forward block.
    pub fn with_block(mut self, seq1: usize, start1: f64, end1: f64,
                      seq2: usize, start2: f64, end2: f64) -> Self {
        self.blocks.push(SyntenyBlock { seq1, start1, end1, seq2, start2, end2,
                                        strand: Strand::Forward, color: None });
        self
    }

    /// Add an inverted (crossed) block.
    pub fn with_inv_block(mut self, seq1: usize, start1: f64, end1: f64,
                          seq2: usize, start2: f64, end2: f64) -> Self {
        self.blocks.push(SyntenyBlock { seq1, start1, end1, seq2, start2, end2,
                                        strand: Strand::Reverse, color: None });
        self
    }

    /// Add a forward block with an explicit color.
    #[allow(clippy::too_many_arguments)]
    pub fn with_colored_block<C: Into<String>>(
        mut self, seq1: usize, start1: f64, end1: f64,
        seq2: usize, start2: f64, end2: f64, color: C,
    ) -> Self {
        self.blocks.push(SyntenyBlock { seq1, start1, end1, seq2, start2, end2,
                                        strand: Strand::Forward, color: Some(color.into()) });
        self
    }

    /// Add an inverted block with an explicit color.
    #[allow(clippy::too_many_arguments)]
    pub fn with_colored_inv_block<C: Into<String>>(
        mut self, seq1: usize, start1: f64, end1: f64,
        seq2: usize, start2: f64, end2: f64, color: C,
    ) -> Self {
        self.blocks.push(SyntenyBlock { seq1, start1, end1, seq2, start2, end2,
                                        strand: Strand::Reverse, color: Some(color.into()) });
        self
    }

    /// Batch-add pre-built blocks.
    pub fn with_blocks(mut self, blocks: impl IntoIterator<Item = SyntenyBlock>) -> Self {
        self.blocks.extend(blocks);
        self
    }

    pub fn with_bar_height(mut self, h: f64) -> Self {
        self.bar_height = h;
        self
    }

    pub fn with_opacity(mut self, opacity: f64) -> Self {
        self.block_opacity = opacity;
        self
    }

    /// Opt in to a shared coordinate ruler (shorter sequences draw narrower bars).
    pub fn with_shared_scale(mut self) -> Self {
        self.shared_scale = true;
        self
    }

    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }
}
