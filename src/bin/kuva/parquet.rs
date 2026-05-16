use arrow::array::{Array, Float64Array, RecordBatchReader};
use arrow::compute::cast;
use arrow::datatypes::DataType;
use bytes::Bytes;
use parquet::arrow::ProjectionMask;
use parquet::schema::types::SchemaDescriptor;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use crate::data::{ColSpec, InputArgs};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

pub struct ParquetScatterSource {
    // While x axis could be a non-numeric type, in bin/kuva/scatter.rs,
    // CLI Scatter only supports numeric X axis anyways
    pub x_col: Vec<f64>,
    pub y_cols: Vec<Vec<f64>>,
}

impl ParquetScatterSource {
    pub fn parse(
        // Copying input: Option<&Path> from data.rs but wondering if this needs to be
        // an Option<>? In `data.rs` the None arm reads from stdin
        //
        // It also seems like an unrecoverable error to attempt to construct
        // a plot without identifying the data from which you want to construct
        // the plot, so not allowing unidentified input seems reasonable.
        //
        // Important: to enter this function, the data should have already been validated to be parquet
        // @dev TODO: how to validate that stdin is parquet. ===
        input: Option<&Path>,
        x_col: ColSpec,
        y_cols: Vec<ColSpec>,
    ) -> Result<Self, String> {
        // we need different branches for stdin, which uses a Builder<_<bytes>>,
        // and from file, which uses a Builder<_<file>>
        let mut requested_leafs: Vec<usize> = Vec::new();

        let reader = match input {
            Some(p) if p.to_str() != Some("-") => {
                let file = File::open(p)
                    .map_err(|e| format!("Failed to open the provided file. {}", e))?;
                let builder = ParquetRecordBatchReaderBuilder::try_new(file)
                    .map_err(|e| format!("Failed to read from the provided parquet file. {}", e))?;

                let mask = {
                    let schema = builder.parquet_schema();

                    let x_leaf_index = resolve_colspec(&x_col, schema)?;
                    let y_leaf_indices = y_cols
                        .iter()
                        .map(|col| resolve_colspec(col, schema))
                        .collect::<Result<Vec<usize>, String>>()?;

                    requested_leafs.push(x_leaf_index);
                    requested_leafs.extend(y_leaf_indices.iter().copied());

                    ProjectionMask::leaves(
                        schema,
                        std::iter::once(x_leaf_index).chain(y_leaf_indices.iter().copied()),
                    )
                };
                let reader = builder
                    .with_projection(mask)
                    .build()
                    .map_err(|e| format!("Failed to construct parquet reader. {}", e))?;

                reader
            }
            _ => {
                let mut buf = Vec::new();
                io::stdin()
                    .read_to_end(&mut buf)
                    .map_err(|e| format!("Cannot read stdin: {e}"))?;
                let builder = ParquetRecordBatchReaderBuilder::try_new(Bytes::from(buf))
                    .map_err(|e| format!("Cannot read parquet from stdin. {}", e))?;
                let mask = {
                    let schema = builder.parquet_schema();

                    let x_leaf_index = resolve_colspec(&x_col, schema)?;
                    let y_leaf_indices = y_cols
                        .iter()
                        .map(|col| resolve_colspec(col, schema))
                        .collect::<Result<Vec<usize>, String>>()?;

                    requested_leafs.push(x_leaf_index);
                    requested_leafs.extend(y_leaf_indices.iter().copied());

                    ProjectionMask::leaves(
                        schema,
                        std::iter::once(x_leaf_index).chain(y_leaf_indices.iter().copied()),
                    )
                };
                let reader = builder
                    .with_projection(mask)
                    .build()
                    .map_err(|e| format!("Failed to construct parquet reader. {}", e))?;

                reader
            }
        };

        // Design decision: can only support logical numeric datatypes for now
        let schema = reader.schema();
        for field in schema.fields() {
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
                    return Err(format!("Unsupported datatype: {:?}", field.data_type()));
                }
            }
        }

        let mut x_vec: Vec<f64> = Vec::new();
        let mut y_vec: Vec<Vec<f64>> = vec![Vec::new(); y_cols.len()];
        let projected_cols = map_requested_leaves(&requested_leafs);

        for batch in reader {
            let batch = batch.map_err(|e| format!("Failed to read parquet batch: {e}"))?;

            let mut casted_cols: Vec<Float64Array> = Vec::new();
            for col in batch.columns() {
                let casted = cast(col, &DataType::Float64)
                    .map_err(|e| format!("Failed to cast column to Float64: {e}"))?;
                let f64_casted = casted
                    .as_any()
                    .downcast_ref::<Float64Array>()
                    .ok_or_else(|| format!("Failed to cast column to a Float64 array"))?;
                // 5/16: @dev TODO: codex recommending a cleaner ownership story to avoid this clone()
                casted_cols.push(f64_casted.clone());
            }

            for row_idx in 0..batch.num_rows() {
                // Null policy: if any column in a row has a null value, skip the entire row.
                if casted_cols.iter().any(|col| col.is_null(row_idx)) {
                    continue;
                }

                // projected_cols is ordered as x_col -> y_cols
                // per map_requested_leaves(), its values are the index of casted_cols that correspond to the user-defined column
                x_vec.push(casted_cols[projected_cols[0]].value(row_idx));
                for (y_idx, col_idx) in projected_cols.iter().enumerate().skip(1) {
                    y_vec[y_idx - 1].push(casted_cols[*col_idx].value(row_idx));
                }
            }
        }

        Ok(ParquetScatterSource {
            x_col: x_vec,
            y_cols: y_vec,
        })
    }
}

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

// When we build our parquet projection, the columns will be ordered according to the parquet storage.
//
// We need to map this parquet projection to the user-defined x and y columns
fn map_requested_leaves(requested_leafs: &[usize]) -> Vec<usize> {
    let mut ordered_leaves = requested_leafs.to_vec();

    ordered_leaves.sort();
    ordered_leaves.dedup();
    // at this point, `ordered_leaves` represents the columns of the reader.schema, by underlying leaf index

    let mut projected_positions: Vec<usize> = Vec::new();

    // go through all requested_leaves.
    // for each leaf, find the column number that matches the requested leaf
    for leaf in requested_leafs {
        let col_idx = ordered_leaves
            .iter()
            .position(|ordered_leaf| ordered_leaf == leaf)
            .expect("ordered_leaves was built from requested_leafs, so values must match.");

        // once we have the column number, we push it onto a new vec
        projected_positions.push(col_idx);

        // this vec is now a list of column numbers.
        // it is ordered in the same order as the user's requested leafs and has the same length.
        // now we can use this by, per-row, indexing into the columns of the batch and getting the value

        // assert_eq!(leafs_to_cols.clone().len(), requested_leafs.clone().len());
    }

    projected_positions
}
