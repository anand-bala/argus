use std::collections::HashMap;
use std::str::FromStr;

use argus::signals::interpolation::{Constant, Linear};
use argus::{AnySignal, BooleanSemantics, QuantitativeSemantics, Signal, Trace};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};

use crate::expr::PyBoolExpr;
use crate::signals::{BoolSignal, FloatSignal, PyInterp, PySignal, SignalKind};
use crate::PyArgusError;

#[pyclass(name = "Trace", module = "argus")]
#[derive(Debug, Clone, Default)]
pub struct PyTrace {
    signals: HashMap<String, SignalKind>,
}

#[pymethods]
impl PyTrace {
    #[new]
    fn new(dict: &PyDict) -> PyResult<Self> {
        let mut signals = HashMap::with_capacity(dict.len());
        for (key, val) in dict {
            let key: &PyString = key
                .downcast()
                .map_err(|e| PyTypeError::new_err(format!("expected dictionary with string keys for trace ({})", e)))?;
            let val: &PyCell<PySignal> = val.downcast().map_err(|e| {
                PyTypeError::new_err(format!(
                    "expected `argus.Signal` value for key `{}` in trace ({})",
                    key, e
                ))
            })?;
            let signal = val.borrow().signal.clone();
            signals.insert(key.to_string(), signal);
        }

        Ok(Self { signals })
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

impl Trace for PyTrace {
    fn signal_names(&self) -> Vec<&str> {
        self.signals.keys().map(|key| key.as_str()).collect()
    }

    fn get<T: 'static>(&self, name: &str) -> Option<&Signal<T>> {
        let kind = self.signals.get(name)?;
        let signal: &dyn AnySignal = match kind {
            SignalKind::Bool(sig) => sig,
            SignalKind::Int(sig) => sig,
            SignalKind::UnsignedInt(sig) => sig,
            SignalKind::Float(sig) => sig,
        };
        signal.as_any().downcast_ref::<Signal<T>>()
    }
}

#[pyfunction]
#[pyo3(signature = (expr, trace, *, interpolation_method = "linear"))]
fn eval_bool_semantics(expr: &PyBoolExpr, trace: &PyTrace, interpolation_method: &str) -> PyResult<Py<BoolSignal>> {
    let interp = PyInterp::from_str(interpolation_method)?;
    let sig = match interp {
        PyInterp::Linear => BooleanSemantics::eval::<Linear, Linear>(&expr.0, trace).map_err(PyArgusError::from)?,
        PyInterp::Constant => {
            BooleanSemantics::eval::<Constant, Constant>(&expr.0, trace).map_err(PyArgusError::from)?
        }
    };
    Python::with_gil(|py| Py::new(py, (BoolSignal, PySignal::new(sig, interp))))
}
#[pyfunction]
#[pyo3(signature = (expr, trace, *, interpolation_method = "linear"))]
fn eval_robust_semantics(expr: &PyBoolExpr, trace: &PyTrace, interpolation_method: &str) -> PyResult<Py<FloatSignal>> {
    let interp = PyInterp::from_str(interpolation_method)?;
    let sig = match interp {
        PyInterp::Linear => QuantitativeSemantics::eval::<Linear>(&expr.0, trace).map_err(PyArgusError::from)?,
        PyInterp::Constant => QuantitativeSemantics::eval::<Constant>(&expr.0, trace).map_err(PyArgusError::from)?,
    };
    Python::with_gil(|py| Py::new(py, (FloatSignal, PySignal::new(sig, interp))))
}

pub fn init(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyTrace>()?;
    m.add_function(wrap_pyfunction!(eval_bool_semantics, m)?)?;
    m.add_function(wrap_pyfunction!(eval_robust_semantics, m)?)?;

    Ok(())
}
