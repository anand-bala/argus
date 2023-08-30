use std::time::Duration;

use argus_core::prelude::*;
use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;

/// A base numeric expression
///
/// This is an abstract base class that provides an interface for all numeric
/// expressions supported in Argus (literals, arithmetic, and so on).
#[pyclass(name = "NumExpr", subclass, module = "argus")]
#[derive(Debug, Clone, derive_more::From)]
pub struct PyNumExpr(pub Box<NumExpr>);

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

    fn __sub__(&self, other: &Self) -> PyResult<Py<Sub>> {
        Python::with_gil(|py| Py::new(py, Sub::new(self.clone(), other.clone())))
    }

    fn __truediv__(&self, other: &Self) -> PyResult<Py<Div>> {
        Python::with_gil(|py| Py::new(py, Div::new(self.clone(), other.clone())))
    }

    fn __abs__(&self) -> PyResult<Py<Abs>> {
        Python::with_gil(|py| Py::new(py, Abs::new(self.clone())))
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<Py<Cmp>> {
        match op {
            CompareOp::Lt => Cmp::less_than(self.clone(), other.clone()),
            CompareOp::Le => Cmp::less_than_eq(self.clone(), other.clone()),
            CompareOp::Eq => Cmp::equal(self.clone(), other.clone()),
            CompareOp::Ne => Cmp::not_equal(self.clone(), other.clone()),
            CompareOp::Gt => Cmp::greater_than(self.clone(), other.clone()),
            CompareOp::Ge => Cmp::greater_than_eq(self.clone(), other.clone()),
        }
    }
}

/// Create a constant integer expression
#[pyclass(extends=PyNumExpr, module = "argus")]
pub struct ConstInt;

#[pymethods]
impl ConstInt {
    #[new]
    fn new(val: i64) -> (Self, PyNumExpr) {
        (Self, Box::new(NumExpr::IntLit(argus_core::expr::IntLit(val))).into())
    }
}

/// Create a constant _unsigned_ integer expression
///
/// # Warning
///
/// Negating an unsigned integer during evaluation _may_ lead to the evaluation method
/// panicking.
#[pyclass(extends=PyNumExpr, module = "argus")]
pub struct ConstUInt;

#[pymethods]
impl ConstUInt {
    #[new]
    fn new(val: u64) -> (Self, PyNumExpr) {
        (Self, Box::new(NumExpr::UIntLit(argus_core::expr::UIntLit(val))).into())
    }
}

/// Create a constant floating point number expression.
#[pyclass(extends=PyNumExpr, module = "argus")]
pub struct ConstFloat;

#[pymethods]
impl ConstFloat {
    #[new]
    fn new(val: f64) -> (Self, PyNumExpr) {
        (
            Self,
            Box::new(NumExpr::FloatLit(argus_core::expr::FloatLit(val))).into(),
        )
    }
}

/// Create a integer variable
#[pyclass(extends=PyNumExpr, module = "argus")]
pub struct VarInt;

#[pymethods]
impl VarInt {
    #[new]
    fn new(name: String) -> (Self, PyNumExpr) {
        (
            Self,
            Box::new(NumExpr::IntVar(argus_core::expr::IntVar { name })).into(),
        )
    }
}

/// Create an _unsigned_ integer variable
#[pyclass(extends=PyNumExpr, module = "argus")]
pub struct VarUInt;

#[pymethods]
impl VarUInt {
    #[new]
    fn new(name: String) -> (Self, PyNumExpr) {
        (
            Self,
            Box::new(NumExpr::UIntVar(argus_core::expr::UIntVar { name })).into(),
        )
    }
}

/// Create a float variable
#[pyclass(extends=PyNumExpr, module = "argus")]
pub struct VarFloat;

#[pymethods]
impl VarFloat {
    #[new]
    fn new(name: String) -> (Self, PyNumExpr) {
        (
            Self,
            Box::new(NumExpr::FloatVar(argus_core::expr::FloatVar { name })).into(),
        )
    }
}

