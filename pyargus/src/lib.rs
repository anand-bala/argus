use argus_core::expr::Ordering;
use argus_core::prelude::*;
use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;

#[pyclass(name = "NumExpr", subclass)]
#[derive(Clone, derive_more::From)]
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

#[pyclass(extends=PyNumExpr)]
struct ConstInt;

#[pymethods]
impl ConstInt {
    #[new]
    fn new(val: i64) -> (Self, PyNumExpr) {
        (Self, Box::new(NumExpr::IntLit(val)).into())
    }
}

#[pyclass(extends=PyNumExpr)]
struct ConstUInt;

#[pymethods]
impl ConstUInt {
    #[new]
    fn new(val: u64) -> (Self, PyNumExpr) {
        (Self, Box::new(NumExpr::UIntLit(val)).into())
    }
}

#[pyclass(extends=PyNumExpr)]
struct ConstFloat;

#[pymethods]
impl ConstFloat {
    #[new]
    fn new(val: f64) -> (Self, PyNumExpr) {
        (Self, Box::new(NumExpr::FloatLit(val)).into())
    }
}

#[pyclass(extends=PyNumExpr)]
struct VarInt;

#[pymethods]
impl VarInt {
    #[new]
    fn new(name: String) -> (Self, PyNumExpr) {
        (Self, Box::new(NumExpr::IntVar { name }).into())
    }
}

#[pyclass(extends=PyNumExpr)]
struct VarUInt;

#[pymethods]
impl VarUInt {
    #[new]
    fn new(name: String) -> (Self, PyNumExpr) {
        (Self, Box::new(NumExpr::UIntVar { name }).into())
    }
}

#[pyclass(extends=PyNumExpr)]
struct VarFloat;

#[pymethods]
impl VarFloat {
    #[new]
    fn new(name: String) -> (Self, PyNumExpr) {
        (Self, Box::new(NumExpr::FloatVar { name }).into())
    }
}

#[pyclass(extends=PyNumExpr)]
struct Negate;

#[pymethods]
impl Negate {
    #[new]
    fn new(arg: PyNumExpr) -> (Self, PyNumExpr) {
        let arg = arg.0;
        (Self, Box::new(NumExpr::Neg { arg }).into())
    }
}

#[pyclass(extends=PyNumExpr)]
struct Add;

#[pymethods]
impl Add {
    #[new]
    fn new(args: Vec<PyNumExpr>) -> (Self, PyNumExpr) {
        let args: Vec<NumExpr> = args.into_iter().map(|arg| *arg.0).collect();
        (Self, Box::new(NumExpr::Add { args }).into())
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
        (Self, Box::new(NumExpr::Sub { lhs, rhs }).into())
    }
}

#[pyclass(extends=PyNumExpr)]
struct Mul;

#[pymethods]
impl Mul {
    #[new]
    fn new(args: Vec<PyNumExpr>) -> (Self, PyNumExpr) {
        let args: Vec<NumExpr> = args.into_iter().map(|arg| *arg.0).collect();
        (Self, Box::new(NumExpr::Mul { args }).into())
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
        (Self, Box::new(NumExpr::Div { dividend, divisor }).into())
    }
}

#[pyclass(extends=PyNumExpr)]
struct Abs;

#[pymethods]
impl Abs {
    #[new]
    fn new(arg: PyNumExpr) -> (Self, PyNumExpr) {
        let arg = arg.0;
        (Self, Box::new(NumExpr::Abs { arg }).into())
    }
}

#[pyclass(name = "BoolExpr", subclass)]
#[derive(Clone, derive_more::From)]
struct PyBoolExpr(Box<BoolExpr>);

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

#[pyclass(extends=PyBoolExpr)]
struct ConstBool;

#[pymethods]
impl ConstBool {
    #[new]
    fn new(val: bool) -> (Self, PyBoolExpr) {
        (Self, Box::new(BoolExpr::BoolLit(val)).into())
    }
}

#[pyclass(extends=PyBoolExpr)]
struct VarBool;

#[pymethods]
impl VarBool {
    #[new]
    fn new(name: String) -> (Self, PyBoolExpr) {
        (Self, Box::new(BoolExpr::BoolVar { name }).into())
    }
}

#[pyclass(extends=PyBoolExpr)]
struct Cmp;

#[pyclass]
#[derive(Copy, Clone, derive_more::From)]
struct PyOrdering(Ordering);

#[pymethods]
impl Cmp {
    #[new]
    fn new(op: PyOrdering, lhs: PyNumExpr, rhs: PyNumExpr) -> (Self, PyBoolExpr) {
        let op = op.0;
        let lhs = lhs.0;
        let rhs = rhs.0;
        (Self, Box::new(BoolExpr::Cmp { op, lhs, rhs }).into())
    }

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

#[pyclass(extends=PyBoolExpr)]
struct Not;

#[pymethods]
impl Not {
    #[new]
    fn new(arg: PyBoolExpr) -> (Self, PyBoolExpr) {
        let arg = arg.0;
        (Self, PyBoolExpr(Box::new(BoolExpr::Not { arg })))
    }
}

#[pyclass(extends=PyBoolExpr)]
struct And;

#[pymethods]
impl And {
    #[new]
    fn new(args: Vec<PyBoolExpr>) -> (Self, PyBoolExpr) {
        let args: Vec<BoolExpr> = args.into_iter().map(|arg| *arg.0).collect();
        (Self, PyBoolExpr(Box::new(BoolExpr::And { args })))
    }
}

#[pyclass(extends=PyBoolExpr)]
struct Or;

#[pymethods]
impl Or {
    #[new]
    fn new(args: Vec<PyBoolExpr>) -> (Self, PyBoolExpr) {
        let args: Vec<BoolExpr> = args.into_iter().map(|arg| *arg.0).collect();
        (Self, PyBoolExpr(Box::new(BoolExpr::Or { args })))
    }
}

#[pyclass(extends=PyBoolExpr)]
struct Next;

#[pymethods]
impl Next {
    #[new]
    fn new(arg: PyBoolExpr) -> (Self, PyBoolExpr) {
        let arg = arg.0;
        (Self, PyBoolExpr(Box::new(BoolExpr::Next { arg })))
    }
}

#[pyclass(extends=PyBoolExpr)]
struct Always;

#[pymethods]
impl Always {
    #[new]
    fn new(arg: PyBoolExpr) -> (Self, PyBoolExpr) {
        let arg = arg.0;
        (Self, PyBoolExpr(Box::new(BoolExpr::Always { arg })))
    }
}

#[pyclass(extends=PyBoolExpr)]
struct Eventually;

#[pymethods]
impl Eventually {
    #[new]
    fn new(arg: PyBoolExpr) -> (Self, PyBoolExpr) {
        let arg = arg.0;
        (Self, PyBoolExpr(Box::new(BoolExpr::Eventually { arg })))
    }
}

#[pyclass(extends=PyBoolExpr)]
struct Until;

#[pymethods]
impl Until {
    #[new]
    fn new(lhs: PyBoolExpr, rhs: PyBoolExpr) -> (Self, PyBoolExpr) {
        let lhs = lhs.0;
        let rhs = rhs.0;
        (Self, PyBoolExpr(Box::new(BoolExpr::Until { lhs, rhs })))
    }
}

#[pymodule]
#[pyo3(name = "_argus")]
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
