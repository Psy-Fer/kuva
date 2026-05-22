use arrow::array::{Array, ArrayRef, Float64Array, RecordBatchReader, StringArray};
use arrow::compute::cast;
use arrow::datatypes::DataType;
use arrow::util::display::{ArrayFormatter, FormatOptions};
use bytes::Bytes;
use parquet::arrow::ProjectionMask;
use parquet::schema::types::SchemaDescriptor;
use std::fs::File;
use std::path::Path;

use crate::data::ColSpec;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

#[derive(Clone)]
pub struct ParquetScatterSource {
    pub x_col: Vec<f64>,
    pub y_cols: Vec<Vec<f64>>,
    pub group_by: Option<Vec<String>>,
    /// Ordered as x_col + y_cols + group_by
    pub column_names: Vec<String>,
}

impl ParquetScatterSource {
    /// Construct a ParquetScatterSource from a byte stream
    pub fn from_bytes(
        bytes: Bytes,
        x_col: &ColSpec,
        y_cols: &[ColSpec],
        gb_col: &Option<ColSpec>,
    ) -> Result<Self, String> {
        let builder = ParquetRecordBatchReaderBuilder::try_new(bytes)
            .map_err(|e| format!("Cannot read parquet from stdin. {}", e))?;
        return Self::parse_builder(builder, x_col, y_cols, gb_col);
    }

    /// Construct a ParquetScatterSource from a file path
    pub fn from_path(
        input: &Path,
        x_col: &ColSpec,
        y_cols: &[ColSpec],
        gb_col: &Option<ColSpec>,
    ) -> Result<Self, String> {
        let file =
            File::open(input).map_err(|e| format!("Failed to open the provided file. {}", e))?;
        let builder = ParquetRecordBatchReaderBuilder::try_new(file)
            .map_err(|e| format!("Failed to read from the provided parquet file. {}", e))?;

        return Self::parse_builder(builder, x_col, y_cols, gb_col);
    }

    /// Return a set of distinct ParquetScatterSource grouped by the unique values of the group_by column
    pub fn group_by(&self) -> Result<Vec<(String, ParquetScatterSource)>, String> {
        let gb_col: &Vec<String> = self
            .group_by
            .as_ref()
            .ok_or_else(|| "trying to group when the group_by column is not set.")?;

        let mut gb_vals: Vec<String> = Vec::new();

        for val in gb_col {
            if !gb_vals.contains(val) {
                gb_vals.push(val.clone());
            }
        }

        let mut groups: Vec<ParquetScatterSource> = vec![
            ParquetScatterSource {
                x_col: Vec::new(),
                y_cols: vec![Vec::new(); self.y_cols.len()],
                group_by: None,
                column_names: self.column_names.clone(),
            };
            gb_vals.len()
        ];

        for row in 0..self.x_col.len() {
            let pos = gb_vals
                .iter()
                .position(|val| {
                    val == gb_col
                        .get(row)
                        .expect("gb_col should be the same length as x_col.")
                })
                .ok_or_else(|| {
                    "Couldn't find a match in gb_vals, which should be impossible".to_string()
                })?;

            let y_vals: Vec<f64> = self.y_cols.iter().map(|y_col| y_col[row]).collect();
            groups[pos].push(self.x_col[row], y_vals, None)?;
        }

        return Ok(gb_vals.into_iter().zip(groups.into_iter()).collect());
    }

