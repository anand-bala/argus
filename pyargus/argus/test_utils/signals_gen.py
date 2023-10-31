import typing
from typing import List, Tuple, Type, Union

from hypothesis import strategies as st
from hypothesis.strategies import SearchStrategy, composite

import argus
from argus import AllowedDtype, dtype


def gen_element_fn(dtype_: Union[Type[AllowedDtype], dtype]) -> SearchStrategy[AllowedDtype]:
    new_dtype = dtype.convert(dtype_)
    if new_dtype == dtype.bool_:
        return st.booleans()
    elif new_dtype == dtype.int64:
        size = 2**64
        return st.integers(min_value=(-size // 2), max_value=((size - 1) // 2))
    elif new_dtype == dtype.uint64:
        size = 2**64
        return st.integers(min_value=0, max_value=(size - 1))
    elif new_dtype == dtype.float64:
        return st.floats(
            width=64,
            allow_nan=False,
            allow_infinity=False,
            allow_subnormal=False,
        )
    else:
        raise ValueError(f"invalid dtype {dtype_}")


@composite
def gen_samples(
    draw: st.DrawFn,
    min_size: int,
    max_size: int,
    dtype_: Union[Type[AllowedDtype], dtype],
    n_lists: int = 1,
) -> Union[List[Tuple[float, AllowedDtype]], List[List[Tuple[float, AllowedDtype]]]]:
    """
    Generate arbitrary samples for a signal where the time stamps are strictly
    monotonically increasing

    :param n_lists: used to generate multiple sample lists with the same time domain. This is used for testing against
                    `metric-temporal-logic` as it doesn't check for non-overlapping domains.
    """
    n_lists = max(1, n_lists)
    timestamps = draw(
        st.lists(
            st.integers(min_value=0, max_value=2**32 - 1).map(lambda i: i / 1000),
            unique=True,
            min_size=min_size,
            max_size=max_size,
        ).map(lambda t: list(sorted(set(t))))
    )
    elements = gen_element_fn(dtype_)

    sample_lists = [
        draw(st.lists(elements, min_size=len(timestamps), max_size=len(timestamps)).map(lambda xs: list(zip(timestamps, xs))))
        for _ in range(n_lists)
    ]
    if n_lists == 1:
        return sample_lists[0]
    else:
        return sample_lists


def empty_signal(dtype_: Union[Type[AllowedDtype], dtype]) -> SearchStrategy[argus.Signal]:
    new_dtype: dtype = dtype.convert(dtype_)
    sig: argus.Signal
    if new_dtype == dtype.bool_:
        sig = argus.BoolSignal()
        assert sig.kind == dtype.bool_
    elif new_dtype == dtype.uint64:
        sig = argus.UnsignedIntSignal()
        assert sig.kind == dtype.uint64
    elif new_dtype == dtype.int64:
        sig = argus.IntSignal()
        assert sig.kind == dtype.int64
    elif new_dtype == dtype.float64:
        sig = argus.FloatSignal()
        assert sig.kind == dtype.float64
    else:
        raise ValueError("unknown dtype")
    return st.just(sig)


def constant_signal(dtype_: Union[Type[AllowedDtype], dtype]) -> SearchStrategy[argus.Signal]:
    element = gen_element_fn(dtype_)
    dtype_ = dtype.convert(dtype_)
    if dtype_ == dtype.bool_:
        return element.map(lambda val: argus.BoolSignal.constant(typing.cast(bool, val)))
    if dtype_ == dtype.uint64:
        return element.map(lambda val: argus.UnsignedIntSignal.constant(typing.cast(int, val)))
    if dtype_ == dtype.int64:
        return element.map(lambda val: argus.IntSignal.constant(typing.cast(int, val)))
    if dtype_ == dtype.float64:
        return element.map(lambda val: argus.FloatSignal.constant(typing.cast(float, val)))
    raise ValueError("unsupported data type for signal")


def sampled_signal(xs: List[Tuple[float, AllowedDtype]], dtype_: Union[Type[AllowedDtype], dtype]) -> argus.Signal:
    dtype_ = dtype.convert(dtype_)
    if dtype_ == dtype.bool_:
        return argus.BoolSignal.from_samples(typing.cast(List[Tuple[float, bool]], xs))
    if dtype_ == dtype.uint64:
        return argus.UnsignedIntSignal.from_samples(typing.cast(List[Tuple[float, int]], xs))
    if dtype_ == dtype.int64:
        return argus.IntSignal.from_samples(typing.cast(List[Tuple[float, int]], xs))
    if dtype_ == dtype.float64:
        return argus.FloatSignal.from_samples(typing.cast(List[Tuple[float, float]], xs))
    raise ValueError("unsupported data type for signal")


@composite
def draw_index(draw: st.DrawFn, vec: List) -> int:
    if len(vec) > 0:
        return draw(st.integers(min_value=0, max_value=len(vec) - 1))
    else:
        return draw(st.just(0))


def gen_dtype() -> SearchStrategy[Union[Type[AllowedDtype], dtype]]:
    return st.one_of(
        list(map(st.just, [dtype.bool_, dtype.uint64, dtype.int64, dtype.float64, bool, int, float])),  # type: ignore[arg-type]
    )