/// Create a numeric negation expression
#[pyclass(extends=PyNumExpr, module = "argus")]
pub struct Negate;

#[pymethods]
impl Negate {
    #[new]
    fn new(arg: PyNumExpr) -> (Self, PyNumExpr) {
        let arg = arg.0;
        (Self, Box::new(NumExpr::Neg(argus_core::expr::Neg { arg })).into())
    }
}

/// Create a numeric addition expression
///
/// This expression is an `n`-ary expression that takes
#[pyclass(extends=PyNumExpr, module = "argus")]
pub struct Add;

#[pymethods]
impl Add {
    #[new]
    fn new(args: Vec<PyNumExpr>) -> (Self, PyNumExpr) {
        let args: Vec<NumExpr> = args.into_iter().map(|arg| *arg.0).collect();
        (Self, Box::new(NumExpr::Add(argus_core::expr::Add { args })).into())
    }
}

#[pyclass(extends=PyNumExpr, module = "argus")]
pub struct Sub;

#[pymethods]
impl Sub {
    #[new]
    fn new(lhs: PyNumExpr, rhs: PyNumExpr) -> (Self, PyNumExpr) {
        let lhs = lhs.0;
        let rhs = rhs.0;
        (Self, Box::new(NumExpr::Sub(argus_core::expr::Sub { lhs, rhs })).into())
    }
}

#[pyclass(extends=PyNumExpr, module = "argus")]
pub struct Mul;

#[pymethods]
impl Mul {
    #[new]
    fn new(args: Vec<PyNumExpr>) -> (Self, PyNumExpr) {
        let args: Vec<NumExpr> = args.into_iter().map(|arg| *arg.0).collect();
        (Self, Box::new(NumExpr::Mul(argus_core::expr::Mul { args })).into())
    }
}

#[pyclass(extends=PyNumExpr, module = "argus")]
pub struct Div;

#[pymethods]
impl Div {
    #[new]
    fn new(dividend: PyNumExpr, divisor: PyNumExpr) -> (Self, PyNumExpr) {
        let dividend = dividend.0;
        let divisor = divisor.0;
        (
            Self,
            Box::new(NumExpr::Div(argus_core::expr::Div { dividend, divisor })).into(),
        )
    }
}

#[pyclass(extends=PyNumExpr, module = "argus")]
pub struct Abs;

#[pymethods]
impl Abs {
    #[new]
    fn new(arg: PyNumExpr) -> (Self, PyNumExpr) {
        let arg = arg.0;
        (Self, Box::new(NumExpr::Abs(argus_core::expr::Abs { arg })).into())
    }
}

#[pyclass(name = "BoolExpr", subclass, module = "argus")]
#[derive(Debug, Clone, derive_more::From)]
pub struct PyBoolExpr(pub Box<BoolExpr>);

#[pymethods]
impl PyBoolExpr {
    fn __repr__(&self) -> String {
        format!("{:?}", &self.0)
    }

    fn __invert__(&self) -> PyResult<Py<Not>> {
        Python::with_gil(|py| Py::new(py, Not::new(self.clone())))
    }

    fn __or__(&self, other: &Self) -> PyResult<Py<Or>> {
        Python::with_gil(|py| Py::new(py, Or::new(vec![self.clone(), other.clone()])))
    }

    fn __and__(&self, other: &Self) -> PyResult<Py<And>> {
        Python::with_gil(|py| Py::new(py, And::new(vec![self.clone(), other.clone()])))
    }
}

#[pyclass(extends=PyBoolExpr, module = "argus")]
pub struct ConstBool;

#[pymethods]
impl ConstBool {
    #[new]
    fn new(val: bool) -> (Self, PyBoolExpr) {
        (Self, Box::new(BoolExpr::BoolLit(argus_core::expr::BoolLit(val))).into())
    }
}

#[pyclass(extends=PyBoolExpr, module = "argus")]
pub struct VarBool;

