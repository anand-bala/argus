use std::collections::HashMap;

use argus_core::signals::{AnySignal, Signal};
use argus_semantics::{BooleanSemantics, Semantics, Trace};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};

use crate::expr::PyBoolExpr;
use crate::signals::{BoolSignal, FloatSignal, IntSignal, Kind, PySignal, UnsignedIntSignal};
use crate::PyArgusError;

#[derive(Debug, Clone, derive_more::From, derive_more::TryInto)]
#[try_into(owned, ref, ref_mut)]
enum SignalKind {
    Bool(Signal<bool>),
    Int(Signal<i64>),
    UnsignedInt(Signal<u64>),
    Float(Signal<f64>),
}

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
            let kind = val.borrow().kind;
            let signal: SignalKind = match kind {
                Kind::Bool => val.downcast::<PyCell<BoolSignal>>().unwrap().borrow().0.clone().into(),
                Kind::Int => val.downcast::<PyCell<IntSignal>>().unwrap().borrow().0.clone().into(),
                Kind::UnsignedInt => val
                    .downcast::<PyCell<UnsignedIntSignal>>()
                    .unwrap()
                    .borrow()
                    .0
                    .clone()
                    .into(),
                Kind::Float => val.downcast::<PyCell<FloatSignal>>().unwrap().borrow().0.clone().into(),
            };

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

#[pyclass(name = "BooleanSemantics")]
struct PyBooleanSemantics;

#[pymethods]
impl PyBooleanSemantics {
    #[staticmethod]
    fn eval(expr: &PyBoolExpr, trace: &PyTrace) -> PyResult<Py<BoolSignal>> {
        let sig = BooleanSemantics::eval(&expr.0, trace, ()).map_err(PyArgusError::from)?;
        Python::with_gil(|py| Py::new(py, (BoolSignal::from(sig), BoolSignal::super_type())))
    }
}

pub fn init(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyBooleanSemantics>()?;
    m.add_class::<PyTrace>()?;

    Ok(())
}
