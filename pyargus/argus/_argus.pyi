from typing import ClassVar, Protocol, TypeVar, final

from typing_extensions import Self

class NumExpr(Protocol):
    def __ge__(self, other) -> NumExpr: ...
    def __gt__(self, other) -> NumExpr: ...
    def __le__(self, other) -> NumExpr: ...
    def __lt__(self, other) -> NumExpr: ...
    def __mul__(self, other) -> NumExpr: ...
    def __eq__(self, other) -> NumExpr: ...  # type: ignore[override]
    def __ne__(self, other) -> NumExpr: ...  # type: ignore[override]
    def __neg__(self) -> NumExpr: ...
    def __add__(self, other) -> NumExpr: ...
    def __radd__(self, other) -> NumExpr: ...
    def __rmul__(self, other) -> NumExpr: ...
    def __sub__(self, other) -> NumExpr: ...
    def __rsub__(self, other) -> NumExpr: ...
    def __truediv__(self, other) -> NumExpr: ...
    def __rtruediv__(self, other) -> NumExpr: ...
    def __abs__(self) -> NumExpr: ...

@final
class ConstInt(NumExpr):
    def __init__(self, value: int): ...

@final
class ConstUInt(NumExpr):
    def __init__(self, value: int): ...

@final
class ConstFloat(NumExpr):
    def __init__(self, value: float): ...

@final
class VarInt(NumExpr):
    def __init__(self, name: str): ...

@final
class VarUInt(NumExpr):
    def __init__(self, name: str): ...

@final
class VarFloat(NumExpr):
    def __init__(self, name: str): ...

@final
class Negate(NumExpr):
    def __init__(self, arg: NumExpr): ...

@final
class Add(NumExpr):
    def __init__(self, args: list[NumExpr]): ...

@final
class Mul(NumExpr):
    def __init__(self, args: list[NumExpr]): ...

@final
class Div(NumExpr):
    def __init__(self, dividend: NumExpr, divisor: NumExpr): ...

@final
class Abs(NumExpr):
    def __init__(self, arg: NumExpr): ...

class BoolExpr(Protocol):
    def __and__(self, other) -> BoolExpr: ...
    def __invert__(self) -> BoolExpr: ...
    def __or__(self, other) -> BoolExpr: ...
    def __rand__(self, other) -> BoolExpr: ...
    def __ror__(self, other) -> BoolExpr: ...

@final
class ConstBool(BoolExpr):
    def __init__(self, value: bool): ...

@final
class VarBool(BoolExpr):
    def __init__(self, name: str): ...

@final
class Cmp(BoolExpr):
    @staticmethod
    def equal(lhs: NumExpr, rhs: NumExpr) -> Cmp: ...
    @staticmethod
    def greater_than(lhs: NumExpr, rhs: NumExpr) -> Cmp: ...
    @staticmethod
    def greater_than_eq(lhs: NumExpr, rhs: NumExpr) -> Cmp: ...
    @staticmethod
    def less_than(lhs: NumExpr, rhs: NumExpr) -> Cmp: ...
    @staticmethod
    def less_than_eq(lhs: NumExpr, rhs: NumExpr) -> Cmp: ...
    @staticmethod
    def not_equal(lhs: NumExpr, rhs: NumExpr) -> Cmp: ...

@final
class Not(BoolExpr):
    def __init__(self, arg: BoolExpr): ...

@final
class And(BoolExpr):
    def __init__(self, args: list[BoolExpr]): ...

@final
class Or(BoolExpr):
    def __init__(self, args: list[BoolExpr]): ...

@final
class Next(BoolExpr):
    def __init__(self, arg: BoolExpr): ...

@final
class Always(BoolExpr):
    def __init__(self, arg: BoolExpr): ...

@final
class Eventually(BoolExpr):
    def __init__(self, arg: BoolExpr): ...

@final
class Until(BoolExpr):
    def __init__(self, lhs: BoolExpr, rhs: BoolExpr): ...

@final
class DType:
    Bool: ClassVar[DType] = ...
    Float: ClassVar[DType] = ...
    Int: ClassVar[DType] = ...
    UnsignedInt: ClassVar[DType] = ...

    @classmethod
    def convert(cls, dtype: type[bool | int | float] | Self) -> Self: ...  # noqa: Y041
    def __eq__(self, other) -> bool: ...
    def __int__(self) -> int: ...

_SignalKind = TypeVar("_SignalKind", bool, int, float, covariant=True)

class Signal(Protocol[_SignalKind]):
    def is_empty(self) -> bool: ...
    @property
    def start_time(self) -> float | None: ...
    @property
    def end_time(self) -> float | None: ...
    @property
    def kind(self) -> type[bool | int | float]: ...

@final
class BoolSignal(Signal[bool]):
    @classmethod
    def constant(cls, value: bool) -> Self: ...
    @classmethod
    def from_samples(cls, samples: list[tuple[float, bool]]) -> Self: ...
    def push(self, time: float, value: bool): ...
    def at(self, time: float) -> _SignalKind | None: ...

@final
class IntSignal(Signal[int]):
    @classmethod
    def constant(cls, value: int) -> Self: ...
    @classmethod
    def from_samples(cls, samples: list[tuple[float, int]]) -> Self: ...
    def push(self, time: float, value: int): ...
    def at(self, time: float) -> int | None: ...

@final
class UnsignedIntSignal(Signal[int]):
    @classmethod
    def constant(cls, value: int) -> Self: ...
    @classmethod
    def from_samples(cls, samples: list[tuple[float, int]]) -> Self: ...
    def push(self, time: float, value: int): ...
    def at(self, time: float) -> int | None: ...

@final
class FloatSignal(Signal[float]):
    @classmethod
    def constant(cls, value: float) -> Self: ...
    @classmethod
    def from_samples(cls, samples: list[tuple[float, float]]) -> Self: ...
    def push(self, time: float, value: float): ...
    def at(self, time: float) -> float | None: ...

@final
class Trace: ...

def eval_bool_semantics(expr: BoolExpr, trace: Trace) -> BoolSignal: ...
def eval_robust_semantics(expr: BoolExpr, trace: Trace) -> BoolSignal: ...
