from __future__ import annotations

from typing import Optional, Type, Union

from argus import _argus
from argus._argus import DType as DType
from argus.exprs import ConstBool, ConstFloat, ConstInt, ConstUInt, VarBool, VarFloat, VarInt, VarUInt
from argus.signals import BoolSignal, FloatSignal, IntSignal, Signal, UnsignedIntSignal

try:
    __doc__ = _argus.__doc__
except AttributeError:
    ...


def declare_var(name: str, dtype: Union[DType, Type[Union[bool, int, float]]]) -> Union[VarBool, VarInt, VarUInt, VarFloat]:
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


def literal(value: Union[bool, int, float]) -> Union[ConstBool, ConstInt, ConstUInt, ConstFloat]:
    """Create a literal expression for the given value"""
    if isinstance(value, bool):
        return ConstBool(value)
    if isinstance(value, int):
        return ConstInt(value)
    if isinstance(value, float):
        return ConstFloat(value)
    raise TypeError(f"unsupported literal type `{type(value)}`")


def signal(
    dtype: Union[DType, Type[Union[bool, int, float]]],
    *,
    value: Optional[Union[bool, int, float]] = None,
) -> Signal:
    """Create a signal of the given type

    If a `value` isn't given the signal created is assumed to be a sampled signal, i.e., new data points can be pushed to the
    signal. Otherwise, the signal is constant with the given value.
    """
    if isinstance(dtype, type):
        if dtype == bool:
            dtype = DType.Bool
        elif dtype == int:
            dtype = DType.Int
        elif dtype == float:
            dtype = DType.Float

    if dtype == DType.Bool:
        if value is None:
            return BoolSignal.from_samples([])
        else:
            assert isinstance(value, bool)
            return BoolSignal.constant(value)
    elif dtype == DType.Int:
        if value is None:
            return IntSignal.from_samples([])
        else:
            assert isinstance(value, int)
            return IntSignal.constant(value)
    elif dtype == DType.UnsignedInt:
        if value is None:
            return UnsignedIntSignal.from_samples([])
        else:
            assert isinstance(value, int)
            return UnsignedIntSignal.constant(value)
    elif dtype == DType.Float:
        return FloatSignal.from_samples([])
    raise TypeError(f"unsupported signal type `{dtype}`")


__all__ = [
    "DType",
    "declare_var",
    "literal",
    "signal",
]
