use std::str::FromStr;

use argus::signals::interpolation::{Constant, Linear};
use argus::signals::Signal;
use pyo3::exceptions::{PyNotImplementedError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyType;

use crate::{DType, PyArgusError};

#[derive(Debug, Clone, Copy, Default)]
pub(crate) enum PyInterp {
    #[default]
    Linear,
    Constant,
}

impl FromStr for PyInterp {
    type Err = PyErr;

    fn from_str(method: &str) -> Result<Self, Self::Err> {
        match method {
            "linear" => Ok(PyInterp::Linear),
            "constant" => Ok(PyInterp::Constant),
            _ => Err(PyValueError::new_err(format!(
                "unsupported interpolation method `{}`",
                method
            ))),
        }
    }
}

#[derive(Debug, Clone, derive_more::From, derive_more::TryInto)]
#[try_into(owned, ref, ref_mut)]
pub enum SignalKind {
    Bool(Signal<bool>),
    Int(Signal<i64>),
    UnsignedInt(Signal<u64>),
    Float(Signal<f64>),
}

impl SignalKind {
    /// Get the kind of the signal
    pub fn kind(&self) -> DType {
        match self {
            SignalKind::Bool(_) => DType::Bool,
            SignalKind::Int(_) => DType::Int,
            SignalKind::UnsignedInt(_) => DType::UnsignedInt,
            SignalKind::Float(_) => DType::Float,
        }
    }
}

#[pyclass(name = "Signal", subclass, module = "argus")]
#[derive(Debug, Clone)]
pub struct PySignal {
    pub(crate) interpolation: PyInterp,
    pub(crate) signal: SignalKind,
}

impl PySignal {
    pub(crate) fn new<T>(signal: T, interpolation: PyInterp) -> Self
    where
        T: Into<SignalKind>,
    {
        Self {
            interpolation,
            signal: signal.into(),
        }
    }
}

#[pymethods]
impl PySignal {
    #[getter]
    fn kind(&self) -> DType {
        self.signal.kind()
    }

    fn __repr__(&self) -> String {
        match &self.signal {
            SignalKind::Bool(sig) => format!("Signal::<{}>::{:?}", "bool", sig),
            SignalKind::Int(sig) => format!("Signal::<{}>::{:?}", "i64", sig),
            SignalKind::UnsignedInt(sig) => format!("Signal::<{}>::{:?}", "u64", sig),
            SignalKind::Float(sig) => format!("Signal::<{}>::{:?}", "f64", sig),
        }
    }

    /// Check if the signal is empty
    fn is_empty(&self) -> bool {
        match &self.signal {
            SignalKind::Bool(sig) => sig.is_empty(),
            SignalKind::Int(sig) => sig.is_empty(),
            SignalKind::UnsignedInt(sig) => sig.is_empty(),
            SignalKind::Float(sig) => sig.is_empty(),
        }
    }

    /// The start time of the signal
    #[getter]
    fn start_time(&self) -> Option<f64> {
        use core::ops::Bound::*;
        let start_time = match &self.signal {
            SignalKind::Bool(sig) => sig.start_time()?,
            SignalKind::Int(sig) => sig.start_time()?,
            SignalKind::UnsignedInt(sig) => sig.start_time()?,
            SignalKind::Float(sig) => sig.start_time()?,
        };
        match start_time {
            Included(t) | Excluded(t) => Some(t.as_secs_f64()),
            _ => None,
        }
    }

    /// The end time of the signal
    #[getter]
    fn end_time(&self) -> Option<f64> {
        use core::ops::Bound::*;
        let end_time = match &self.signal {
            SignalKind::Bool(sig) => sig.end_time()?,
            SignalKind::Int(sig) => sig.end_time()?,
            SignalKind::UnsignedInt(sig) => sig.end_time()?,
            SignalKind::Float(sig) => sig.end_time()?,
        };
        match end_time {
            Included(t) | Excluded(t) => Some(t.as_secs_f64()),
            _ => None,
        }
    }

    /// Create a new empty signal
    #[new]
    #[pyo3(signature = (*, interpolation_method = "linear"))]
    fn init(interpolation_method: &str) -> PyResult<Self> {
        _ = interpolation_method;
        Err(PyNotImplementedError::new_err(
            "cannot directly construct an abstract Signal",
        ))
    }

    /// Create a new signal with constant value
    #[classmethod]
    #[pyo3(signature = (value, *, interpolation_method = "linear"))]
    fn constant(_: &PyType, _py: Python<'_>, value: &PyAny, interpolation_method: &str) -> PyResult<Py<Self>> {
        _ = value;
        _ = interpolation_method;
        Err(PyNotImplementedError::new_err(
            "cannot directly construct an abstract Signal",
        ))
    }

    /// Create a new signal from some finite number of samples
    #[classmethod]
    #[pyo3(signature = (samples, *, interpolation_method = "linear"))]
    fn from_samples(_: &PyType, samples: Vec<(f64, &PyAny)>, interpolation_method: &str) -> PyResult<Py<Self>> {
        _ = samples;
        _ = interpolation_method;
        Err(PyNotImplementedError::new_err(
            "cannot directly construct an abstract Signal",
        ))
    }

    /// Push a new sample into the given signal.
    #[pyo3(signature = (time, value))]
    fn push(_: PyRefMut<'_, Self>, time: f64, value: &PyAny) -> PyResult<()> {
        _ = time;
        _ = value;
        Err(PyNotImplementedError::new_err(
            "cannot push samples to an abstract Signal",
        ))
    }

    /// Get the value of the signal at the given time point.
    ///
    /// If there exists a sample, then the value is returned, otherwise the value is
    /// interpolated. If the time point lies outside of the domain of the signal, then
    /// `None` is returned.
    fn at(_self_: PyRef<'_, Self>, time: f64) -> PyResult<Option<&PyAny>> {
        _ = time;
        Err(PyNotImplementedError::new_err(
            "cannot query for samples in an abstract Signal",
        ))
    }
}

macro_rules! impl_signals {
    ($ty_name:ident, $ty:ty) => {
        paste::paste! {
            #[pyclass(extends=PySignal, module = "argus")]
            #[derive(Debug, Copy, Clone)]
            pub struct [<$ty_name Signal>];

            #[pymethods]
            impl [<$ty_name Signal>] {
                /// Create a new empty signal
                #[new]
                #[pyo3(signature = (*, interpolation_method = "linear"))]
                fn init(interpolation_method: &str) -> PyResult<(Self, PySignal)> {
                    let interp = PyInterp::from_str(interpolation_method)?;
                    Ok((Self, PySignal::new(Signal::<$ty>::new(), interp)))
                }

                /// Create a new signal with constant value
                #[classmethod]
                #[pyo3(signature = (value, *, interpolation_method = "linear"))]
                fn constant(_: &PyType, py: Python<'_>, value: $ty, interpolation_method: &str) -> PyResult<Py<Self>> {
                    let interp = PyInterp::from_str(interpolation_method)?;
                    Py::new(
                        py,
                        (Self, PySignal::new(Signal::constant(value), interp))
                    )
                }

                /// Create a new signal from some finite number of samples
                #[classmethod]
                #[pyo3(signature = (samples, *, interpolation_method = "linear"))]
                fn from_samples(_: &PyType, samples: Vec<(f64, $ty)>, interpolation_method: &str) -> PyResult<Py<Self>> {
                    let ret: Signal::<$ty> = Signal::<$ty>::try_from_iter(samples
                        .into_iter()
                        .map(|(t, v)| (core::time::Duration::try_from_secs_f64(t).unwrap_or_else(|err| panic!("Value = {}, {}", t, err)), v))
                    ).map_err(PyArgusError::from)?;

                    let interp = PyInterp::from_str(interpolation_method)?;
                    Python::with_gil(|py| {
                        Py::new(
                            py,
                            (Self, PySignal::new(ret, interp))
                        )
                    })
                }

                /// Push a new sample into the given signal.
                #[pyo3(signature = (time, value))]
                fn push(mut self_: PyRefMut<'_, Self>, time: f64, value: $ty) -> Result<(), PyArgusError> {
                    let super_: &mut PySignal = self_.as_mut();
                    let signal: &mut Signal<$ty> = (&mut super_.signal).try_into().unwrap();
                    // if it is an empty signal, make it sampled. Otherwise, throw an error.
                    let signal: &mut Signal<$ty> = match signal {
                        Signal::Empty => {
                            super_.signal = Signal::<$ty>::with_capacity(1).into();
                            (&mut super_.signal).try_into().unwrap()
                        }
                        _ => signal,
                    };
                    signal.push(core::time::Duration::from_secs_f64(time), value)?;
                    Ok(())
                }

                /// Get the value of the signal at the given time point.
                ///
                /// If there exists a sample, then the value is returned, otherwise the value is
                /// interpolated. If the time point lies outside of the domain of the signal, then `None`
                /// is returned.
                fn at(self_: PyRef<'_, Self>, time: f64) -> Option<$ty> {
                    let super_ = self_.as_ref();
                    let signal: &Signal<$ty> = (&super_.signal).try_into().unwrap();
                    let time = core::time::Duration::from_secs_f64(time);
                    match super_.interpolation {
                        PyInterp::Linear => signal.interpolate_at::<Linear>(time),
                        PyInterp::Constant => signal.interpolate_at::<Constant>(time),
                    }
                }

            }
        }
    };
}

impl_signals!(Bool, bool);
impl_signals!(Int, i64);
impl_signals!(UnsignedInt, u64);
impl_signals!(Float, f64);

pub fn init(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PySignal>()?;
    m.add_class::<BoolSignal>()?;
    m.add_class::<IntSignal>()?;
    m.add_class::<UnsignedIntSignal>()?;
    m.add_class::<FloatSignal>()?;

    Ok(())
}