    /// Build a ParquetScatterSource from a parquet builder
    fn parse_builder<T: ::parquet::file::reader::ChunkReader + 'static>(
        builder: ParquetRecordBatchReaderBuilder<T>,
        x_col: &ColSpec,
        y_cols: &[ColSpec],
        gb_col: &Option<ColSpec>,
    ) -> Result<Self, String> {
        let schema = builder.parquet_schema();

        let x_leaf_index = resolve_colspec(&x_col, schema)?;
        let y_leaf_indices = y_cols
            .iter()
            .map(|col| resolve_colspec(col, schema))
            .collect::<Result<Vec<usize>, String>>()?;
        let group_by_index = if let Some(col) = gb_col {
            Some(resolve_colspec(col, schema)?)
        } else {
            None
        };

        let requested_leafs: Vec<usize> = std::iter::once(x_leaf_index)
            .chain(y_leaf_indices.iter().copied())
            .chain(group_by_index.iter().copied())
            .collect();

        let gb_is_also_data = if let Some(idx) = group_by_index {
            x_leaf_index == idx || y_leaf_indices.iter().any(|i| i == &idx)
        } else {
            false
        };

        let mask = ProjectionMask::leaves(schema, requested_leafs.clone());
        let reader = builder
            .with_projection(mask)
            .build()
            .map_err(|e| format!("Failed to construct parquet reader. {}", e))?;

        let projected_cols = map_requested_leaves(&requested_leafs);

        let schema = reader.schema();
        let mut reader = reader.peekable();

        for (i, field) in schema.fields().iter().enumerate() {
            // Design decision: x/y can only support numeric datatypes for now.
            // Matches existing functionality
            match field.data_type() {
                DataType::Int8
                | DataType::Int16
                | DataType::Int32
                | DataType::Int64
                | DataType::UInt8
                | DataType::UInt16
                | DataType::UInt32
                | DataType::UInt64
                | DataType::Float16
                | DataType::Float32
                | DataType::Float64 => {}
                _ => {
                    // if this column is ONLY used as the group_by column then
                    // we can support anything that can be cast to a String
                    if gb_col.is_some()
                        && i == projected_cols[projected_cols.len() - 1]
                        && !gb_is_also_data
                    {
                        let first_batch = reader
                            .peek()
                            .ok_or_else(|| format!("Parquet has no batches."))?
                            .as_ref()
                            .map_err(|e| format!("Failed to read parquet batch. {}", e))?;

                        let try_group_by_col =
                            first_batch.column(projected_cols[projected_cols.len() - 1]);
                        let Ok(_) =
                            ArrayFormatter::try_new(try_group_by_col, &FormatOptions::default())
                        else {
                            return Err(format!(
                                "Unsupported datatype for field {}: {:?}",
                                field.name(),
                                field.data_type()
                            ));
                        };
                    } else {
                        return Err(format!(
                            "Unsupported datatype for field {}: {:?}",
                            field.name(),
                            field.data_type()
                        ));
                    }
                }
            }
        }

        let gb_projected_idx = if gb_col.is_some() {
            Some(*projected_cols.last().expect(
                "we must have at least one column here, or else we've been processing zero data.",
            ))
        } else {
            None
        };

        let mut x_vec: Vec<f64> = Vec::new();
        let mut y_vec: Vec<Vec<f64>> = vec![Vec::new(); y_cols.len()];
        let mut gb_vec: Option<Vec<String>> = if gb_col.is_some() {
            Some(Vec::new())
        } else {
            None
        };

        for batch in reader {
            let batch = batch.map_err(|e| format!("Failed to read parquet batch: {e}"))?;

            let mut casted_cols: Vec<ArrayRef> = Vec::new();

            for (i, col) in batch.columns().iter().enumerate() {
                let target_type = if Some(i) == gb_projected_idx && !gb_is_also_data {
                    DataType::Utf8
                } else {
                    DataType::Float64
                };

                let casted = cast(col, &target_type)
                    .map_err(|e| format!("Failed to cast projected column {i}: {e}"))?;
                casted_cols.push(casted);
            }

            let mut numeric_cols: Vec<&Float64Array> = Vec::new();
            // to handle gb_col.is_some() && gb_is_also_data
            let mut gb_numeric_index: Option<usize> = None;
            // to handle gb_col.is_some() && !gb_is_also_data
            let mut gb_array: Option<&StringArray> = None;

            // order numeric_cols as x + ys
            for (i, proj_idx) in projected_cols.iter().enumerate() {
                if Some(*proj_idx) == gb_projected_idx && !gb_is_also_data {
                    assert_eq!(i, projected_cols.len() - 1);

                    let str_col = casted_cols[*proj_idx]
                        .as_any()
                        .downcast_ref::<StringArray>()
                        .ok_or_else(|| {
                            format!("Failed to downcast group_by column to StringArray")
                        })?;
                    gb_array = Some(str_col);
                } else if Some(*proj_idx) == gb_projected_idx
                    && gb_is_also_data
                    && i == projected_cols.len() - 1
                {
                    continue;
                } else {
                    let num_col = casted_cols[*proj_idx]
                        .as_any()
                        .downcast_ref::<Float64Array>()
                        .ok_or_else(|| {
                            format!("Failed to downcast numeric column to Float64Array.")
                        })?;
                    numeric_cols.push(num_col);

                    if Some(*proj_idx) == gb_projected_idx && gb_is_also_data {
                        gb_numeric_index = Some(numeric_cols.len() - 1);
                    }
                }
            }

            for row_idx in 0..batch.num_rows() {
                // Null policy: if any column in a row has a null value, skip the entire row.
                if numeric_cols.iter().any(|col| col.is_null(row_idx)) {
                    continue;
                }
                if let Some(gb_array) = gb_array {
                    if gb_array.is_null(row_idx) {
                        continue;
                    }
                }

                x_vec.push(numeric_cols[0].value(row_idx));

                for (i, col) in numeric_cols.iter().skip(1).enumerate() {
                    y_vec[i].push(col.value(row_idx));
                }

                if let Some(gb_idx) = gb_numeric_index {
                    gb_vec
                        .as_mut()
                        .expect("if gb_numeric_index.is_some() then gb_vec must also be Some")
                        .push(numeric_cols[gb_idx].value(row_idx).to_string());
                } else if let Some(gb_array) = gb_array {
                    gb_vec
                        .as_mut()
                        .expect("if gb_array.is_some() then gb_vec must also be Some")
                        .push(gb_array.value(row_idx).to_string());
                }
            }
        }

        let column_names: Vec<String> = projected_cols
            .iter()
            .map(|col_no| schema.field(*col_no).name().clone())
            .collect();

        Ok(ParquetScatterSource {
            x_col: x_vec,
            y_cols: y_vec,
            group_by: gb_vec,
            column_names,
        })
    }

    /// Push a single set of values onto a ParquetScatterSource
    fn push(
        &mut self,
        x_val: f64,
        y_vals: Vec<f64>,
        group_by: Option<String>,
    ) -> Result<&mut Self, String> {
        if y_vals.len() != self.y_cols.len() {
            return Err("length of y_col parameter does not match length of y_cols. Ensure you're pushing one value per y column".to_string());
        }

        if group_by.is_some() && self.group_by.is_none() {
            return Err(
                "Attempting to add a group_by value when the data does not have a group_by column"
                    .to_string(),
            );
        } else if group_by.is_none() && self.group_by.is_some() {
            return Err(
                "You must include a group_by value to add because the underlying data has a group_by column"
                    .to_string(),
            );
        } else if group_by.is_some() && self.group_by.is_some() {
            self.group_by.as_mut().unwrap().push(group_by.unwrap());
        }

        self.x_col.push(x_val);
        for (i, col) in self.y_cols.iter_mut().enumerate() {
            col.push(y_vals[i]);
        }

        return Ok(self);
    }
}

