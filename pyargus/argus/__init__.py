from __future__ import annotations

from typing import List, Optional, Tuple, Type, Union

from argus import _argus
from argus._argus import DType as DType
from argus.exprs import ConstBool, ConstFloat, ConstInt, ConstUInt, VarBool, VarFloat, VarInt, VarUInt
from argus.signals import BoolSignal, FloatSignal, IntSignal, Signal, UnsignedIntSignal

try:
    __doc__ = _argus.__doc__
except AttributeError:
    ...

AllowedDtype = Union[bool, int, float]


def declare_var(name: str, dtype: Union[DType, Type[AllowedDtype]]) -> Union[VarBool, VarInt, VarUInt, VarFloat]:
    """Declare a variable with the given name and type"""
    if isinstance(dtype, type):
        if dtype == bool:
            dtype = DType.Bool
        elif dtype == int:
            dtype = DType.Int
        elif dtype == float:
            dtype = DType.Float

    if dtype == DType.Bool:
        return VarBool(name)
    elif dtype == DType.Int:
        return VarInt(name)
    elif dtype == DType.UnsignedInt:
        return VarUInt(name)
    elif dtype == DType.Float:
        return VarFloat(name)
    raise TypeError(f"unsupported variable type `{dtype}`")


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
    dtype: Union[DType, Type[AllowedDtype]],
    *,
    data: Optional[Union[AllowedDtype, List[Tuple[float, AllowedDtype]]]] = None,
) -> Union[BoolSignal, UnsignedIntSignal, IntSignal, FloatSignal]:
    """Create a signal of the given type

    Parameters
    ----------

    dtype:
        Type of the signal

    data :
        If a constant scalar is given, a constant signal is created. Otherwise, if a list of sample points are given, a sampled
        signal is constructed. Otherwise, an empty signal is created.
    """
    factory: Type[Union[BoolSignal, UnsignedIntSignal, IntSignal, FloatSignal]]
    expected_type: Type[AllowedDtype]

    dtype = DType.convert(dtype)
    if dtype == DType.Bool:
        factory = BoolSignal
        expected_type = bool
    elif dtype == DType.UnsignedInt:
        factory = UnsignedIntSignal
        expected_type = int
    elif dtype == DType.Int:
        factory = IntSignal
        expected_type = int
    elif dtype == DType.Float:
        factory = FloatSignal
        expected_type = float
    else:
        raise ValueError(f"unsupported dtype {dtype}")

    if data is None:
        return factory.from_samples([])
    elif isinstance(data, (list, tuple)):
        return factory.from_samples(data)  # type: ignore[arg-type]
    assert isinstance(data, expected_type)
    return factory.constant(data)  # type: ignore[arg-type]


__all__ = [
    "DType",
    "declare_var",
    "literal",
    "signal",
    "Signal",
]