#[pymethods]
impl VarBool {
    #[new]
    fn new(name: String) -> (Self, PyBoolExpr) {
        (
            Self,
            Box::new(BoolExpr::BoolVar(argus_core::expr::BoolVar { name })).into(),
        )
    }
}

#[pyclass(extends=PyBoolExpr, module = "argus")]
pub struct Cmp;

#[pyclass(module = "argus")]
#[derive(Debug, Copy, Clone, derive_more::From)]
pub struct PyOrdering(Ordering);

impl Cmp {
    fn new(op: PyOrdering, lhs: PyNumExpr, rhs: PyNumExpr) -> (Self, PyBoolExpr) {
        let op = op.0;
        let lhs = lhs.0;
        let rhs = rhs.0;
        (
            Self,
            Box::new(BoolExpr::Cmp(argus_core::expr::Cmp { op, lhs, rhs })).into(),
        )
    }
}

#[pymethods]
impl Cmp {
    #[staticmethod]
    fn less_than(lhs: PyNumExpr, rhs: PyNumExpr) -> PyResult<Py<Self>> {
        Python::with_gil(|py| Py::new(py, Cmp::new(PyOrdering(Ordering::less_than()), lhs, rhs)))
    }

    #[staticmethod]
    fn less_than_eq(lhs: PyNumExpr, rhs: PyNumExpr) -> PyResult<Py<Self>> {
        Python::with_gil(|py| Py::new(py, Cmp::new(PyOrdering(Ordering::less_than_eq()), lhs, rhs)))
    }

    #[staticmethod]
    fn greater_than(lhs: PyNumExpr, rhs: PyNumExpr) -> PyResult<Py<Self>> {
        Python::with_gil(|py| Py::new(py, Cmp::new(PyOrdering(Ordering::greater_than()), lhs, rhs)))
    }

    #[staticmethod]
    fn greater_than_eq(lhs: PyNumExpr, rhs: PyNumExpr) -> PyResult<Py<Self>> {
        Python::with_gil(|py| Py::new(py, Cmp::new(PyOrdering(Ordering::greater_than_eq()), lhs, rhs)))
    }

    #[staticmethod]
    fn equal(lhs: PyNumExpr, rhs: PyNumExpr) -> PyResult<Py<Self>> {
        Python::with_gil(|py| Py::new(py, Cmp::new(PyOrdering(Ordering::equal()), lhs, rhs)))
    }

    #[staticmethod]
    fn not_equal(lhs: PyNumExpr, rhs: PyNumExpr) -> PyResult<Py<Self>> {
        Python::with_gil(|py| Py::new(py, Cmp::new(PyOrdering(Ordering::not_equal()), lhs, rhs)))
    }
}

#[pyclass(extends=PyBoolExpr, module = "argus")]
pub struct Not;

#[pymethods]
impl Not {
    #[new]
    fn new(arg: PyBoolExpr) -> (Self, PyBoolExpr) {
        let arg = arg.0;
        (Self, PyBoolExpr(Box::new(BoolExpr::Not(argus_core::expr::Not { arg }))))
    }
}

#[pyclass(extends=PyBoolExpr, module = "argus")]
pub struct And;

#[pymethods]
impl And {
    #[new]
    fn new(args: Vec<PyBoolExpr>) -> (Self, PyBoolExpr) {
        let args: Vec<BoolExpr> = args.into_iter().map(|arg| *arg.0).collect();
        (
            Self,
            PyBoolExpr(Box::new(BoolExpr::And(argus_core::expr::And { args }))),
        )
    }
}

#[pyclass(extends=PyBoolExpr, module = "argus")]
pub struct Or;

#[pymethods]
impl Or {
    #[new]
    fn new(args: Vec<PyBoolExpr>) -> (Self, PyBoolExpr) {
        let args: Vec<BoolExpr> = args.into_iter().map(|arg| *arg.0).collect();
        (Self, PyBoolExpr(Box::new(BoolExpr::Or(argus_core::expr::Or { args }))))
    }
}

