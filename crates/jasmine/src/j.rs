use indexmap::IndexMap;
use ndarray::ArcArray2;
use polars::{frame::DataFrame, series::Series};

#[derive(PartialEq, Debug, Clone)]
pub enum J {
    Boolean(bool),  // -1
    I64(i64),       // -5
    Date(i32),      // -6 start from 1970.01.01
    Time(i64),      // -7 00:00:00.0 - 23:59:59.999999999
    Datetime(i64),  // -8 start from 1970.01.01T00:00:00.0
    Timestamp(i64), // -9 start from 1970.01.01D00:00:00.0
    Duration(i64),  // -10
    F64(f64),       // -12
    String(String), // -13
    Symbol(String), // -14

    None, // 0

    Series(Series), // 1-14 -> Arrow IPC

    Matrix(ArcArray2<f64>), // 21

    MixedList(Vec<J>),         // 90
    Dict(IndexMap<String, J>), // 91 -> skip Dataframe
    DataFrame(DataFrame),      // 92 -> Arrow IPC

    Err(String), // 128 => string
}
