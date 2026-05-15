use arrow::array::{RecordBatch, RecordBatchReader};
use arrow::compute::cast;
use arrow::datatypes::DataType;
use bytes::Bytes;
use parquet::arrow::ProjectionMask;
use parquet::schema::types::{SchemaDescriptor, Type};
use std::fs::File;
use std::io::{self, Cursor, Read};
use std::ops::Deref;
use std::path::Path;

use crate::data::{ColSpec, InputArgs};
use arrow::record_batch;
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
        // we need different branches for stdin, which uses a SyncReader<bytes>, and
        // from file, which uses a SyncReader<file>
        match input {
            Some(p) if p.to_str() != Some("-") => {
                let file = File::open(input.unwrap())
                    .map_err(|e| format!("Failed to open the provided file. {}", e))?;
                let mut builder = ParquetRecordBatchReaderBuilder::try_new(file)
                    .map_err(|e| format!("Failed to read from the provided parquet file. {}", e))?;

                // Lazily identify only the columns we want
                let mask = {
                    let schema = builder.parquet_schema();

                    let x_leaf_idx: usize = resolve_colspec(&x_col, schema)?;
                    let y_leaf_idxs: Vec<usize> = y_cols
                        .iter()
                        .map(|y| resolve_colspec(y, schema))
                        .collect::<Result<Vec<_>, _>>()?;
                    let all_leaves: Vec<usize> = std::iter::once(x_leaf_idx)
                        .chain(y_leaf_idxs.into_iter())
                        .collect();

                    ProjectionMask::leaves(schema, all_leaves)
                };

                let reader = builder
                    .with_projection(mask)
                    .build()
                    .map_err(|e| format!("Failed to construct parquet reader. {}", e))?;

                // Check that each column is a supported (i.e., numeric) datatype
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

                // Parquets are read in batches of row, so one batch is a 2d data shape
                // where the columns are defined by the projection mask
                for batch in reader {
                    let batch = batch.map_err(|e| format!("Failed to read parquet batch: {e}"))?;
                    let x_col = batch.column(0);
                    let casted = cast(x_col, &DataType::Float64)
                        .map_err(|e| format!("Failed to cast column to Float64. {}", e))?;
                    // you are here: 5/15
                    // let float64 = casted.as_any().downcast_ref::<Float64Array>().ok_or(...)?;
                    // let values: Vec<f64> = ...

                    for y_col in batch.columns().iter() {
                        let values = y_col;
                    }
                }
            }
            _ => {
                let mut buf = Vec::new();
                io::stdin()
                    .read_to_end(&mut buf)
                    .map_err(|e| format!("Cannot read stdin: {e}"))?;
                let mut builder = ParquetRecordBatchReaderBuilder::try_new(Bytes::from(buf))
                    .map_err(|e| format!("Cannot read parquet from stdin"))?;

                let schema = builder.parquet_schema();

                let x_leaf_idx: usize = resolve_colspec(&x_col, schema)?;
                let y_leaf_idxs: Vec<usize> = y_cols
                    .iter()
                    .map(|y| resolve_colspec(y, schema))
                    .collect::<Result<Vec<_>, _>>()?;

                // TODO: Construct the actual ParquetScatterSource, and include a ProjectionMask
            }
        }

        // @dev TODO: delete this placeholder
        return Ok(ParquetScatterSource {
            x_col: vec![0.0],
            y_cols: vec![vec![0.0], vec![0.0]],
        });
    }
}

fn resolve_colspec(col: &ColSpec, schema: &SchemaDescriptor) -> Result<usize, String> {
    match col {
        ColSpec::Index(i) => {
            let all_roots = schema.root_schema().get_fields();

            if *i >= all_roots.len() {
                return Err("X column index exceed total number of columns in parquet file. Make sure you are zero-indexing. Indices only support unnested columns.".to_string());
            }

            let matching_leaf_indices: Vec<usize> = (0..schema.num_columns())
                .filter(|leaf_idx| schema.get_column_root_idx(*leaf_idx) == *i)
                .collect();

            match matching_leaf_indices.len() {
                0 => return Err("No matching leaf index found for x column index".to_string()),
                1 => return Ok(matching_leaf_indices[0]),
                _ => return Err("Provided x column index matches to a nested column, which is not currently supported. Please specify a single column within the nested column by name".to_string()),
            }
        }
        ColSpec::Name(x_name) => {
            let matching_leaf_indices: Vec<usize> = schema
                .columns()
                .iter()
                .enumerate()
                .filter(|(_, col)| x_name == col.name())
                .map(|(i, _)| i)
                .collect();

            match matching_leaf_indices.len() {
                0 => return Err("Couldn't find a column that matched the x column name provided".to_string()),
                1 => return Ok(matching_leaf_indices[0]),
               _ => return Err("Provided column name matched to multiple columns, which is not currently supported.".to_string()),
            }
        }
    }
}
