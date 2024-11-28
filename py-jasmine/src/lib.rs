pub mod error;
pub mod j;
pub mod parse;
use error::{JasmineError, JasmineParseError};
use j::{JObj, JType};
use jasmine::trace;
use parse::{
    parse_source_code, AstAssign, AstBinOp, AstCall, AstDataFrame, AstDict, AstFn, AstId, AstIf,
    AstIndexAssign, AstList, AstMatrix, AstOp, AstRaise, AstReturn, AstSeries, AstSkip, AstSql,
    AstTry, AstUnaryOp, AstWhile,
};
use pyo3::prelude::*;

#[pyfunction]
pub fn print_trace(source: &str, path: &str, pos: usize, msg: &str) -> String {
    trace(source, path, pos, msg)
}

#[pymodule]
fn jasminum(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("JasmineError", py.get_type_bound::<JasmineError>())?;
    m.add(
        "JasmineParseError",
        py.get_type_bound::<JasmineParseError>(),
    )?;
    m.add_class::<JObj>()?;
    m.add_class::<AstId>()?;
    m.add_class::<AstFn>()?;
    m.add_class::<AstUnaryOp>()?;
    m.add_class::<AstBinOp>()?;
    m.add_class::<AstAssign>()?;
    m.add_class::<AstIndexAssign>()?;
    m.add_class::<AstOp>()?;
    m.add_class::<AstId>()?;
    m.add_class::<AstCall>()?;
    m.add_class::<AstIf>()?;
    m.add_class::<AstWhile>()?;
    m.add_class::<AstTry>()?;
    m.add_class::<AstReturn>()?;
    m.add_class::<AstRaise>()?;
    m.add_class::<AstDataFrame>()?;
    m.add_class::<AstMatrix>()?;
    m.add_class::<AstDict>()?;
    m.add_class::<AstList>()?;
    m.add_class::<AstSeries>()?;
    m.add_class::<AstSql>()?;
    m.add_class::<AstSkip>()?;
    m.add_function(wrap_pyfunction!(parse_source_code, m)?)?;
    m.add_function(wrap_pyfunction!(print_trace, m)?)?;
    Ok(())
}
