use chrono::{DateTime, NaiveDate, NaiveTime, TimeDelta};
use jasmine::j::J;
use numpy::ToPyArray;
use pyo3::{
    pyclass, pymethods,
    types::{PyDict, PyDictMethods, PyTuple},
    IntoPy, PyObject, PyResult, Python, ToPyObject,
};
use pyo3_polars::{PyDataFrame, PySeries};

use crate::error::JasmineError;

#[pyclass]
pub struct JObj {
    j: J,
    #[pyo3(get)]
    j_type: JType,
}

#[pymethods]
impl JObj {
    pub fn as_py(&self, py: Python<'_>) -> PyResult<PyObject> {
        match &self.j {
            J::Boolean(v) => Ok(v.into_py(py)),
            J::I64(v) => Ok(v.into_py(py)),
            J::Date(v) => Ok(NaiveDate::from_num_days_from_ce_opt(*v)
                .unwrap()
                .to_object(py)),
            J::Time(v) => Ok(NaiveTime::from_num_seconds_from_midnight_opt(
                (v / 1000_000_000) as u32,
                (v % 1000_000_000) as u32,
            )
            .into_py(py)),
            J::Datetime(v) => Ok(DateTime::from_timestamp_nanos(*v).into_py(py)),
            J::Timestamp(v) => Ok(DateTime::from_timestamp_nanos(*v).into_py(py)),
            J::Duration(v) => Ok(TimeDelta::nanoseconds(*v).into_py(py)),
            J::F64(v) => Ok(v.into_py(py)),
            J::String(v) => Ok(v.into_py(py)),
            J::Symbol(v) => Ok(v.into_py(py)),
            J::None => Ok(().to_object(py)),
            J::Series(series) => Ok(PySeries(series.clone()).into_py(py)),
            J::Matrix(matrix) => Ok(matrix.to_pyarray_bound(py).into()),
            J::MixedList(l) => {
                let py_objects = l
                    .into_iter()
                    .map(|k| JObj::new(k.clone()).as_py(py))
                    .collect::<PyResult<Vec<PyObject>>>()?;
                Ok(PyTuple::new_bound(py, py_objects).into())
            }
            J::Dict(dict) => {
                let py_dict = PyDict::new_bound(py);
                for (k, v) in dict.into_iter() {
                    py_dict.set_item(k, JObj::new(v.clone()).as_py(py)?)?;
                }
                Ok(py_dict.into())
            }
            J::DataFrame(data_frame) => Ok(PyDataFrame(data_frame.clone()).into_py(py)),
            J::Err(v) => Err(JasmineError::new_err(v.to_string()).into()),
        }
    }
}

impl JObj {
    pub fn new(j: J) -> Self {
        let j_type = match j {
            J::None => JType::None,
            J::Boolean(_) => JType::Boolean,
            J::I64(_) => JType::I64,
            J::Date(_) => JType::Date,
            J::Time(_) => JType::Time,
            J::Datetime(_) => JType::Datetime,
            J::Timestamp(_) => JType::Timestamp,
            J::Duration(_) => JType::Duration,
            J::F64(_) => JType::F64,
            J::String(_) => JType::String,
            J::Symbol(_) => JType::Symbol,
            J::Series(_) => JType::Series,
            J::Matrix(_) => JType::Matrix,
            J::MixedList(_) => JType::List,
            J::Dict(_) => JType::Dict,
            J::DataFrame(_) => JType::DataFrame,
            J::Err(_) => JType::Err,
        };
        Self { j, j_type }
    }
}

#[pyclass]
#[derive(Clone, PartialEq)]
pub enum JType {
    None,
    Boolean,
    I64,
    Date,
    Time,
    Datetime,
    Timestamp,
    Duration,
    F64,
    String,
    Symbol,
    Series,
    Matrix,
    List,
    Dict,
    DataFrame,
    Err,
}
