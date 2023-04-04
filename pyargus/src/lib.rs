use argus_core::expr::{BoolExpr, NumExpr};
use argus_core::prelude::*;
use pyo3::prelude::*;

#[pyclass(name = "NumExpr", subclass)]
#[derive(Clone)]
struct PyNumExpr(Box<NumExpr>);

#[pymethods]
impl PyNumExpr {
    fn __repr__(&self) -> String {
        format!("{:?}", &self.0)
    }

    fn __neg__(&self) -> PyResult<Py<Negate>> {
        Python::with_gil(|py| Py::new(py, Negate::new(self.clone())))
    }

    fn __add__(&self, other: &Self) -> PyResult<Py<Add>> {
        Python::with_gil(|py| Py::new(py, Add::new(vec![self.clone(), other.clone()])))
    }

    fn __mul__(&self, other: &Self) -> PyResult<Py<Mul>> {
        Python::with_gil(|py| Py::new(py, Mul::new(vec![self.clone(), other.clone()])))
    }

    fn __truediv__(&self, other: &Self) -> PyResult<Py<Div>> {
        Python::with_gil(|py| Py::new(py, Div::new(self.clone(), other.clone())))
    }

    fn __abs__(&self) -> PyResult<Py<Abs>> {
        Python::with_gil(|py| Py::new(py, Abs::new(self.clone())))
    }
}

#[pyclass(extends=PyNumExpr)]
struct ConstInt;

#[pymethods]
impl ConstInt {
    #[new]
    fn new(val: i64) -> (Self, PyNumExpr) {
        (Self, PyNumExpr(Box::new(NumExpr::IntLit(val))))
    }
}

#[pyclass(extends=PyNumExpr)]
struct ConstUInt;

#[pymethods]
impl ConstUInt {
    #[new]
    fn new(val: u64) -> (Self, PyNumExpr) {
        (Self, PyNumExpr(Box::new(NumExpr::UIntLit(val))))
    }
}

#[pyclass(extends=PyNumExpr)]
struct ConstFloat;

#[pymethods]
impl ConstFloat {
    #[new]
    fn new(val: f64) -> (Self, PyNumExpr) {
        (Self, PyNumExpr(Box::new(NumExpr::FloatLit(val))))
    }
}

#[pyclass(extends=PyNumExpr)]
struct VarInt;

#[pymethods]
impl VarInt {
    #[new]
    fn new(name: String) -> (Self, PyNumExpr) {
        (Self, PyNumExpr(Box::new(NumExpr::IntVar { name })))
    }
}

#[pyclass(extends=PyNumExpr)]
struct VarUInt;

#[pymethods]
impl VarUInt {
    #[new]
    fn new(name: String) -> (Self, PyNumExpr) {
        (Self, PyNumExpr(Box::new(NumExpr::UIntVar { name })))
    }
}

#[pyclass(extends=PyNumExpr)]
struct VarFloat;

#[pymethods]
impl VarFloat {
    #[new]
    fn new(name: String) -> (Self, PyNumExpr) {
        (Self, PyNumExpr(Box::new(NumExpr::FloatVar { name })))
    }
}

#[pyclass(extends=PyNumExpr)]
struct Negate;

#[pymethods]
impl Negate {
    #[new]
    fn new(arg: PyNumExpr) -> (Self, PyNumExpr) {
        let arg = arg.0;
        (Self, PyNumExpr(Box::new(NumExpr::Neg { arg })))
    }
}

#[pyclass(extends=PyNumExpr)]
struct Add;

#[pymethods]
impl Add {
    #[new]
    fn new(args: Vec<PyNumExpr>) -> (Self, PyNumExpr) {
        let args: Vec<NumExpr> = args.into_iter().map(|arg| *arg.0).collect();
        (Self, PyNumExpr(Box::new(NumExpr::Add { args })))
    }
}

#[pyclass(extends=PyNumExpr)]
struct Sub;

#[pymethods]
impl Sub {
    #[new]
    fn new(lhs: PyNumExpr, rhs: PyNumExpr) -> (Self, PyNumExpr) {
        let lhs = lhs.0;
        let rhs = rhs.0;
        (Self, PyNumExpr(Box::new(NumExpr::Sub { lhs, rhs })))
    }
}

#[pyclass(extends=PyNumExpr)]
struct Mul;

#[pymethods]
impl Mul {
    #[new]
    fn new(args: Vec<PyNumExpr>) -> (Self, PyNumExpr) {
        let args: Vec<NumExpr> = args.into_iter().map(|arg| *arg.0).collect();
        (Self, PyNumExpr(Box::new(NumExpr::Mul { args })))
    }
}

#[pyclass(extends=PyNumExpr)]
struct Div;

#[pymethods]
impl Div {
    #[new]
    fn new(dividend: PyNumExpr, divisor: PyNumExpr) -> (Self, PyNumExpr) {
        let dividend = dividend.0;
        let divisor = divisor.0;
        (Self, PyNumExpr(Box::new(NumExpr::Div { dividend, divisor })))
    }
}

// #[pyclass(extends=PyNumExpr)]
// struct Pow;
//
// #[pymethods]
// impl Pow {
//     #[new]
//     fn new(base: PyNumExpr, exponent: PyNumExpr) -> (Self, PyNumExpr) {
//         let base = base.0;
//         let exponent = exponent.0;
//         (Self, PyNumExpr(Box::new(NumExpr:: { base, exponent })))
//     }
// }

#[pyclass(extends=PyNumExpr)]
struct Abs;

#[pymethods]
impl Abs {
    #[new]
    fn new(arg: PyNumExpr) -> (Self, PyNumExpr) {
        let arg = arg.0;
        (Self, PyNumExpr(Box::new(NumExpr::Abs { arg })))
    }
}

#[pyclass(name = "BoolExpr")]
struct PyBoolExpr(BoolExpr);

#[pymodule]
fn pyargus(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyNumExpr>()?;
    m.add_class::<ConstInt>()?;
    m.add_class::<ConstUInt>()?;
    m.add_class::<ConstFloat>()?;
    m.add_class::<VarInt>()?;
    m.add_class::<VarUInt>()?;
    m.add_class::<VarFloat>()?;
    m.add_class::<Negate>()?;
    m.add_class::<Add>()?;
    m.add_class::<Mul>()?;
    m.add_class::<Div>()?;
    // m.add_class::<Pow>()?;

    m.add_class::<PyBoolExpr>()?;

    Ok(())
}