#[pyclass(extends=PyBoolExpr, module = "argus")]
pub struct Next;

#[pymethods]
impl Next {
    #[new]
    fn new(arg: PyBoolExpr) -> (Self, PyBoolExpr) {
        let arg = arg.0;
        (
            Self,
            PyBoolExpr(Box::new(BoolExpr::Next(argus_core::expr::Next { arg }))),
        )
    }
}

#[pyclass(extends=PyBoolExpr, module = "argus")]
pub struct Always;

#[pymethods]
impl Always {
    #[new]
    #[pyo3(signature = (arg, *, interval=(None, None)))]
    fn new(arg: PyBoolExpr, interval: (Option<f64>, Option<f64>)) -> (Self, PyBoolExpr) {
        let arg = arg.0;
        let interval: Interval = match interval {
            (None, None) => (..).into(),
            (None, Some(b)) => (..Duration::from_secs_f64(b)).into(),
            (Some(a), None) => (Duration::from_secs_f64(a)..).into(),
            (Some(a), Some(b)) => (Duration::from_secs_f64(a)..Duration::from_secs_f64(b)).into(),
        };
        (
            Self,
            PyBoolExpr(Box::new(BoolExpr::Always(argus_core::expr::Always { arg, interval }))),
        )
    }
}

#[pyclass(extends=PyBoolExpr, module = "argus")]
pub struct Eventually;

#[pymethods]
impl Eventually {
    #[new]
    #[pyo3(signature = (arg, *, interval=(None, None)))]
    fn new(arg: PyBoolExpr, interval: (Option<f64>, Option<f64>)) -> (Self, PyBoolExpr) {
        let arg = arg.0;
        let interval: Interval = match interval {
            (None, None) => (..).into(),
            (None, Some(b)) => (..Duration::from_secs_f64(b)).into(),
            (Some(a), None) => (Duration::from_secs_f64(a)..).into(),
            (Some(a), Some(b)) => (Duration::from_secs_f64(a)..Duration::from_secs_f64(b)).into(),
        };
        (
            Self,
            PyBoolExpr(Box::new(BoolExpr::Eventually(argus_core::expr::Eventually {
                arg,
                interval,
            }))),
        )
    }
}

#[pyclass(extends=PyBoolExpr, module = "argus")]
pub struct Until;

#[pymethods]
impl Until {
    #[new]
    #[pyo3(signature = (lhs, rhs, *, interval=(None, None)))]
    fn new(lhs: PyBoolExpr, rhs: PyBoolExpr, interval: (Option<f64>, Option<f64>)) -> (Self, PyBoolExpr) {
        let lhs = lhs.0;
        let rhs = rhs.0;
        let interval: Interval = match interval {
            (None, None) => (..).into(),
            (None, Some(b)) => (..Duration::from_secs_f64(b)).into(),
            (Some(a), None) => (Duration::from_secs_f64(a)..).into(),
            (Some(a), Some(b)) => (Duration::from_secs_f64(a)..Duration::from_secs_f64(b)).into(),
        };
        (
            Self,
            PyBoolExpr(Box::new(BoolExpr::Until(argus_core::expr::Until {
                lhs,
                rhs,
                interval,
            }))),
        )
    }
}

pub fn init(_py: Python, m: &PyModule) -> PyResult<()> {
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
    m.add_class::<Abs>()?;

    m.add_class::<PyBoolExpr>()?;
    m.add_class::<ConstBool>()?;
    m.add_class::<VarBool>()?;
    m.add_class::<Cmp>()?;
    m.add_class::<Not>()?;
    m.add_class::<And>()?;
    m.add_class::<Or>()?;
    m.add_class::<Next>()?;
    m.add_class::<Always>()?;
    m.add_class::<Eventually>()?;
    m.add_class::<Until>()?;

    Ok(())
}
