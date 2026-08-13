use std::io::{self, Read};
use std::path::Path;
use std::str::FromStr;

use clap::Args;

/// A column selector.
///
/// From the CLI, a value may be prefixed to force an interpretation:
/// - `idx:N` / `index:N` / `#N` -> a 0-based index, never reinterpreted (`ForcedIndex`)
/// - `name:X` / `col:X` -> always a column name (`Name`), even if `X` is all-digits
/// - a bare all-digit token -> `Index` (index-first, with the out-of-range name fallback in
///   [`DataTable::resolve`]); a bare non-numeric token -> `Name`.
#[derive(Debug, Clone)]
pub enum ColSpec {
    /// Bare numeric token: a 0-based index that falls back to a header column *named* by that
    /// number when the index is out of range (issue #109).
    Index(usize),
    /// A column name (from `name:`/`col:`, or any non-numeric bare token).
    Name(String),
    /// An explicit 0-based index (from `idx:`/`index:`/`#`): never resolved to a name.
    ForcedIndex(usize),
    /// A 0-based index range (`2..5`, `2..=5`, `3..`, `..4`, `..`). Multi-column: expands
    /// via [`DataTable::expand_columns`]; invalid for single-column arguments.
    Range {
        start: Option<usize>,
        end: Option<usize>,
        inclusive: bool,
    },
    /// A glob over header names (`sum_*`, `*_A`, `y*`, `*mid*`). Multi-column: expands via
    /// [`DataTable::expand_columns`]; invalid for single-column arguments.
    Glob(String),
}

impl FromStr for ColSpec {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Explicit index override: `idx:N`, `index:N`, or `#N`.
        for pfx in ["idx:", "index:", "#"] {
            if let Some(rest) = s.strip_prefix(pfx) {
                let i = rest.parse::<usize>().map_err(|_| {
                    format!("'{s}': '{rest}' after '{pfx}' is not a 0-based column index")
                })?;
                return Ok(ColSpec::ForcedIndex(i));
            }
        }
        // Explicit name override: `name:X` or `col:X` (X may be all-digits or contain `..`/`*`).
        for pfx in ["name:", "col:"] {
            if let Some(rest) = s.strip_prefix(pfx) {
                return Ok(ColSpec::Name(rest.to_string()));
            }
        }
        // Index range: `a..b`, `a..=b`, `a..`, `..b`, `..`.
        if s.contains("..") {
            let (sep, inclusive) = if s.contains("..=") {
                ("..=", true)
            } else {
                ("..", false)
            };
            let (lhs, rhs) = s.split_once(sep).unwrap();
            let parse_end = |t: &str| -> Result<Option<usize>, String> {
                if t.is_empty() {
                    Ok(None)
                } else {
                    t.parse::<usize>()
                        .map(Some)
                        .map_err(|_| format!("'{s}': '{t}' is not a 0-based index in a range"))
                }
            };
            return Ok(ColSpec::Range {
                start: parse_end(lhs)?,
                end: parse_end(rhs)?,
                inclusive,
            });
        }
        // Glob over header names.
        if s.contains('*') {
            return Ok(ColSpec::Glob(s.to_string()));
        }
        // Bare token: numeric -> index (heuristic), else name.
        if let Ok(i) = s.parse::<usize>() {
            Ok(ColSpec::Index(i))
        } else {
            Ok(ColSpec::Name(s.to_string()))
        }
    }
}

impl ColSpec {
    /// True for selectors that expand to (potentially) several columns: ranges and globs.
    /// Used by subcommands to route a single such selector into their multi-column path.
    pub fn is_multi(&self) -> bool {
        matches!(self, ColSpec::Range { .. } | ColSpec::Glob(_))
    }
}

/// Simple glob match: `*` matches any run of characters. Supports prefix (`a*`), suffix (`*a`),
/// contains (`*a*`), and interior (`a*b*c`) patterns. No character classes or escaping.
pub(crate) fn glob_match(pattern: &str, name: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == name;
    }
    if !name.starts_with(parts[0]) {
        return false;
    }
    let mut pos = parts[0].len();
    let last = parts.len() - 1;
    for (i, part) in parts.iter().enumerate() {
        if i == 0 {
            continue;
        }
        if i == last {
            // Final literal must be a suffix of the remaining tail.
            return name[pos..].ends_with(part);
        }
        if part.is_empty() {
            continue;
        }
        match name[pos..].find(part) {
            Some(idx) => pos += idx + part.len(),
            None => return false,
        }
    }
    true
}

#[derive(Args, Debug)]
#[command(next_help_heading = "Input")]
pub struct InputArgs {
    /// Input file (TSV, CSV, or Parquet). Omit or pass "-" to read from stdin.
    pub input: Option<std::path::PathBuf>,

