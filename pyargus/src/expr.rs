use std::time::Duration;

use argus::expr::*;
use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;

/// A base expression
///
/// This is an abstract base class that provides an interface for all specific
#[pyclass(name = "Expr", subclass, module = "argus")]
#[derive(Debug, Clone)]
pub struct PyExpr;

impl PyExpr {
    pub fn from_expr(py: Python, expr: Expr) -> PyResult<PyObject> {
        match expr {
            Expr::Bool(e) => PyBoolExpr::from_expr(py, e),
            Expr::Num(e) => PyNumExpr::from_expr(py, e),
        }
    }
}

/// A base numeric expression
///
/// This is an abstract base class that provides an interface for all numeric
/// expressions supported in Argus (literals, arithmetic, and so on).
#[pyclass(name = "NumExpr", subclass, extends = PyExpr, module = "argus")]
#[derive(Debug, Clone, derive_more::From)]
pub struct PyNumExpr(pub Box<NumExpr>);

macro_rules! make_expr {
    ($py:expr, $expr:expr, $subclass:expr) => {
        PyCell::new(
            $py,
            PyClassInitializer::from(PyExpr)
                .add_subclass(Self(Box::new($expr)))
                .add_subclass($subclass),
        )
        .map(|obj| obj.to_object($py))
    };
}

impl PyNumExpr {
    pub fn from_expr(py: Python, expr: NumExpr) -> PyResult<PyObject> {
        match expr {
            NumExpr::IntLit(_) => make_expr!(py, expr, ConstInt),
            NumExpr::UIntLit(_) => make_expr!(py, expr, ConstUInt),
            NumExpr::FloatLit(_) => make_expr!(py, expr, ConstFloat),
            NumExpr::IntVar(_) => make_expr!(py, expr, VarInt),
            NumExpr::UIntVar(_) => make_expr!(py, expr, VarUInt),
            NumExpr::FloatVar(_) => make_expr!(py, expr, VarFloat),
            NumExpr::Neg(_) => make_expr!(py, expr, Negate),
            NumExpr::Add(_) => make_expr!(py, expr, Add),
            NumExpr::Sub(_) => make_expr!(py, expr, Sub),
            NumExpr::Mul(_) => make_expr!(py, expr, Mul),
            NumExpr::Div(_) => make_expr!(py, expr, Div),
            NumExpr::Abs(_) => make_expr!(py, expr, Abs),
        }
    }
}

#[pymethods]
impl PyNumExpr {
    fn __repr__(&self) -> String {
        format!("{:?}", &self.0)
    }

