use std::sync::Arc;
use arrow::{error::ArrowError, record_batch::RecordBatch};
use datafusion::logical_plan::{col, min};
use datafusion::{datasource::MemTable, prelude::ExecutionContext};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn main(contents: Vec<u8>) -> Result<(), JsValue> {
    let cursor = std::io::Cursor::new(contents);
    let reader = match arrow::ipc::reader::FileReader::try_new(cursor) {
        Ok(reader) => reader,
        Err(error) => return Err(format!("{}", error).into()),
    };

    let schema = reader.schema();

    let batches: Result<Vec<arrow::record_batch::RecordBatch>, ArrowError> = reader.collect();

    let record_batches = match batches {
        Ok(record_batches) => record_batches,
        Err(error) => Err(format!("{}", error).into()),
    };

    let table = Arc::new(MemTable::try_new(schema, vec![record_batches]).unwrap());

    let mut ctx = ExecutionContext::new();

    let df = ctx.read_table(table).unwrap();

    let df = df.filter(col("a").lt_eq(col("b"))).unwrap()
            .aggregate(vec![col("a")], vec![min(col("b"))]).unwrap()
            .limit(100).unwrap();

    let results: Vec<RecordBatch> = df.collect().await.unwrap();

    Ok(())
}
