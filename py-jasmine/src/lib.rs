pub mod error;
use error::{JasmineError, JasmineParseError};
use pyo3::prelude::*;

#[pymodule]
fn jasmine(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // m.add_class::<QConnector>()?;
    m.add("JasmineError", py.get_type_bound::<JasmineError>())?;
    m.add(
        "JasmineParseError",
        py.get_type_bound::<JasmineParseError>(),
    )?;
    // m.add_function(wrap_pyfunction!(read_binary_table, m)?)?;
    // m.add_function(wrap_pyfunction!(generate_ipc_msg, m)?)?;
    Ok(())
}
