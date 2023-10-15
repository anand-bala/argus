mod expr;
mod semantics;
mod signals;

use argus::Error as ArgusError;
use ariadne::Source;
use pyo3::exceptions::{PyKeyError, PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyFloat, PyInt, PyType};

use crate::expr::{PyBoolExpr, PyNumExpr};

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

/// Parse a string expression into a concrete Argus expression.
#[pyfunction]
fn parse_expr(expr_str: &str) -> PyResult<PyObject> {
    use argus::expr::Expr;
    use ariadne::{Color, Label, Report, ReportKind};

    match argus::parse_str(expr_str) {
        Ok(expr) => Python::with_gil(|py| match expr {
            Expr::Bool(e) => PyBoolExpr::from_expr(py, e),
            Expr::Num(e) => PyNumExpr::from_expr(py, e),
        }),
        Err(errs) => {
            let mut buf = Vec::new();
            {
                errs.into_iter().for_each(|e| {
                    Report::build(ReportKind::Error, (), e.span().start)
                        .with_message(e.to_string())
                        .with_label(
                            Label::new(e.span().into_range())
                                .with_message(e.reason().to_string())
                                .with_color(Color::Red),
                        )
                        .with_labels(e.contexts().map(|(label, span)| {
                            Label::new(span.into_range())
                                .with_message(format!("while parsing this {}", label))
                                .with_color(Color::Yellow)
                        }))
                        .finish()
                        .write(Source::from(expr_str.to_owned()), &mut buf)
                        .unwrap()
                });
            }
            let output = std::str::from_utf8(buf.as_slice())?.to_owned();
            Err(PyValueError::new_err(output))
        }
    }
}

#[pymodule]
#[pyo3(name = "_argus")]
fn pyargus(py: Python, m: &PyModule) -> PyResult<()> {
    pyo3_log::init();

    let version = env!("CARGO_PKG_VERSION");

    m.add("__version__", version)?;
    m.add_class::<DType>()?;
    m.add_function(wrap_pyfunction!(parse_expr, m)?)?;

    expr::init(py, m)?;
    signals::init(py, m)?;
    semantics::init(py, m)?;
    Ok(())
}
