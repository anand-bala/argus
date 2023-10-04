mod expr;
mod semantics;
mod signals;

use argus::Error as ArgusError;
use pyo3::exceptions::{PyKeyError, PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyFloat, PyInt, PyType};

#[derive(derive_more::From)]
struct PyArgusError(ArgusError);

impl From<PyArgusError> for PyErr {
    fn from(value: PyArgusError) -> Self {
        use argus::Error::*;
        match value.0 {
            err @ (IncompleteArgs | InvalidOperation | IdentifierRedeclaration | InvalidInterval { reason: _ }) => {
                PyValueError::new_err(err.to_string())
            }
            err @ (InvalidPushToSignal
            | NonMonotonicSignal {
                end_time: _,
                current_sample: _,
            }) => PyRuntimeError::new_err(err.to_string()),
            err @ SignalNotPresent => PyKeyError::new_err(err.to_string()),
            err @ (InvalidSignalType | InvalidCast { from: _, to: _ }) => PyTypeError::new_err(err.to_string()),
        }
    }
}

#[pyclass(module = "argus", name = "dtype")]
#[derive(Copy, Clone, Debug)]
pub enum DType {
    #[pyo3(name = "bool_")]
    Bool,
    #[pyo3(name = "int64")]
    Int,
    #[pyo3(name = "uint64")]
    UnsignedInt,
    #[pyo3(name = "float64")]
    Float,
}

#[pymethods]
impl DType {
    #[classmethod]
    fn convert(_: &PyType, dtype: &PyAny, py: Python<'_>) -> PyResult<Self> {
        use DType::*;
        if dtype.is_instance_of::<DType>() {
            dtype.extract::<DType>()
        } else if dtype.is_instance_of::<PyType>() {
            let dtype = dtype.downcast_exact::<PyType>()?;
            if dtype.is(PyType::new::<PyBool>(py)) {
                Ok(Bool)
            } else if dtype.is(PyType::new::<PyInt>(py)) {
                Ok(Int)
            } else if dtype.is(PyType::new::<PyFloat>(py)) {
                Ok(Float)
            } else {
                Err(PyTypeError::new_err(format!("unsupported type {}", dtype)))
            }
        } else {
            Err(PyTypeError::new_err(format!(
                "unsupported dtype {}, expected a `type`",
                dtype
            )))
        }
    }
}

#[pymodule]
#[pyo3(name = "_argus")]
fn pyargus(py: Python, m: &PyModule) -> PyResult<()> {
    pyo3_log::init();

    m.add_class::<DType>()?;

    expr::init(py, m)?;
    signals::init(py, m)?;
    semantics::init(py, m)?;
    Ok(())
}
