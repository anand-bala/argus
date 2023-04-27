mod expr;
mod semantics;
mod signals;

use pyo3::prelude::*;

#[pymodule]
#[pyo3(name = "_argus")]
fn pyargus(py: Python, m: &PyModule) -> PyResult<()> {
    expr::init(py, m)?;
    signals::init(py, m)?;
    semantics::init(py, m)?;
    Ok(())
}