    /// Treat the first row as data even if it looks like a header.
    #[arg(long)]
    pub no_header: bool,

    /// Treat the first row as a header even if it looks like data (e.g. all-numeric
    /// column names). Overrides the auto-detection.
    #[arg(long, conflicts_with = "no_header")]
    pub header: bool,

    /// Override the field delimiter (default: auto-detect from extension or content).
    #[arg(long, short = 'd')]
    pub delimiter: Option<char>,
}

impl InputArgs {
    /// Resolve the `--header` / `--no-header` flags to a `HeaderMode`.
    /// clap guarantees the two flags are mutually exclusive.
    pub fn header_mode(&self) -> HeaderMode {
        if self.header {
            HeaderMode::Header
        } else if self.no_header {
            HeaderMode::NoHeader
        } else {
            HeaderMode::Auto
        }
    }
}

/// How the first row of a CSV/TSV input should be treated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HeaderMode {
    /// Auto-detect whether the first row is a header (the default).
    #[default]
    Auto,
    /// Force the first row to be treated as a header.
    Header,
    /// Force the first row to be treated as data.
    NoHeader,
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
        header: HeaderMode,
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
                    if header != HeaderMode::Auto {
                        eprintln!("warning: --header/--no-header is ignored for parquet input");
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
                Self::parse_str(&content, input, header, delim_override)
            }
            _ => {
                let mut buf = Vec::new();
                io::stdin()
                    .read_to_end(&mut buf)
                    .map_err(|e| format!("Cannot read stdin: {e}"))?;

                #[cfg(feature = "parquet")]
                if sniff_parquet(&buf) {
                    if header != HeaderMode::Auto {
                        eprintln!("warning: --header/--no-header is ignored for parquet input");
                    }
                    if delim_override.is_some() {
                        eprintln!("warning: --delimiter is ignored for parquet input");
                    }
                    return from_parquet_projected(bytes::Bytes::from(buf), project);
                }

                let content =
                    String::from_utf8(buf).map_err(|e| format!("stdin is not valid UTF-8: {e}"))?;
                Self::parse_str(&content, None, header, delim_override)
            }
        }
    }

    pub(crate) fn parse_str(
        content: &str,
        input: Option<&Path>,
        header: HeaderMode,
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

        let has_header = match header {
            HeaderMode::NoHeader => false,
            HeaderMode::Header => true,
            HeaderMode::Auto => detect_header(&all_records),
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
            ColSpec::Index(i) => {
                // A numeric token is normally a 0-based index. But because `--x 2024` parses to
                // `Index(2024)`, an all-numeric column *name* (e.g. a year) could never be selected.
                // So when the index is out of range and the file has a header, fall back to a column
                // literally named by that number. In-range indices are untouched, keeping existing
                // "numeric = index" behaviour fully backward compatible (issue #109).
                if let Some(header) = self.header.as_ref() {
                    if *i >= header.len() {
                        let as_name = i.to_string();
                        if let Some(pos) = header.iter().position(|h| h == &as_name) {
                            return Ok(pos);
                        }
                    }
                }
                Ok(*i)
            }
            // Explicit `idx:`/`#`: a pure index, never reinterpreted as a name.
            ColSpec::ForcedIndex(i) => Ok(*i),
            ColSpec::Name(name) => {
                let header = self.header.as_ref().ok_or_else(|| {
                    format!(
                        "Column name '{name}' requested but no header row was detected. \
                             Use --header to force treating the first row as a header, or \
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
            ColSpec::Range { .. } | ColSpec::Glob(_) => Err(
                "a column range or glob selects multiple columns and cannot be used for a \
                 single-column argument (only for multi-column ones like --y)"
                    .to_string(),
            ),
        }
    }

    /// Number of columns: the header width, or the widest data row.
    fn n_cols(&self) -> usize {
        self.header
            .as_ref()
            .map(|h| h.len())
            .unwrap_or_else(|| self.rows.iter().map(|r| r.len()).max().unwrap_or(0))
    }

    /// Expand a list of selectors (which may include ranges and globs) into concrete per-column
    /// specs, in order, de-duplicated. Single selectors pass through unchanged.
    pub fn expand_columns(&self, cols: &[ColSpec]) -> Result<Vec<ColSpec>, String> {
        let mut out: Vec<ColSpec> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let mut push = |idx: usize, out: &mut Vec<ColSpec>| {
            if seen.insert(idx) {
                out.push(ColSpec::ForcedIndex(idx));
            }
        };
        let ncols = self.n_cols();
        for col in cols {
            match col {
                ColSpec::Range {
                    start,
                    end,
                    inclusive,
                } => {
                    let lo = start.unwrap_or(0);
                    let hi = match end {
                        Some(e) if *inclusive => e + 1,
                        Some(e) => *e,
                        None => ncols,
                    }
                    .min(ncols);
                    for i in lo..hi {
                        push(i, &mut out);
                    }
                }
                ColSpec::Glob(pattern) => {
                    let header = self.header.as_ref().ok_or_else(|| {
                        format!("glob '{pattern}' needs a header row (use --header)")
                    })?;
                    let mut any = false;
                    for (i, name) in header.iter().enumerate() {
                        if glob_match(pattern, name) {
                            push(i, &mut out);
                            any = true;
                        }
                    }
                    if !any {
                        return Err(format!(
                            "glob '{pattern}' matched no columns. Available: {}",
                            header.join(", ")
                        ));
                    }
                }
                // Single selectors resolve now so the result is a flat, concrete list.
                other => push(self.resolve(other)?, &mut out),
            }
        }
        Ok(out)
    }

    /// Extract a column as f64 values.
    pub fn col_f64(&self, col: &ColSpec) -> Result<Vec<f64>, String> {
        let idx = self.resolve(col)?;
        self.rows
            .iter()
            .enumerate()
            .map(|(row_i, row)| {
                let s = row
                    .get(idx)
                    .ok_or_else(|| format!("Row {row_i}: no column at index {idx}"))?;
                let v = s
                    .parse::<f64>()
                    .map_err(|_| format!("Row {row_i}: cannot parse '{s}' as a number"))?;
                // Reject non-finite values (inf / NaN) here rather than letting them silently
                // corrupt axis auto-ranging. Subcommands that read via `col_f64_opt` (scatter,
                // line, histogram) instead treat them as missing / clampable (issue #108).
                if v.is_finite() {
                    Ok(v)
                } else {
                    Err(format!("Row {row_i}: value '{s}' is not finite (inf/NaN)"))
                }
            })
            .collect()
    }

    /// Extract a numeric column, treating empty / NA-token cells as missing (`None`) rather than
    /// erroring. A non-empty cell that is neither an NA token nor a number is still a hard error
    /// (keeps the wrong-column-selection safety net). See issue #108.
    ///
    /// `clamp = (min, max)` clips finite values into the given bounds and, crucially, maps `+inf`
    /// to `max` and `-inf` to `min` (so an infinite value can be capped at a real number rather
    /// than dropped). Any value still non-finite after clamping (e.g. `-inf` with no lower bound,
    /// or `NaN`) is treated as missing, because it is not plottable and would corrupt axis
    /// auto-ranging. Pass `(None, None)` for no clamping (the default: all non-finite → missing).
    pub fn col_f64_opt(
        &self,
        col: &ColSpec,
        na: &NaSet,
        clamp: (Option<f64>, Option<f64>),
    ) -> Result<Vec<Option<f64>>, String> {
        let idx = self.resolve(col)?;
        self.rows
            .iter()
            .enumerate()
            .map(|(row_i, row)| {
                let s = row
                    .get(idx)
                    .ok_or_else(|| format!("Row {row_i}: no column at index {idx}"))?;
                parse_cell_opt(s, na, clamp).map_err(|e| format!("Row {row_i}: {e}"))
            })
            .collect()
    }

    /// Extract a column as f64 Unix timestamps (seconds), parsing each cell with
    /// `format` (a chrono strftime pattern, e.g. `"%Y-%m-%d"` or `"%m/%d/%Y %H:%M"`).
    ///
    /// Tries a full datetime parse first, falling back to a date-only parse at
    /// midnight UTC when `format` has no time component — the same UTC-midnight
    /// convention as `kuva::render::datetime::ymd`.
    pub fn col_date_f64(&self, col: &ColSpec, format: &str) -> Result<Vec<f64>, String> {
        let idx = self.resolve(col)?;
        self.rows
            .iter()
            .enumerate()
            .map(|(row_i, row)| {
                let s = row
                    .get(idx)
                    .ok_or_else(|| format!("Row {row_i}: no column at index {idx}"))?;
                parse_date_to_timestamp(s, format).ok_or_else(|| {
                    format!("Row {row_i}: cannot parse '{s}' as a date/time with format '{format}'")
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
            ColSpec::Index(i) | ColSpec::ForcedIndex(i) => {
                // Route through resolve() so a numeric name that fell back to a header column
                // (e.g. "2024") shows its label rather than "col_2024".
                let idx = self.resolve(col).unwrap_or(*i);
                self.header
                    .as_ref()
                    .and_then(|h| h.get(idx))
                    .cloned()
                    .unwrap_or_else(|| format!("col_{i}"))
            }
            // Multi-column selectors are expanded (via expand_columns) before display; a bare
            // call here is a fallback only.
            ColSpec::Range { .. } => "range".to_string(),
            ColSpec::Glob(p) => p.clone(),
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

/// Parse `s` as a date/time in `format`, returning a Unix timestamp (seconds).
/// Tries a full datetime parse first; falls back to a date-only parse anchored
/// at midnight UTC (so a pure-date format like `"%Y-%m-%d"` works without the
/// caller needing to add a fake time-of-day to the format string).
fn parse_date_to_timestamp(s: &str, format: &str) -> Option<f64> {
    use chrono::{NaiveDate, NaiveDateTime};
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, format) {
        return Some(dt.and_utc().timestamp() as f64);
    }
    let date = NaiveDate::parse_from_str(s, format).ok()?;
    Some(date.and_hms_opt(0, 0, 0)?.and_utc().timestamp() as f64)
}

/// Auto-detect whether the first row of a parsed table is a header.
///
/// Two independent signals, either of which marks row 0 as a header:
///  1. The first cell of row 0 is non-numeric (the long-standing heuristic).
///  2. Some column is non-numeric in row 0 but numeric in every non-empty cell
///     below it, i.e. a text label sitting atop an otherwise numeric column.
///
/// Signal 2 catches cases the first-cell check alone misses, e.g. a leading
/// numeric key column masking a header (`5,data` / `0,1` / `1,2` — issue #111).
/// The rule is a strict superset of the old first-cell check, so it never
/// *removes* a detection the old behaviour made; it only adds new ones.
/// Genuinely ambiguous inputs (all-numeric headers, header-less all-text data)
/// are what the explicit `--header` / `--no-header` flags are for.
fn detect_header(records: &[Vec<String>]) -> bool {
    let first = match records.first() {
        Some(r) => r,
        None => return false,
    };

    // Signal 1: leading cell is non-numeric.
    if first
        .first()
        .map(|f| f.parse::<f64>().is_err())
        .unwrap_or(false)
    {
        return true;
    }

    // Signal 2: a non-numeric label atop an all-numeric column.
    let data = &records[1..];
    for (col, cell) in first.iter().enumerate() {
        if cell.parse::<f64>().is_ok() {
            continue; // numeric header cell carries no signal
        }
        let mut numeric = 0usize;
        let mut nonempty = 0usize;
        for row in data {
            if let Some(v) = row.get(col) {
                let v = v.trim();
                if v.is_empty() {
                    continue;
                }
                nonempty += 1;
                if v.parse::<f64>().is_ok() {
                    numeric += 1;
                }
            }
        }
        if nonempty > 0 && numeric == nonempty {
            return true;
        }
    }

    false
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

    // Ranges/globs need the full schema to expand, so read every column when any is present
    // (the caller expands them against the returned header). Explicit specs still project.
    let read_all = project.is_empty()
        || project
            .iter()
            .any(|s| matches!(s, ColSpec::Range { .. } | ColSpec::Glob(_)));
    let (col_indices, header): (Vec<usize>, Vec<String>) = if read_all {
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
                        // Out of range: fall back to a column literally named by that number
                        // (e.g. a year column "2024"), matching the TSV/CSV path (issue #109).
                        let as_name = i.to_string();
                        match fields.iter().position(|f| f.name() == &as_name) {
                            Some(pos) => (pos, fields[pos].name().clone()),
                            None => {
                                return Err(format!(
                                    "Column index {i} out of range (file has {} columns)",
                                    fields.len()
                                ));
                            }
                        }
                    } else {
                        (*i, fields[*i].name().clone())
                    }
                }
                // Explicit `idx:`/`#`: a pure index, no name fallback.
                ColSpec::ForcedIndex(i) => {
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
                // `read_all` above is true whenever a range/glob is present, so this branch
                // only runs for explicit single specs.
                ColSpec::Range { .. } | ColSpec::Glob(_) => {
                    unreachable!("ranges/globs force read_all")
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
/// The set of cell values treated as "missing" by [`DataTable::col_f64_opt`]. An empty (or
/// whitespace-only) cell is ALWAYS missing; the token set adds named sentinels, matched
/// case-insensitively.
#[derive(Debug, Clone)]
pub struct NaSet {
    tokens: std::collections::HashSet<String>,
}

impl NaSet {
    /// The default NA tokens: `NA`, `NaN`, `null`, `N/A`, `.` (plus empty cells always).
    pub fn default_tokens() -> Self {
        Self::from_tokens(
            ["na", "nan", "null", "n/a", "."]
                .iter()
                .map(|s| s.to_string()),
        )
    }
    pub fn from_tokens(tokens: impl IntoIterator<Item = String>) -> Self {
        Self {
            tokens: tokens
                .into_iter()
                .map(|t| t.trim().to_ascii_lowercase())
                .collect(),
        }
    }
    pub fn is_na(&self, s: &str) -> bool {
        let t = s.trim();
        t.is_empty() || self.tokens.contains(&t.to_ascii_lowercase())
    }
}

/// Parse one cell into an optional number, NA-aware. Empty / NA-token cells become `None`; a
/// parsed value is clamped (`+inf`→max, `-inf`→min, finite outliers clipped) and anything still
/// non-finite becomes `None` (missing). A non-empty, non-NA, non-numeric cell is an error. Shared
/// by [`DataTable::col_f64_opt`] and the row-wise readers (parallel coords).
pub fn parse_cell_opt(
    s: &str,
    na: &NaSet,
    clamp: (Option<f64>, Option<f64>),
) -> Result<Option<f64>, String> {
    if na.is_na(s) {
        return Ok(None);
    }
    match s.parse::<f64>() {
        Ok(mut v) => {
            if let Some(mx) = clamp.1 {
                v = v.min(mx);
            }
            if let Some(mn) = clamp.0 {
                v = v.max(mn);
            }
            Ok(v.is_finite().then_some(v))
        }
        Err(_) => Err(format!("cannot parse '{s}' as a number")),
    }
}

/// How to handle missing values in selected numeric columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NaStrategy {
    /// Drop any row that has a missing value in a selected column (default).
    Drop,
    /// Replace missing values with 0.
    Zero,
    /// Treat any missing value as a hard error (the pre-#108 behaviour).
    Error,
}

/// Resolve a missing-value strategy over a set of aligned numeric columns, returning the row
/// indices to keep. The caller uses these to filter every column (numeric and string) in lockstep,
/// building each numeric column as `col[i].unwrap_or(0.0)` (already `Some` under Drop/Error; filled
/// with 0 under Zero). For `Drop`, prints a one-line note to stderr when rows are removed.
pub fn apply_na(
    strategy: NaStrategy,
    cols: &[&[Option<f64>]],
    n: usize,
) -> Result<Vec<usize>, String> {
    match strategy {
        NaStrategy::Drop => {
            let keep: Vec<usize> = (0..n)
                .filter(|&i| cols.iter().all(|c| c[i].is_some()))
                .collect();
            let dropped = n - keep.len();
            if dropped > 0 {
                eprintln!(
                    "note: dropped {dropped} row(s) with missing or non-finite values \
                     (use --na-strategy zero to keep them as 0, or error to fail)"
                );
            }
            Ok(keep)
        }
        NaStrategy::Zero => Ok((0..n).collect()),
        NaStrategy::Error => {
            for c in cols {
                if let Some(i) = c.iter().position(|v| v.is_none()) {
                    return Err(format!(
                        "Row {i}: missing value (use --na-strategy drop or zero to handle it)"
                    ));
                }
            }
            Ok((0..n).collect())
        }
    }
}

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

#[cfg(test)]
mod na_tests {
    use super::*;

    fn table(content: &str) -> DataTable {
        DataTable::parse_str(content, None, HeaderMode::Header, Some(',')).unwrap()
    }

    #[test]
    fn na_set_matches_empty_and_tokens_case_insensitively() {
        let na = NaSet::default_tokens();
        assert!(na.is_na(""));
        assert!(na.is_na("  "));
        assert!(na.is_na("NA") && na.is_na("na") && na.is_na("nan"));
        assert!(na.is_na("null") && na.is_na("."));
        assert!(!na.is_na("0") && !na.is_na("3.5") && !na.is_na("foo"));
    }

    #[test]
    fn col_f64_opt_marks_missing_but_errors_on_real_garbage() {
        let t = table("x,y\n1,10\n2,\n3,NA\n");
        let ys = t
            .col_f64_opt(&ColSpec::Index(1), &NaSet::default_tokens(), (None, None))
            .unwrap();
        assert_eq!(ys, vec![Some(10.0), None, None]);
        // A non-NA, non-numeric cell is still a hard error.
        let bad = table("x,y\n1,10\n2,foo\n");
        assert!(bad
            .col_f64_opt(&ColSpec::Index(1), &NaSet::default_tokens(), (None, None))
            .is_err());
    }

    #[test]
    fn col_f64_opt_treats_non_finite_as_missing() {
        // inf / -inf / infinity parse to non-finite floats; treat them as missing (not real values)
        // so they can't corrupt axis ranges. (nan is already caught as an NA token.)
        let t = table("x,y\n1,inf\n2,-inf\n3,Infinity\n4,5\n");
        let ys = t
            .col_f64_opt(&ColSpec::Index(1), &NaSet::default_tokens(), (None, None))
            .unwrap();
        assert_eq!(ys, vec![None, None, None, Some(5.0)]);
    }

    #[test]
    fn col_f64_opt_clamp_maps_inf_and_clips_finite() {
        let t = table("x,y\n1,inf\n2,-inf\n3,500\n4,-7\n5,50\n");
        let na = NaSet::default_tokens();
        // clamp [0, 300]: +inf -> 300, -inf -> 0, 500 -> 300, -7 -> 0, 50 unchanged.
        let two_sided = t
            .col_f64_opt(&ColSpec::Index(1), &na, (Some(0.0), Some(300.0)))
            .unwrap();
        assert_eq!(
            two_sided,
            vec![Some(300.0), Some(0.0), Some(300.0), Some(0.0), Some(50.0)]
        );
        // Only a max bound: +inf -> 300, but -inf has no floor so stays non-finite -> missing.
        let max_only = t
            .col_f64_opt(&ColSpec::Index(1), &na, (None, Some(300.0)))
            .unwrap();
        assert_eq!(
            max_only,
            vec![Some(300.0), None, Some(300.0), Some(-7.0), Some(50.0)]
        );
    }

    #[test]
    fn apply_na_drop_keeps_rows_where_all_present() {
        let xs = vec![Some(1.0), None, Some(3.0), Some(4.0)];
        let ys = vec![Some(10.0), Some(20.0), None, Some(40.0)];
        let keep = apply_na(NaStrategy::Drop, &[&xs, &ys], 4).unwrap();
        assert_eq!(keep, vec![0, 3]); // rows 1 (x missing) and 2 (y missing) dropped
    }

    #[test]
    fn apply_na_zero_keeps_all_and_error_fails() {
        let xs = vec![Some(1.0), None];
        assert_eq!(apply_na(NaStrategy::Zero, &[&xs], 2).unwrap(), vec![0, 1]);
        assert!(apply_na(NaStrategy::Error, &[&xs], 2).is_err());
    }
}

#[cfg(test)]
mod header_tests {
    use super::*;

    fn parse(content: &str, mode: HeaderMode) -> DataTable {
        DataTable::parse_str(content, None, mode, Some(',')).unwrap()
    }

    #[test]
    fn auto_detects_header_masked_by_numeric_key_column() {
        // issue #111: leading numeric column hid the header from the old
        // first-cell-only check.
        let t = parse("5,data\n0,1\n1,2\n", HeaderMode::Auto);
        assert_eq!(t.header, Some(vec!["5".into(), "data".into()]));
        assert_eq!(t.rows.len(), 2);
    }

    #[test]
    fn auto_detects_standard_header() {
        let t = parse("x,y\n1,2\n3,4\n", HeaderMode::Auto);
        assert_eq!(t.header, Some(vec!["x".into(), "y".into()]));
        assert_eq!(t.rows.len(), 2);
    }

    #[test]
    fn auto_detects_all_text_header() {
        // First cell non-numeric -> header, unchanged from the old behaviour.
        let t = parse(
            "country,region\nUSA,North\nCanada,North\n",
            HeaderMode::Auto,
        );
        assert_eq!(t.header, Some(vec!["country".into(), "region".into()]));
        assert_eq!(t.rows.len(), 2);
    }

    #[test]
    fn auto_keeps_numeric_headerless_data() {
        let t = parse("0,1\n1,2\n2,3\n", HeaderMode::Auto);
        assert_eq!(t.header, None);
        assert_eq!(t.rows.len(), 3);
    }

    #[test]
    fn auto_keeps_categorical_headerless_data() {
        // A text column that is text all the way down is not a header signal.
        let t = parse("1,foo\n2,bar\n3,baz\n", HeaderMode::Auto);
        assert_eq!(t.header, None);
        assert_eq!(t.rows.len(), 3);
    }

    #[test]
    fn force_header_on_all_numeric_row() {
        let t = parse("2024,2025\n1,2\n3,4\n", HeaderMode::Header);
        assert_eq!(t.header, Some(vec!["2024".into(), "2025".into()]));
        assert_eq!(t.rows.len(), 2);
    }

    #[test]
    fn force_no_header_on_header_looking_row() {
        let t = parse("x,y\n1,2\n", HeaderMode::NoHeader);
        assert_eq!(t.header, None);
        assert_eq!(t.rows.len(), 2);
    }

    #[test]
    fn single_row_numeric_is_data() {
        let t = parse("5,data\n", HeaderMode::Auto);
        // No data rows to compare against and the first cell is numeric, so the
        // lone row stays as data.
        assert_eq!(t.header, None);
        assert_eq!(t.rows.len(), 1);
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
        let table = DataTable::parse(Some(&tmp), HeaderMode::Auto, None, &project).unwrap();
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

// ── Issue #109: all-numeric column names ────────────────────────────────────────
// `--x 2024` parses to `ColSpec::Index(2024)`; when that index is out of range and a
// header column is literally named "2024", resolve to that column. Not parquet-gated,
// so this runs on every test build.
#[cfg(test)]
mod colspec_tests {
    use super::*;

    fn headered(content: &str) -> DataTable {
        DataTable::parse_str(content, None, HeaderMode::Header, Some(',')).unwrap()
    }

    #[test]
    fn numeric_name_out_of_range_resolves_to_header_column() {
        let t = headered("gene,2023,2024\nBRCA,5,9\nTP53,6,10\n");
        // Index 2024 is way out of range -> falls back to the column named "2024".
        let idx = t.resolve(&ColSpec::Index(2024)).unwrap();
        assert_eq!(idx, 2);
        assert_eq!(t.col_f64(&ColSpec::Index(2024)).unwrap(), vec![9.0, 10.0]);
        assert_eq!(t.col_f64(&ColSpec::Index(2023)).unwrap(), vec![5.0, 6.0]);
        // Display name reflects the header label, not "col_2024".
        assert_eq!(t.col_display_name(&ColSpec::Index(2024)), "2024");
    }

    #[test]
    fn in_range_index_is_untouched_by_numeric_name_fallback() {
        // Header cells are themselves numbers, but an in-range index must still mean the index.
        let t = headered("10,11,12\n1,2,3\n");
        assert_eq!(t.resolve(&ColSpec::Index(0)).unwrap(), 0); // NOT a name lookup
        assert_eq!(t.col_f64(&ColSpec::Index(0)).unwrap(), vec![1.0]);
        // Index 1 is in range -> stays index 1 (the "11" column), not a name lookup.
        assert_eq!(t.resolve(&ColSpec::Index(1)).unwrap(), 1);
        assert_eq!(t.col_f64(&ColSpec::Index(1)).unwrap(), vec![2.0]);
    }

    #[test]
    fn out_of_range_numeric_with_no_matching_name_still_errors() {
        let t = headered("gene,value\nBRCA,5\n");
        // No column named "7" and index 7 is out of range -> the later lookup errors.
        assert!(t.col_f64(&ColSpec::Index(7)).is_err());
    }

    #[test]
    fn colspec_parses_all_digit_string_as_index() {
        // Documents the parse behaviour the resolve() fallback compensates for.
        assert!(matches!(
            "2024".parse::<ColSpec>(),
            Ok(ColSpec::Index(2024))
        ));
        assert!(matches!("gene".parse::<ColSpec>(), Ok(ColSpec::Name(_))));
    }

    // ── Explicit prefixes: name:/col: force a name, idx:/index:/# force an index ─────

    #[test]
    fn prefixes_parse_to_the_right_variant() {
        assert!(matches!(
            "idx:2".parse::<ColSpec>(),
            Ok(ColSpec::ForcedIndex(2))
        ));
        assert!(matches!(
            "index:5".parse::<ColSpec>(),
            Ok(ColSpec::ForcedIndex(5))
        ));
        assert!(matches!(
            "#7".parse::<ColSpec>(),
            Ok(ColSpec::ForcedIndex(7))
        ));
        // name:/col: force a Name even for an all-digit body.
        assert!(matches!(
            "name:2024".parse::<ColSpec>(),
            Ok(ColSpec::Name(n)) if n == "2024"
        ));
        assert!(matches!(
            "col:1".parse::<ColSpec>(),
            Ok(ColSpec::Name(n)) if n == "1"
        ));
        // A non-numeric body after idx: is an error.
        assert!("idx:abc".parse::<ColSpec>().is_err());
        // name: can carry a literal name that itself contains a colon.
        assert!(matches!(
            "name:idx:2".parse::<ColSpec>(),
            Ok(ColSpec::Name(n)) if n == "idx:2"
        ));
    }

    #[test]
    fn forced_name_selects_a_small_in_range_numeric_column() {
        // Columns literally named "30","10","20" at indices 0,1,2. The whole point of the
        // escape hatch: select the column NAMED "10" (index 1), not index 10.
        let t = headered("30,10,20\n1,7,4\n2,8,5\n");
        let name10 = "name:10".parse::<ColSpec>().unwrap();
        assert_eq!(t.resolve(&name10).unwrap(), 1);
        assert_eq!(t.col_f64(&name10).unwrap(), vec![7.0, 8.0]);
        assert_eq!(t.col_display_name(&name10), "10");
    }

    #[test]
    fn forced_index_is_pure_and_never_falls_back_to_a_name() {
        let t = headered("gene,2024\nBRCA,9\n");
        // Bare 2024 -> heuristic -> the "2024" column (index 1).
        assert_eq!(t.resolve(&ColSpec::Index(2024)).unwrap(), 1);
        // idx:2024 -> pure index 2024 -> out of range -> hard error, no name rescue.
        let forced = "idx:2024".parse::<ColSpec>().unwrap();
        assert_eq!(t.resolve(&forced).unwrap(), 2024);
        assert!(t.col_f64(&forced).is_err());
    }

    // ── Ranges and globs (issue #109) ───────────────────────────────────────

    fn idxs(t: &DataTable, spec: &str) -> Vec<usize> {
        let cs: Vec<ColSpec> = spec.split(' ').map(|s| s.parse().unwrap()).collect();
        t.expand_columns(&cs)
            .unwrap()
            .iter()
            .map(|c| t.resolve(c).unwrap())
            .collect()
    }

    #[test]
    fn range_parses_all_forms() {
        assert!(matches!(
            "2..5".parse::<ColSpec>(),
            Ok(ColSpec::Range {
                start: Some(2),
                end: Some(5),
                inclusive: false
            })
        ));
        assert!(matches!(
            "2..=5".parse::<ColSpec>(),
            Ok(ColSpec::Range {
                start: Some(2),
                end: Some(5),
                inclusive: true
            })
        ));
        assert!(matches!(
            "3..".parse::<ColSpec>(),
            Ok(ColSpec::Range {
                start: Some(3),
                end: None,
                ..
            })
        ));
        assert!(matches!(
            "..4".parse::<ColSpec>(),
            Ok(ColSpec::Range {
                start: None,
                end: Some(4),
                ..
            })
        ));
        assert!(matches!("y*".parse::<ColSpec>(), Ok(ColSpec::Glob(_))));
    }

    #[test]
    fn expand_ranges() {
        let t = headered("a,b,c,d,e\n1,2,3,4,5\n");
        assert_eq!(idxs(&t, "1..4"), vec![1, 2, 3]); // exclusive
        assert_eq!(idxs(&t, "1..=3"), vec![1, 2, 3]); // inclusive
        assert_eq!(idxs(&t, "3.."), vec![3, 4]); // to last
        assert_eq!(idxs(&t, "..2"), vec![0, 1]); // from start
        assert_eq!(idxs(&t, ".."), vec![0, 1, 2, 3, 4]); // all
                                                         // Out-of-range end is clamped to the column count.
        assert_eq!(idxs(&t, "3..99"), vec![3, 4]);
    }

    #[test]
    fn expand_globs() {
        let t = headered("cat,sum_A,sum_B,y1,y2,tot_A\n0,1,2,3,4,5\n");
        assert_eq!(idxs(&t, "sum_*"), vec![1, 2]); // prefix
        assert_eq!(idxs(&t, "*_A"), vec![1, 5]); // suffix
        assert_eq!(idxs(&t, "y*"), vec![3, 4]); // prefix
    }

    #[test]
    fn expand_mixes_and_dedupes() {
        let t = headered("a,b,c,d\n1,2,3,4\n");
        // Single + range with an overlap: b appears once.
        assert_eq!(idxs(&t, "b 1..3"), vec![1, 2]);
    }

    #[test]
    fn glob_with_no_match_errors() {
        let t = headered("a,b,c\n1,2,3\n");
        let cs = vec!["zzz*".parse::<ColSpec>().unwrap()];
        assert!(t.expand_columns(&cs).is_err());
    }

    #[test]
    fn range_glob_invalid_for_single_column() {
        let t = headered("a,b,c\n1,2,3\n");
        assert!(t.resolve(&"1..3".parse::<ColSpec>().unwrap()).is_err());
        assert!(t.resolve(&"a*".parse::<ColSpec>().unwrap()).is_err());
    }

    #[test]
    fn glob_match_forms() {
        assert!(glob_match("sum_*", "sum_A"));
        assert!(!glob_match("sum_*", "tot_A"));
        assert!(glob_match("*_A", "sum_A"));
        assert!(glob_match("*sum*", "x_sum_y"));
        assert!(glob_match("a*c", "abc"));
        assert!(glob_match("a*c", "axxxc"));
        assert!(!glob_match("a*c", "abd"));
        assert!(glob_match("exact", "exact"));
        assert!(!glob_match("exact", "exacts"));
    }
}
