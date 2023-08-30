use std::time::Duration;

use argus_core::signals::Signal;
use pyo3::prelude::*;

use crate::{DType, PyArgusError};

#[pyclass(name = "InterpolationMethod", module = "argus")]
#[derive(Debug, Clone, Copy, Default)]
pub enum PyInterp {
    #[default]
    Linear,
}

#[pyclass(name = "Signal", subclass, module = "argus")]
#[derive(Debug, Clone)]
pub struct PySignal {
    pub kind: DType,
    pub interpolation: PyInterp,
}

macro_rules! impl_signals {
    ($ty_name:ident, $ty:ty) => {
        paste::paste! {
            #[pyclass(extends=PySignal, module = "argus")]
            #[derive(Debug, Clone, derive_more::From)]
            pub struct [<$ty_name Signal>](pub Signal<$ty>);

            impl [<$ty_name Signal>] {
                #[inline]
                pub fn super_type() -> PySignal {
                    PySignal {
                        interpolation: PyInterp::Linear,
                        kind: DType::$ty_name,
                    }
                }
            }

            #[pymethods]
            impl [<$ty_name Signal>] {
                fn __repr__(&self) -> String {
                    format!("Signal::<{}>::{:?}", stringify!($ty), self.0)
                }

                /// Create a new empty signal
                #[new]
                #[pyo3(signature = ())]
                fn new() -> (Self, PySignal) {
                    (Self(Signal::new()), Self::super_type())
                }

                fn __init__(self_: PyRef<'_, Self>) -> PyRef<'_, Self> {
                    self_
                }

                /// Create a new signal with constant value
                #[staticmethod]
                fn constant(py: Python<'_>, value: $ty) -> PyResult<Py<Self>> {
                    Py::new(
                        py,
                        (Self(Signal::constant(value)), Self::super_type())
                    )
                }

                /// Create a new signal from some finite number of samples
                #[staticmethod]
                fn from_samples(samples: Vec<(f64, $ty)>) -> PyResult<Py<Self>> {
                    let ret: Signal<$ty> = samples
                        .into_iter()
                        .map(|(t, v)| (Duration::from_secs_f64(t), v))
                        .collect();
                    Python::with_gil(|py| {
                        Py::new(
                            py,
                            (Self(ret), Self::super_type())
                        )
                    })
                }

                /// Push a new sample into the given signal.
                #[pyo3(signature = (time, value))]
                fn push(&mut self, time: f64, value: $ty) -> Result<(), PyArgusError> {
                    self.0.push(Duration::from_secs_f64(time), value)?;
                    Ok(())
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