/// Helper to quickly identify if stdin input is likely to be valid parquet
pub fn sniff_parquet(buf: &[u8]) -> bool {
    buf.len() >= 8 && &buf[..4] == b"PAR1" && &buf[buf.len() - 4..] == b"PAR1"
}

/// Returns the index of the parquet schema that matches the provided ColSpec
fn resolve_colspec(col: &ColSpec, schema: &SchemaDescriptor) -> Result<usize, String> {
    let leaf_index = match col {
        ColSpec::Index(i) => {
            let all_roots = schema.root_schema().get_fields();

            if *i >= all_roots.len() {
                return Err("X column index exceed total number of columns in parquet file. Make sure you are zero-indexing. Indices only support unnested columns.".to_string());
            }

            let matching_leaf_indices: Vec<usize> = (0..schema.num_columns())
                .filter(|leaf_idx| schema.get_column_root_idx(*leaf_idx) == *i)
                .collect();

            match matching_leaf_indices.len() {
                        0 => return Err("No matching leaf index found for column index".to_string()),
                        1 => matching_leaf_indices[0],
                        _ => return Err("Provided column index matches to a nested column, which is not currently supported. Try specifying a single leaf within the nested column by name".to_string()),
                    }
        }
        ColSpec::Name(col_name) => {
            let matching_leaf_indices: Vec<usize> = schema
                .columns()
                .iter()
                .enumerate()
                .filter(|(_, col)| col_name == col.name())
                .map(|(i, _)| i)
                .collect();

            match matching_leaf_indices.len() {
                        0 => return Err(format!("Couldn't find a column named {}", col_name)),
                        1 => matching_leaf_indices[0],
                       _ => return Err(format!("Column name {} is ambiguous within this parquet schema. Nested fields with the same column name are not yet supported.", col_name)),
                    }
        }
    };

    Ok(leaf_index)
}

/// Map requested parquet leaf indices into positions within the projected reader schema.
///
/// `requested_leafs` is ordered by the user's logical request:
/// `x_col + y_cols + group_by`.
/// Parquet supports column selection ("projection"), which is efficient. After projection,
/// the parquet reader returns the selected columns in parquet leaf order rather than the
/// user's requested order.
///
/// This function returns, for each requested leaf, its column position in the
/// projected reader so Kuva can maintain the original user-defined ordering.
///
/// Example:
///
/// Full parquet leaf indices:  0  1  2  3  4  5
/// User request (`--y 3,1`):   3  1
/// requested_leafs:           <3, 1>
/// Projected reader columns:   1  3
/// Returned mapping:           1  0
///
/// Here, requested leaf `3` is column 1 in the projected reader, and requested
/// leaf `1` is column 0.
fn map_requested_leaves(requested_leafs: &[usize]) -> Vec<usize> {
    let mut ordered_leaves = requested_leafs.to_vec();
    ordered_leaves.sort();
    ordered_leaves.dedup();

    let mut projected_positions: Vec<usize> = Vec::new();

    for leaf in requested_leafs {
        let col_idx = ordered_leaves
            .iter()
            .position(|ordered_leaf| ordered_leaf == leaf)
            .expect("ordered_leaves was built from requested_leafs, so values must match.");

        projected_positions.push(col_idx);
    }

    projected_positions
}
