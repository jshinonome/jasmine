use chrono::{DateTime, NaiveDate, NaiveTime, TimeDelta};
use jasmine::j::J;
use numpy::ToPyArray;
use pyo3::{
    pyclass,
    types::{PyDict, PyDictMethods, PyTuple},
    IntoPy, PyObject, PyResult, Python, ToPyObject,
};
use pyo3_polars::{PyDataFrame, PySeries};

use crate::error::JasmineError;

#[pyclass]
pub struct JObj {
    j: J,
}

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
                    .map(|k| JObj { j: k.clone() }.as_py(py))
                    .collect::<PyResult<Vec<PyObject>>>()?;
                Ok(PyTuple::new_bound(py, py_objects).into())
            }
            J::Dict(dict) => {
                let py_dict = PyDict::new_bound(py);
                for (k, v) in dict.into_iter() {
                    py_dict.set_item(k, JObj { j: v.clone() }.as_py(py)?)?;
                }
                Ok(py_dict.into())
            }
            J::DataFrame(data_frame) => Ok(PyDataFrame(data_frame.clone()).into_py(py)),
            J::Err(v) => Err(JasmineError::new_err(v.to_string()).into()),
        }
    }

    pub fn get_type_num(&self, py: Python<'_>) -> PyObject {
        match &self.j {
            J::None => 0i32.into_py(py),

            J::Boolean(_) => 1i32.into_py(py),
            J::I64(_) => 2i32.into_py(py),
            J::Date(_) => 3i32.into_py(py),
            J::Time(_) => 4i32.into_py(py),
            J::Datetime(_) => 5i32.into_py(py),
            J::Timestamp(_) => 6i32.into_py(py),
            J::Duration(_) => 7i32.into_py(py),
            J::F64(_) => 8i32.into_py(py),
            J::String(_) => 9i32.into_py(py),
            J::Symbol(_) => 10i32.into_py(py),
            J::Series(_) => 11i32.into_py(py),
            J::Matrix(_) => 12i32.into_py(py),
            J::MixedList(_) => 13i32.into_py(py),
            J::Dict(_) => 14i32.into_py(py),
            J::DataFrame(_) => 15i32.into_py(py),
            J::Err(_) => 16i32.into_py(py),
        }
    }
}