    fn __str__(&self) -> String {
        format!("{}", self.0)
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
    fn new(val: i64) -> PyClassInitializer<Self> {
        PyClassInitializer::from(PyExpr)
            .add_subclass(Box::new(NumExpr::IntLit(argus::expr::IntLit(val))).into())
            .add_subclass(Self)
    }
}

/// Create a constant *unsigned* integer expression
///
/// # Warning
///
/// Negating an unsigned integer during evaluation *may* lead to the evaluation method
/// panicking.
#[pyclass(extends=PyNumExpr, module = "argus")]
pub struct ConstUInt;

#[pymethods]
impl ConstUInt {
    #[new]
    fn new(val: u64) -> PyClassInitializer<Self> {
        PyClassInitializer::from(PyExpr)
            .add_subclass(Box::new(NumExpr::UIntLit(argus::expr::UIntLit(val))).into())
            .add_subclass(Self)
    }
}

/// Create a constant floating point number expression.
#[pyclass(extends=PyNumExpr, module = "argus")]
pub struct ConstFloat;

#[pymethods]
impl ConstFloat {
    #[new]
    fn new(val: f64) -> PyClassInitializer<ConstFloat> {
        PyClassInitializer::from(PyExpr)
            .add_subclass(Box::new(NumExpr::FloatLit(argus::expr::FloatLit(val))).into())
            .add_subclass(Self)
    }
}

/// Create a integer variable
#[pyclass(extends=PyNumExpr, module = "argus")]
pub struct VarInt;

#[pymethods]
impl VarInt {
    #[new]
    fn new(name: String) -> PyClassInitializer<Self> {
        PyClassInitializer::from(PyExpr)
            .add_subclass(Box::new(NumExpr::IntVar(argus::expr::IntVar { name })).into())
            .add_subclass(Self)
    }
}

/// Create an *unsigned* integer variable
#[pyclass(extends=PyNumExpr, module = "argus")]
pub struct VarUInt;

#[pymethods]
impl VarUInt {
    #[new]
    fn new(name: String) -> PyClassInitializer<Self> {
        PyClassInitializer::from(PyExpr)
            .add_subclass(Box::new(NumExpr::UIntVar(argus::expr::UIntVar { name })).into())
            .add_subclass(Self)
    }
}

/// Create a float variable
#[pyclass(extends=PyNumExpr, module = "argus")]
pub struct VarFloat;

#[pymethods]
impl VarFloat {
    #[new]
    fn new(name: String) -> PyClassInitializer<Self> {
        PyClassInitializer::from(PyExpr)
            .add_subclass(Box::new(NumExpr::FloatVar(argus::expr::FloatVar { name })).into())
            .add_subclass(Self)
    }
}

/// Create a numeric negation expression
#[pyclass(extends=PyNumExpr, module = "argus")]
pub struct Negate;

#[pymethods]
impl Negate {
    #[new]
    fn new(arg: PyNumExpr) -> PyClassInitializer<Self> {
        let arg = arg.0;
        PyClassInitializer::from(PyExpr)
            .add_subclass(Box::new(NumExpr::Neg(argus::expr::Neg { arg })).into())
            .add_subclass(Self)
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
    fn new(args: Vec<PyNumExpr>) -> PyClassInitializer<Self> {
        let args: Vec<NumExpr> = args.into_iter().map(|arg| *arg.0).collect();
        PyClassInitializer::from(PyExpr)
            .add_subclass(Box::new(NumExpr::Add(argus::expr::Add { args })).into())
            .add_subclass(Self)
    }
}

#[pyclass(extends=PyNumExpr, module = "argus")]
pub struct Sub;

#[pymethods]
impl Sub {
    #[new]
    fn new(lhs: PyNumExpr, rhs: PyNumExpr) -> PyClassInitializer<Self> {
        let lhs = lhs.0;
        let rhs = rhs.0;
        PyClassInitializer::from(PyExpr)
            .add_subclass(Box::new(NumExpr::Sub(argus::expr::Sub { lhs, rhs })).into())
            .add_subclass(Self)
    }
}

#[pyclass(extends=PyNumExpr, module = "argus")]
pub struct Mul;

#[pymethods]
impl Mul {
    #[new]
    fn new(args: Vec<PyNumExpr>) -> PyClassInitializer<Self> {
        let args: Vec<NumExpr> = args.into_iter().map(|arg| *arg.0).collect();
        PyClassInitializer::from(PyExpr)
            .add_subclass(Box::new(NumExpr::Mul(argus::expr::Mul { args })).into())
            .add_subclass(Self)
    }
}

#[pyclass(extends=PyNumExpr, module = "argus")]
pub struct Div;

#[pymethods]
impl Div {
    #[new]
    fn new(dividend: PyNumExpr, divisor: PyNumExpr) -> PyClassInitializer<Self> {
        let dividend = dividend.0;
        let divisor = divisor.0;
        PyClassInitializer::from(PyExpr)
            .add_subclass(Box::new(NumExpr::Div(argus::expr::Div { dividend, divisor })).into())
            .add_subclass(Self)
    }
}

#[pyclass(extends=PyNumExpr, module = "argus")]
pub struct Abs;

#[pymethods]
impl Abs {
    #[new]
    fn new(arg: PyNumExpr) -> PyClassInitializer<Self> {
        let arg = arg.0;
        PyClassInitializer::from(PyExpr)
            .add_subclass(Box::new(NumExpr::Abs(argus::expr::Abs { arg })).into())
            .add_subclass(Self)
    }
}

#[pyclass(name = "BoolExpr", subclass, extends=PyExpr, module = "argus")]
#[derive(Debug, Clone, derive_more::From)]
pub struct PyBoolExpr(pub Box<BoolExpr>);

impl PyBoolExpr {
    pub fn from_expr(py: Python, expr: BoolExpr) -> PyResult<PyObject> {
        match expr {
            BoolExpr::BoolLit(_) => make_expr!(py, expr, ConstBool),
            BoolExpr::BoolVar(_) => make_expr!(py, expr, VarBool),
            BoolExpr::Cmp(_) => make_expr!(py, expr, Cmp),
            BoolExpr::Not(_) => make_expr!(py, expr, Not),
            BoolExpr::And(_) => make_expr!(py, expr, And),
            BoolExpr::Or(_) => make_expr!(py, expr, Or),
            BoolExpr::Next(_) => make_expr!(py, expr, Next),
            BoolExpr::Oracle(_) => make_expr!(py, expr, Oracle),
            BoolExpr::Always(_) => make_expr!(py, expr, Always),
            BoolExpr::Eventually(_) => make_expr!(py, expr, Eventually),
            BoolExpr::Until(_) => make_expr!(py, expr, Until),
        }
    }
}

#[pymethods]
impl PyBoolExpr {
    fn __repr__(&self) -> String {
        format!("{:?}", &self.0)
    }

