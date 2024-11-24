use jasmine::errors::JError;
use pyo3::create_exception;
use pyo3::exceptions::{PyException, PyRuntimeError};
use pyo3::PyErr;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JasmineErr {
    #[error(transparent)]
    KolaError(#[from] JError),

    #[error("{0:?}")]
    PythonError(String),
}

impl std::convert::From<JasmineErr> for PyErr {
    fn from(err: JasmineErr) -> PyErr {
        let default = || PyRuntimeError::new_err(format!("{:?}", &err));
        use JasmineErr::*;
        match &err {
            KolaError(e) => match e {
                JError::ParserErr(_) => JasmineParseError::new_err(err.to_string()),
                _ => JasmineError::new_err(err.to_string()),
            },

            PythonError(_) => default(),
        }
    }
}

create_exception!(kola.exceptions, JasmineError, PyException);
create_exception!(kola.exceptions, JasmineParseError, PyException);
