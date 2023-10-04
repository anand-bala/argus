from __future__ import annotations

from typing import List, Optional, Tuple, Type, Union

from . import _argus
from ._argus import dtype, parse_expr
from .exprs import ConstBool, ConstFloat, ConstInt, ConstUInt, VarBool, VarFloat, VarInt, VarUInt
from .signals import BoolSignal, FloatSignal, IntSignal, Signal, UnsignedIntSignal

try:
    __doc__ = _argus.__doc__
except AttributeError:
    ...

AllowedDtype = Union[bool, int, float]


def declare_var(name: str, dtype_: Union[dtype, Type[AllowedDtype]]) -> Union[VarBool, VarInt, VarUInt, VarFloat]:
    """Declare a variable with the given name and type"""
    dtype_ = dtype.convert(dtype_)

    if dtype_ == dtype.bool_:
        return VarBool(name)
    elif dtype_ == dtype.int64:
        return VarInt(name)
    elif dtype_ == dtype.uint64:
        return VarUInt(name)
    elif dtype_ == dtype.float64:
        return VarFloat(name)
    raise TypeError(f"unsupported variable type `{dtype_}`")


def literal(value: AllowedDtype) -> Union[ConstBool, ConstInt, ConstUInt, ConstFloat]:
    """Create a literal expression for the given value"""
    if isinstance(value, bool):
        return ConstBool(value)
    if isinstance(value, int):
        return ConstInt(value)
    if isinstance(value, float):
        return ConstFloat(value)
    raise TypeError(f"unsupported literal type `{type(value)}`")


def signal(
    dtype_: Union[dtype, Type[AllowedDtype]],
    *,
    data: Optional[Union[AllowedDtype, List[Tuple[float, AllowedDtype]]]] = None,
) -> Union[BoolSignal, UnsignedIntSignal, IntSignal, FloatSignal]:
    """Create a signal of the given type

    Parameters
    ----------

    dtype_:
        Type of the signal

    data :
        If a constant scalar is given, a constant signal is created. Otherwise, if a list of sample points are given, a sampled
        signal is constructed. Otherwise, an empty signal is created.
    """
    factory: Type[Union[BoolSignal, UnsignedIntSignal, IntSignal, FloatSignal]]
    expected_type: Type[AllowedDtype]

    dtype_ = dtype.convert(dtype_)
    if dtype_ == dtype.bool_:
        factory = BoolSignal
        expected_type = bool
    elif dtype_ == dtype.uint64:
        factory = UnsignedIntSignal
        expected_type = int
    elif dtype_ == dtype.int64:
        factory = IntSignal
        expected_type = int
    elif dtype_ == dtype.float64:
        factory = FloatSignal
        expected_type = float
    else:
        raise ValueError(f"unsupported dtype_ {dtype}")

    if data is None:
        return factory.from_samples([])
    elif isinstance(data, (list, tuple)):
        return factory.from_samples(data)  # type: ignore[arg-type]
    assert isinstance(data, expected_type)
    return factory.constant(data)  # type: ignore[arg-type]


__all__ = [
    "dtype",
    "parse_expr",
    "declare_var",
    "literal",
    "signal",
    "Signal",
]
