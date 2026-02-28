use std::io::{self, Read};
use std::path::Path;
use std::str::FromStr;

use clap::Args;

/// A column selector: either a 0-based integer index or a header name.
#[derive(Debug, Clone)]
pub enum ColSpec {
    Index(usize),
    Name(String),
}

impl FromStr for ColSpec {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(i) = s.parse::<usize>() {
            Ok(ColSpec::Index(i))
        } else {
            Ok(ColSpec::Name(s.to_string()))
        }
    }
}

#[derive(Args, Debug)]
#[command(next_help_heading = "Input")]
pub struct InputArgs {
    /// Input file (TSV or CSV). Omit or pass "-" to read from stdin.
    pub input: Option<std::path::PathBuf>,

    /// Treat the first row as data even if it looks like a header.
    #[arg(long)]
    pub no_header: bool,

    /// Override the field delimiter (default: auto-detect from extension or content).
    #[arg(long, short = 'd')]
    pub delimiter: Option<char>,
}

/// Parsed tabular data.
#[derive(Debug, Clone)]
pub struct DataTable {
    pub header: Option<Vec<String>>,
    /// Data rows (header excluded).
    pub rows: Vec<Vec<String>>,
}

impl DataTable {
    /// Read and parse input from a file path or stdin.
    pub fn parse(input: Option<&Path>, no_header: bool, delim_override: Option<char>) -> Result<Self, String> {
        let content = match input {
            Some(p) if p.to_str() != Some("-") => {
                std::fs::read_to_string(p)
                    .map_err(|e| format!("Cannot read {}: {e}", p.display()))?
            }
            _ => {
                let mut s = String::new();
                io::stdin()
                    .read_to_string(&mut s)
                    .map_err(|e| format!("Cannot read stdin: {e}"))?;
                s
            }
        };

        let delim = if let Some(d) = delim_override {
            d
        } else if let Some(p) = input {
            match p.extension().and_then(|e| e.to_str()).unwrap_or("") {
                "csv" => ',',
                "tsv" | "txt" => '\t',
                _ => sniff_delim(&content),
            }
        } else {
            sniff_delim(&content)
        };

        let all_lines: Vec<&str> = content.lines().collect();
        if all_lines.is_empty() {
            return Err("Input is empty".to_string());
        }

        let first_fields: Vec<String> = split_line(all_lines[0], delim);

        let has_header = if no_header {
            false
        } else {
            first_fields.first().map(|f| f.parse::<f64>().is_err()).unwrap_or(false)
        };

        let (header, data_start) = if has_header {
            (Some(first_fields), 1)
        } else {
            (None, 0)
        };

        let rows: Vec<Vec<String>> = all_lines[data_start..]
            .iter()
            .filter(|l| !l.trim().is_empty())
            .map(|l| split_line(l, delim))
            .collect();

        Ok(DataTable { header, rows })
    }

    /// Resolve a `ColSpec` to a 0-based column index.
    pub fn resolve(&self, col: &ColSpec) -> Result<usize, String> {
        match col {
            ColSpec::Index(i) => Ok(*i),
            ColSpec::Name(name) => {
                let header = self.header.as_ref().ok_or_else(|| {
                    format!("Column name '{name}' requested but no header row was detected. \
                             Use --no-header to force treating the first row as data, or \
                             use a 0-based integer index instead.")
                })?;
                header.iter().position(|h| h == name).ok_or_else(|| {
                    format!("Column '{name}' not found. Available columns: {}", header.join(", "))
                })
            }
        }
    }

    /// Extract a column as f64 values.
    pub fn col_f64(&self, col: &ColSpec) -> Result<Vec<f64>, String> {
        let idx = self.resolve(col)?;
        self.rows
            .iter()
            .enumerate()
            .map(|(row_i, row)| {
                row.get(idx)
                    .ok_or_else(|| format!("Row {row_i}: no column at index {idx}"))
                    .and_then(|s| {
                        s.parse::<f64>()
                            .map_err(|_| format!("Row {row_i}: cannot parse '{s}' as a number"))
                    })
            })
            .collect()
    }

    /// Extract a column as strings.
    pub fn col_str(&self, col: &ColSpec) -> Result<Vec<String>, String> {
        let idx = self.resolve(col)?;
        self.rows
            .iter()
            .enumerate()
            .map(|(row_i, row)| {
                row.get(idx)
                    .cloned()
                    .ok_or_else(|| format!("Row {row_i}: no column at index {idx}"))
            })
            .collect()
    }

    /// Split the table into groups by the distinct values in `col`.
    ///
    /// Groups are returned in first-seen order.
    pub fn group_by(&self, col: &ColSpec) -> Result<Vec<(String, DataTable)>, String> {
        let idx = self.resolve(col)?;
        let mut groups: Vec<(String, Vec<Vec<String>>)> = Vec::new();

        for row in &self.rows {
            let key = row.get(idx).cloned().unwrap_or_default();
            if let Some(g) = groups.iter_mut().find(|(k, _)| k == &key) {
                g.1.push(row.clone());
            } else {
                groups.push((key, vec![row.clone()]));
            }
        }

        Ok(groups
            .into_iter()
            .map(|(name, rows)| (name, DataTable { header: self.header.clone(), rows }))
            .collect())
    }
}

fn sniff_delim(content: &str) -> char {
    let first = content.lines().next().unwrap_or("");
    let tabs = first.chars().filter(|&c| c == '\t').count();
    let commas = first.chars().filter(|&c| c == ',').count();
    if tabs >= commas { '\t' } else { ',' }
}

fn split_line(line: &str, delim: char) -> Vec<String> {
    line.split(delim).map(|s| s.trim().to_string()).collect()
}
