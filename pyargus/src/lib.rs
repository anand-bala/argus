mod expr;
mod semantics;
mod signals;

use argus_core::ArgusError;
use pyo3::exceptions::{PyKeyError, PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;

#[derive(derive_more::From)]
struct PyArgusError(ArgusError);

impl From<PyArgusError> for PyErr {
    fn from(value: PyArgusError) -> Self {
        use argus_core::Error::*;
        match value.0 {
            err @ (IncompleteArgs | InvalidOperation | IdentifierRedeclaration) => {
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

#[pyclass]
#[derive(Copy, Clone, Debug)]
pub enum DType {
    Bool,
    Int,
    UnsignedInt,
    Float,
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