    fn __str__(&self) -> String {
        format!("{}", self.0)
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
    fn new(val: bool) -> PyClassInitializer<Self> {
        PyClassInitializer::from(PyExpr)
            .add_subclass(Box::new(BoolExpr::BoolLit(argus::expr::BoolLit(val))).into())
            .add_subclass(Self)
    }
}

#[pyclass(extends=PyBoolExpr, module = "argus")]
pub struct VarBool;

#[pymethods]
impl VarBool {
    #[new]
    fn new(name: String) -> PyClassInitializer<Self> {
        PyClassInitializer::from(PyExpr)
            .add_subclass(Box::new(BoolExpr::BoolVar(argus::expr::BoolVar { name })).into())
            .add_subclass(Self)
    }
}

#[pyclass(extends=PyBoolExpr, module = "argus")]
pub struct Cmp;

#[pyclass(module = "argus")]
#[derive(Debug, Copy, Clone, derive_more::From)]
pub struct PyOrdering(Ordering);

impl Cmp {
    fn new(op: PyOrdering, lhs: PyNumExpr, rhs: PyNumExpr) -> PyClassInitializer<Self> {
        let op = op.0;
        let lhs = lhs.0;
        let rhs = rhs.0;
        PyClassInitializer::from(PyExpr)
            .add_subclass(Box::new(BoolExpr::Cmp(argus::expr::Cmp { op, lhs, rhs })).into())
            .add_subclass(Self)
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
    fn new(arg: PyBoolExpr) -> PyClassInitializer<Self> {
        let arg = arg.0;
        PyClassInitializer::from(PyExpr)
            .add_subclass(PyBoolExpr(Box::new(BoolExpr::Not(argus::expr::Not { arg }))))
            .add_subclass(Self)
    }
}

#[pyclass(extends=PyBoolExpr, module = "argus")]
pub struct And;

#[pymethods]
impl And {
    #[new]
    fn new(args: Vec<PyBoolExpr>) -> PyClassInitializer<Self> {
        let args: Vec<BoolExpr> = args.into_iter().map(|arg| *arg.0).collect();
        PyClassInitializer::from(PyExpr)
            .add_subclass(PyBoolExpr(Box::new(BoolExpr::And(argus::expr::And { args }))))
            .add_subclass(Self)
    }
}

#[pyclass(extends=PyBoolExpr, module = "argus")]
pub struct Or;

#[pymethods]
impl Or {
    #[new]
    fn new(args: Vec<PyBoolExpr>) -> PyClassInitializer<Self> {
        let args: Vec<BoolExpr> = args.into_iter().map(|arg| *arg.0).collect();
        PyClassInitializer::from(PyExpr)
            .add_subclass(PyBoolExpr(Box::new(BoolExpr::Or(argus::expr::Or { args }))))
            .add_subclass(Self)
    }
}

#[pyclass(extends=PyBoolExpr, module = "argus")]
pub struct Next;

#[pymethods]
impl Next {
    #[new]
    fn new(arg: PyBoolExpr) -> PyClassInitializer<Self> {
        let arg = arg.0;
        PyClassInitializer::from(PyExpr)
            .add_subclass(PyBoolExpr(Box::new(BoolExpr::Next(argus::expr::Next { arg }))))
            .add_subclass(Self)
    }
}

#[pyclass(extends=PyBoolExpr, module = "argus")]
pub struct Oracle;

#[pymethods]
impl Oracle {
    #[new]
    fn new(arg: PyBoolExpr, steps: usize) -> PyClassInitializer<Self> {
        let arg = arg.0;
        PyClassInitializer::from(PyExpr)
            .add_subclass(PyBoolExpr(Box::new(BoolExpr::Oracle(argus::expr::Oracle {
                arg,
                steps,
            }))))
            .add_subclass(Self)
    }
}

#[pyclass(extends=PyBoolExpr, module = "argus")]
pub struct Always;

#[pymethods]
impl Always {
    #[new]
    #[pyo3(signature = (arg, *, interval=(None, None)))]
    fn new(arg: PyBoolExpr, interval: (Option<f64>, Option<f64>)) -> PyClassInitializer<Self> {
        let arg = arg.0;
        let interval: Interval = match interval {
            (None, None) => (..).into(),
            (None, Some(b)) => (..Duration::from_secs_f64(b)).into(),
            (Some(a), None) => (Duration::from_secs_f64(a)..).into(),
            (Some(a), Some(b)) => (Duration::from_secs_f64(a)..Duration::from_secs_f64(b)).into(),
        };
        PyClassInitializer::from(PyExpr)
            .add_subclass(PyBoolExpr(Box::new(BoolExpr::Always(argus::expr::Always {
                arg,
                interval,
            }))))
            .add_subclass(Self)
    }
}

#[pyclass(extends=PyBoolExpr, module = "argus")]
pub struct Eventually;

#[pymethods]
impl Eventually {
    #[new]
    #[pyo3(signature = (arg, *, interval=(None, None)))]
    fn new(arg: PyBoolExpr, interval: (Option<f64>, Option<f64>)) -> PyClassInitializer<Self> {
        let arg = arg.0;
        let interval: Interval = match interval {
            (None, None) => (..).into(),
            (None, Some(b)) => (..Duration::from_secs_f64(b)).into(),
            (Some(a), None) => (Duration::from_secs_f64(a)..).into(),
            (Some(a), Some(b)) => (Duration::from_secs_f64(a)..Duration::from_secs_f64(b)).into(),
        };
        PyClassInitializer::from(PyExpr)
            .add_subclass(PyBoolExpr(Box::new(BoolExpr::Eventually(argus::expr::Eventually {
                arg,
                interval,
            }))))
            .add_subclass(Self)
    }
}

#[pyclass(extends=PyBoolExpr, module = "argus")]
pub struct Until;

#[pymethods]
impl Until {
    #[new]
    #[pyo3(signature = (lhs, rhs, *, interval=(None, None)))]
    fn new(lhs: PyBoolExpr, rhs: PyBoolExpr, interval: (Option<f64>, Option<f64>)) -> PyClassInitializer<Self> {
        let lhs = lhs.0;
        let rhs = rhs.0;
        let interval: Interval = match interval {
            (None, None) => (..).into(),
            (None, Some(b)) => (..Duration::from_secs_f64(b)).into(),
            (Some(a), None) => (Duration::from_secs_f64(a)..).into(),
            (Some(a), Some(b)) => (Duration::from_secs_f64(a)..Duration::from_secs_f64(b)).into(),
        };
        PyClassInitializer::from(PyExpr)
            .add_subclass(PyBoolExpr(Box::new(BoolExpr::Until(argus::expr::Until {
                lhs,
                rhs,
                interval,
            }))))
            .add_subclass(Self)
    }
}

pub fn init(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyExpr>()?;

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
