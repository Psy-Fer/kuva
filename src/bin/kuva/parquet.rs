use arrow::array::RecordBatch;
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
    pub x_col: record_batch::RecordBatch,
    pub y_cols: record_batch::RecordBatch,
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
        x: ColSpec,
        y: Vec<ColSpec>,
    ) -> Result<Self, String> {
        // Logic for reading parquet data as bytes from stdin

        match input {
            Some(p) if p.to_str() != Some("-") => {
                let file = File::open(input.unwrap())
                    .map_err(|e| format!("Failed to open the provided file. {}", e))?;
                let mut builder = ParquetRecordBatchReaderBuilder::try_new(file)
                    .map_err(|e| format!("Failed to read from the provided parquet file. {}", e))?;

                let schema = builder.parquet_schema();

                let x_leaf_idx: usize = parquet_schema_match_colspec(&x, schema)?;
                let y_leaf_idxs: Vec<usize> = y
                    .iter()
                    .map(|y| parquet_schema_match_colspec(y, schema))
                    .collect::<Result<Vec<_>, _>>()?;

                // TODO: Construct the actual ParquetScatterSource, and include a ProjectionMask
            }
            _ => {
                let mut buf = Vec::new();
                io::stdin()
                    .read_to_end(&mut buf)
                    .map_err(|e| format!("Cannot read stdin: {e}"))?;
                let mut builder = ParquetRecordBatchReaderBuilder::try_new(Bytes::from(buf))
                    .map_err(|e| format!("Cannot read parquet from stdin"))?;

                let schema = builder.parquet_schema();

                let x_leaf_idx: usize = parquet_schema_match_colspec(&x, schema)?;
                let y_leaf_idxs: Vec<usize> = y
                    .iter()
                    .map(|y| parquet_schema_match_colspec(y, schema))
                    .collect::<Result<Vec<_>, _>>()?;

                // TODO: Construct the actual ParquetScatterSource, and include a ProjectionMask
            }
        }

        // @dev TODO: delete this placeholder
        return Ok(ParquetScatterSource {
            x_col: RecordBatch::new_empty(std::sync::Arc::new(arrow::datatypes::Schema::empty())),
            y_cols: RecordBatch::new_empty(std::sync::Arc::new(arrow::datatypes::Schema::empty())),
        });
    }
}

fn parquet_schema_match_colspec(col: &ColSpec, schema: &SchemaDescriptor) -> Result<usize, String> {
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
