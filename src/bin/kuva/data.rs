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
    /// Input file (TSV, CSV, or Parquet). Omit or pass "-" to read from stdin.
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
    ///
    /// `project` lists the columns to read.  For parquet files only the
    /// requested columns are decoded from disk (projected Arrow read), keeping
    /// memory and time proportional to the selected columns rather than the
    /// full schema.  Pass an empty slice to read every column.  For CSV/TSV
    /// the parameter is accepted but ignored — the whole file is always read.
    ///
    /// When the `parquet` feature is enabled, `.parquet` files and parquet
    /// piped via stdin (detected by magic bytes `PAR1`) are handled
    /// automatically; no flag is needed.
    #[cfg_attr(not(feature = "parquet"), allow(unused_variables))]
    pub fn parse(
        input: Option<&Path>,
        no_header: bool,
        delim_override: Option<char>,
        project: &[ColSpec],
    ) -> Result<Self, String> {
        match input {
            Some(p) if p.to_str() != Some("-") => {
                #[cfg(feature = "parquet")]
                if p.extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.eq_ignore_ascii_case("parquet"))
                    .unwrap_or(false)
                {
                    if no_header {
                        eprintln!("warning: --no-header is ignored for parquet input");
                    }
                    if delim_override.is_some() {
                        eprintln!("warning: --delimiter is ignored for parquet input");
                    }
                    let file = std::fs::File::open(p)
                        .map_err(|e| format!("Cannot open {}: {e}", p.display()))?;
                    return from_parquet_projected(file, project);
                }
                let content = std::fs::read_to_string(p)
                    .map_err(|e| format!("Cannot read {}: {e}", p.display()))?;
                Self::parse_str(&content, input, no_header, delim_override)
            }
            _ => {
                let mut buf = Vec::new();
                io::stdin()
                    .read_to_end(&mut buf)
                    .map_err(|e| format!("Cannot read stdin: {e}"))?;

                #[cfg(feature = "parquet")]
                if sniff_parquet(&buf) {
                    if no_header {
                        eprintln!("warning: --no-header is ignored for parquet input");
                    }
                    if delim_override.is_some() {
                        eprintln!("warning: --delimiter is ignored for parquet input");
                    }
                    return from_parquet_projected(bytes::Bytes::from(buf), project);
                }

                let content =
                    String::from_utf8(buf).map_err(|e| format!("stdin is not valid UTF-8: {e}"))?;
                Self::parse_str(&content, None, no_header, delim_override)
            }
        }
    }

    pub(crate) fn parse_str(
        content: &str,
        input: Option<&Path>,
        no_header: bool,
        delim_override: Option<char>,
    ) -> Result<Self, String> {
        let delim = if let Some(d) = delim_override {
            d
        } else if let Some(p) = input {
            match p.extension().and_then(|e| e.to_str()).unwrap_or("") {
                "csv" => ',',
                "tsv" | "txt" => '\t',
                _ => sniff_delim(content),
            }
        } else {
            sniff_delim(content)
        };

        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(delim as u8)
            .has_headers(false)
            .flexible(true)
            .trim(csv::Trim::All)
            .from_reader(content.as_bytes());

        let mut all_records: Vec<Vec<String>> = rdr
            .records()
            .filter_map(|r| r.ok())
            .filter(|r| !r.iter().all(|f| f.trim().is_empty()))
            .map(|r| r.iter().map(|f| f.to_string()).collect())
            .collect();

        if all_records.is_empty() {
            return Err("Input is empty".to_string());
        }

        let has_header = if no_header {
            false
        } else {
            all_records[0]
                .first()
                .map(|f| f.parse::<f64>().is_err())
                .unwrap_or(false)
        };

        let (header, rows) = if has_header {
            let h = all_records.remove(0);
            (Some(h), all_records)
        } else {
            (None, all_records)
        };

        Ok(DataTable { header, rows })
    }

    /// Resolve a `ColSpec` to a 0-based column index.
    pub fn resolve(&self, col: &ColSpec) -> Result<usize, String> {
        match col {
            ColSpec::Index(i) => Ok(*i),
            ColSpec::Name(name) => {
                let header = self.header.as_ref().ok_or_else(|| {
                    format!(
                        "Column name '{name}' requested but no header row was detected. \
                             Use --no-header to force treating the first row as data, or \
                             use a 0-based integer index instead."
                    )
                })?;
                header.iter().position(|h| h == name).ok_or_else(|| {
                    format!(
                        "Column '{name}' not found. Available columns: {}",
                        header.join(", ")
                    )
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

    /// Return a human-readable name for a column: the header name when available,
    /// or `"col_N"` for index-based specs with no header.
    pub fn col_display_name(&self, col: &ColSpec) -> String {
        match col {
            ColSpec::Name(n) => n.clone(),
            ColSpec::Index(i) => self
                .header
                .as_ref()
                .and_then(|h| h.get(*i))
                .cloned()
                .unwrap_or_else(|| format!("col_{i}")),
        }
    }

    /// Split the table into groups by the distinct values in `col`.
    ///
    /// Groups are returned in first-seen order.
    pub fn group_by(&self, col: &ColSpec) -> Result<Vec<(String, DataTable)>, String> {
        use std::collections::HashMap;
        let idx = self.resolve(col)?;
        let mut order: Vec<String> = Vec::new();
        let mut map: HashMap<String, Vec<Vec<String>>> = HashMap::new();

        for row in &self.rows {
            let key = row.get(idx).cloned().unwrap_or_default();
            if !map.contains_key(&key) {
                order.push(key.clone());
            }
            map.entry(key).or_default().push(row.clone());
        }

        Ok(order
            .into_iter()
            .map(|name| {
                let rows = map.remove(&name).unwrap();
                (
                    name,
                    DataTable {
                        header: self.header.clone(),
                        rows,
                    },
                )
            })
            .collect())
    }
}

fn sniff_delim(content: &str) -> char {
    let first = content.lines().next().unwrap_or("");
    let tabs = first.chars().filter(|&c| c == '\t').count();
    let commas = first.chars().filter(|&c| c == ',').count();
    if tabs >= commas {
        '\t'
    } else {
        ','
    }
}

// ── Parquet support ───────────────────────────────────────────────────────────

#[cfg(feature = "parquet")]
fn sniff_parquet(buf: &[u8]) -> bool {
    buf.len() >= 8 && &buf[..4] == b"PAR1" && &buf[buf.len() - 4..] == b"PAR1"
}

/// Read a parquet source, decoding only the columns listed in `project`.
///
/// Works for both `std::fs::File` (random-access, enables column-chunk skip)
/// and `bytes::Bytes` (stdin buffer).  An empty `project` slice reads every
/// column.
#[cfg(feature = "parquet")]
fn from_parquet_projected<R>(reader: R, project: &[ColSpec]) -> Result<DataTable, String>
where
    R: parquet::file::reader::ChunkReader + 'static,
{
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    use parquet::arrow::ProjectionMask;

    let builder = ParquetRecordBatchReaderBuilder::try_new(reader)
        .map_err(|e| format!("Cannot open parquet: {e}"))?;

    let arrow_schema = builder.schema().clone();
    let parquet_schema = builder.parquet_schema().clone();
    let fields = arrow_schema.fields();

    let (col_indices, header): (Vec<usize>, Vec<String>) = if project.is_empty() {
        (
            (0..fields.len()).collect(),
            fields.iter().map(|f| f.name().clone()).collect(),
        )
    } else {
        // Resolve specs to (col_index, name), then deduplicate by col_index
        // (first occurrence wins).  Duplicates arise when --x default=Index(0)
        // and --y includes the same column by name; Arrow's ProjectionMask is
        // idempotent but the batch column count would mismatch our header.
        let mut seen = std::collections::HashSet::new();
        let mut indices = Vec::with_capacity(project.len());
        let mut names = Vec::with_capacity(project.len());
        for spec in project {
            let (idx, name) = match spec {
                ColSpec::Index(i) => {
                    if *i >= fields.len() {
                        return Err(format!(
                            "Column index {i} out of range (file has {} columns)",
                            fields.len()
                        ));
                    }
                    (*i, fields[*i].name().clone())
                }
                ColSpec::Name(n) => {
                    let idx = fields.iter().position(|f| f.name() == n).ok_or_else(|| {
                        format!(
                            "Column '{n}' not found in parquet file. Available: {}",
                            fields
                                .iter()
                                .map(|f| f.name().as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    })?;
                    (idx, n.clone())
                }
            };
            if seen.insert(idx) {
                indices.push(idx);
                names.push(name);
            }
        }
        // Sort by physical on-disk column index so the header order matches
        // the order Arrow returns columns in the projected batch.
        // ProjectionMask always delivers columns in on-disk order regardless
        // of the order specs were passed, so without this sort the header and
        // data rows would be misaligned for out-of-order requests.
        let mut pairs: Vec<(usize, String)> = indices.into_iter().zip(names).collect();
        pairs.sort_unstable_by_key(|(i, _)| *i);
        let col_indices: Vec<usize> = pairs.iter().map(|(i, _)| *i).collect();
        let header: Vec<String> = pairs.into_iter().map(|(_, n)| n).collect();
        (col_indices, header)
    };

    let mask = ProjectionMask::roots(&parquet_schema, col_indices);

    let batch_reader = builder
        .with_projection(mask)
        .with_batch_size(65536)
        .build()
        .map_err(|e| format!("Cannot build parquet reader: {e}"))?;

    let mut rows: Vec<Vec<String>> = Vec::new();
    for batch_result in batch_reader {
        let batch = batch_result.map_err(|e| format!("Failed to read parquet batch: {e}"))?;
        let n_rows = batch.num_rows();
        let n_cols = batch.num_columns();
        // Convert each projected column to strings, then transpose into rows.
        let col_strs: Vec<Vec<String>> = (0..n_cols)
            .map(|ci| {
                let col = batch.column(ci);
                (0..n_rows)
                    .map(|ri| arrow_value_to_string(col.as_ref(), ri))
                    .collect()
            })
            .collect();
        for ri in 0..n_rows {
            rows.push(col_strs.iter().map(|c| c[ri].clone()).collect());
        }
    }

    Ok(DataTable {
        header: Some(header),
        rows,
    })
}

#[cfg(feature = "parquet")]
fn arrow_value_to_string(col: &dyn arrow_array::Array, i: usize) -> String {
    use arrow_array::*;
    use arrow_schema::DataType;

    if col.is_null(i) {
        return String::new();
    }

    macro_rules! prim {
        ($T:ty) => {
            col.as_any()
                .downcast_ref::<$T>()
                .map(|a| a.value(i).to_string())
                .unwrap_or_default()
        };
    }

    match col.data_type() {
        DataType::Float32 => prim!(Float32Array),
        DataType::Float64 => prim!(Float64Array),
        DataType::Int8 => prim!(Int8Array),
        DataType::Int16 => prim!(Int16Array),
        DataType::Int32 => prim!(Int32Array),
        DataType::Int64 => prim!(Int64Array),
        DataType::UInt8 => prim!(UInt8Array),
        DataType::UInt16 => prim!(UInt16Array),
        DataType::UInt32 => prim!(UInt32Array),
        DataType::UInt64 => prim!(UInt64Array),
        DataType::Boolean => col
            .as_any()
            .downcast_ref::<BooleanArray>()
            .map(|a| a.value(i).to_string())
            .unwrap_or_default(),
        DataType::Utf8 => col
            .as_any()
            .downcast_ref::<StringArray>()
            .map(|a| a.value(i).to_string())
            .unwrap_or_default(),
        DataType::LargeUtf8 => col
            .as_any()
            .downcast_ref::<LargeStringArray>()
            .map(|a| a.value(i).to_string())
            .unwrap_or_default(),
        DataType::Date32 => col
            .as_any()
            .downcast_ref::<Date32Array>()
            .and_then(|a| {
                let epoch = chrono::NaiveDate::from_ymd_opt(1970, 1, 1)?;
                epoch
                    .checked_add_signed(chrono::Duration::days(a.value(i) as i64))
                    .map(|d| d.to_string())
            })
            .unwrap_or_default(),
        DataType::Date64 => col
            .as_any()
            .downcast_ref::<Date64Array>()
            .and_then(|a| {
                chrono::DateTime::from_timestamp_millis(a.value(i))
                    .map(|dt| dt.date_naive().to_string())
            })
            .unwrap_or_default(),
        DataType::Dictionary(key_dt, _) => arrow_dict_to_string(col, i, key_dt),
        _ => String::new(),
    }
}

#[cfg(feature = "parquet")]
fn arrow_dict_to_string(
    col: &dyn arrow_array::Array,
    i: usize,
    key_dt: &arrow_schema::DataType,
) -> String {
    use arrow_array::*;
    use arrow_schema::DataType;

    macro_rules! dict_str {
        ($K:ty) => {{
            col.as_any()
                .downcast_ref::<DictionaryArray<$K>>()
                .and_then(|dict| {
                    if dict.keys().is_null(i) {
                        return Some(String::new());
                    }
                    let key = dict.keys().value(i) as usize;
                    let values = dict.values();
                    match values.data_type() {
                        DataType::Utf8 => values
                            .as_any()
                            .downcast_ref::<StringArray>()
                            .map(|s| s.value(key).to_string()),
                        DataType::LargeUtf8 => values
                            .as_any()
                            .downcast_ref::<LargeStringArray>()
                            .map(|s| s.value(key).to_string()),
                        _ => Some(arrow_value_to_string(values.as_ref(), key)),
                    }
                })
                .unwrap_or_default()
        }};
    }

    match key_dt {
        DataType::Int8 => dict_str!(types::Int8Type),
        DataType::Int16 => dict_str!(types::Int16Type),
        DataType::Int32 => dict_str!(types::Int32Type),
        DataType::Int64 => dict_str!(types::Int64Type),
        DataType::UInt8 => dict_str!(types::UInt8Type),
        DataType::UInt16 => dict_str!(types::UInt16Type),
        DataType::UInt32 => dict_str!(types::UInt32Type),
        DataType::UInt64 => dict_str!(types::UInt64Type),
        _ => String::new(),
    }
}

// ── Colormap helper (used by heatmap, hexbin, etc.) ──────────────────────────

/// Parse a colormap name string into a `ColorMap` enum.
/// Unrecognized names default to Viridis with a warning on stderr.
///
/// Accepted names (case-insensitive, hyphens or no separator both work):
/// viridis, inferno, magma, plasma, cividis, turbo, warm, cool, cubehelix,
/// blue-green, blue-purple, green-blue, orange-red, purple-blue, purple-blue-green,
/// purple-red, red-purple, yellow-green, yellow-green-blue, yellow-orange-brown,
/// yellow-orange-red, blues, greens, grayscale (grey/gray), oranges, purples, reds,
/// brown-green, pink-green, purple-green, purple-orange, red-blue, red-grey,
/// red-yellow-blue, red-yellow-green, spectral, rainbow, sinebow.
pub fn parse_colormap(name: &str) -> kuva::plot::ColorMap {
    use kuva::plot::ColorMap;
    match name.to_ascii_lowercase().replace('_', "-").as_str() {
        // Sequential perceptual
        "viridis" => ColorMap::Viridis,
        "inferno" => ColorMap::Inferno,
        "magma" => ColorMap::Magma,
        "plasma" => ColorMap::Plasma,
        "cividis" => ColorMap::Cividis,
        "turbo" => ColorMap::Turbo,
        "warm" => ColorMap::Warm,
        "cool" => ColorMap::Cool,
        "cubehelix" => ColorMap::Cubehelix,
        // Sequential ColorBrewer
        "blue-green" | "bluegreen" | "bugn" => ColorMap::BlueGreen,
        "blue-purple" | "bluepurple" | "bupu" => ColorMap::BluePurple,
        "green-blue" | "greenblue" | "gnbu" => ColorMap::GreenBlue,
        "orange-red" | "orangered" | "orrd" => ColorMap::OrangeRed,
        "purple-blue-green" | "purplebluegre" | "pubugn" => ColorMap::PurpleBlueGreen,
        "purple-blue" | "purpleblue" | "pubu" => ColorMap::PurpleBlue,
        "purple-red" | "purplered" | "purd" => ColorMap::PurpleRed,
        "red-purple" | "redpurple" | "rdpu" => ColorMap::RedPurple,
        "yellow-green-blue" | "yellowgreenblue" | "ylgnbu" => ColorMap::YellowGreenBlue,
        "yellow-green" | "yellowgreen" | "ylgn" => ColorMap::YellowGreen,
        "yellow-orange-brown" | "yelloworangebrown" | "ylorb" | "ylorbr" => {
            ColorMap::YellowOrangeBrown
        }
        "yellow-orange-red" | "yelloworangered" | "ylord" | "ylorrd" => ColorMap::YellowOrangeRed,
        // Sequential single-hue
        "blues" => ColorMap::Blues,
        "greens" => ColorMap::Greens,
        "grayscale" | "grey" | "gray" | "greys" | "grays" => ColorMap::Grayscale,
        "oranges" => ColorMap::Oranges,
        "purples" => ColorMap::Purples,
        "reds" => ColorMap::Reds,
        // Diverging
        "brown-green" | "browngreen" | "brbg" => ColorMap::BrownGreen,
        "pink-green" | "pinkgreen" | "piyg" => ColorMap::PinkGreen,
        "purple-green" | "purplegreen" | "prgn" => ColorMap::PurpleGreen,
        "purple-orange" | "purpleorange" | "puor" => ColorMap::PurpleOrange,
        "red-blue" | "redblue" | "rdbu" => ColorMap::RedBlue,
        "red-grey" | "red-gray" | "redgrey" | "redgray" | "rdgy" => ColorMap::RedGrey,
        "red-yellow-blue" | "redyellowblue" | "rdylbu" => ColorMap::RedYellowBlue,
        "red-yellow-green" | "redyellowgreen" | "rdylgn" => ColorMap::RedYellowGreen,
        "spectral" => ColorMap::Spectral,
        // Cyclical
        "rainbow" => ColorMap::Rainbow,
        "sinebow" => ColorMap::Sinebow,
        _ => {
            eprintln!(
                "warning: unknown colormap '{name}', using viridis. \
                Run with --help to see accepted names."
            );
            ColorMap::Viridis
        }
    }
}

#[cfg(all(test, feature = "parquet"))]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// Build a minimal two-column parquet in memory with `b` stored first on
    /// disk and `a` second, then return the raw bytes.
    fn make_parquet_b_then_a(b_val: f64, a_val: f64) -> Vec<u8> {
        use arrow_array::{Float64Array, RecordBatch};
        use arrow_schema::{DataType, Field, Schema};
        use parquet::arrow::ArrowWriter;

        let schema = Arc::new(Schema::new(vec![
            Field::new("b", DataType::Float64, false),
            Field::new("a", DataType::Float64, false),
        ]));
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Float64Array::from(vec![b_val])),
                Arc::new(Float64Array::from(vec![a_val])),
            ],
        )
        .unwrap();
        let mut buf = Vec::new();
        let mut writer = ArrowWriter::try_new(&mut buf, schema, None).unwrap();
        writer.write(&batch).unwrap();
        writer.close().unwrap();
        buf
    }

    /// Regression test: requesting columns in reverse order from their on-disk
    /// layout must not silently swap the data.
    ///
    /// On-disk order: b (index 0), a (index 1).
    /// Requested order: a first, then b.
    /// Before the fix the header was built in request order ["a","b"] but the
    /// batch came back in on-disk order [b_data, a_data], so col_f64("a")
    /// returned b's value and vice-versa.
    #[test]
    fn test_parquet_column_order_not_swapped() {
        let bytes = make_parquet_b_then_a(99.0, 1.0);
        let tmp = std::env::temp_dir().join("kuva_col_order_test.parquet");
        std::fs::write(&tmp, &bytes).unwrap();

        let project = vec![
            ColSpec::Name("a".to_string()),
            ColSpec::Name("b".to_string()),
        ];
        let table = DataTable::parse(Some(&tmp), false, None, &project).unwrap();
        std::fs::remove_file(&tmp).ok();

        let a = table.col_f64(&ColSpec::Name("a".to_string())).unwrap();
        let b = table.col_f64(&ColSpec::Name("b".to_string())).unwrap();

        assert_eq!(
            a,
            vec![1.0],
            "col 'a' returned b's value — on-disk order bug"
        );
        assert_eq!(
            b,
            vec![99.0],
            "col 'b' returned a's value — on-disk order bug"
        );
    }
}
