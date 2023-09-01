from typing import List, Tuple, Type, Union

from hypothesis import given, note
from hypothesis import strategies as st
from hypothesis.strategies import composite

import argus

AllowedDtype = Union[bool, int, float]


@composite
def samples(draw, *, min_size: int, max_size: int, dtype: Type[AllowedDtype]):
    """
    Generate arbitrary samples for a signal where the time stamps are strictly
    monotonically increasing
    """
    elements: st.SearchStrategy[AllowedDtype]
    if dtype == bool:
        elements = st.booleans()
    elif dtype == int:
        size = 2**64
        elements = st.integers(min_value=(-size // 2), max_value=((size - 1) // 2))
    elif dtype == float:
        elements = st.floats(width=64)
    else:
        raise ValueError(f"invalid dtype {dtype}")

    values = draw(st.lists(elements, min_size=min_size, max_size=max_size))
    return draw(
        st.lists(st.floats(min_value=0), unique=True, min_size=len(values), max_size=len(values))
        .map(lambda t: sorted(t))
        .map(lambda t: list(zip(t, values)))
    )


@composite
def samples_and_indices(
    draw: st.DrawFn, *, min_size: int, max_size: int
) -> Tuple[List[Tuple[float, AllowedDtype]], int, int, Type[AllowedDtype]]:
    """
    Generate an arbitrary list of samples and two indices within the list
    """
    dtype = draw(st.one_of(st.just(bool), st.just(int), st.just(float)))
    xs = draw(samples(min_size=min_size, max_size=max_size, dtype=dtype))
    if len(xs) > 0:
        i0 = draw(st.integers(min_value=0, max_value=len(xs) - 1))
        i1 = draw(st.integers(min_value=0, max_value=len(xs) - 1))
    else:
        i0 = draw(st.just(0))
        i1 = draw(st.just(0))

    return (xs, i0, i1, dtype)


@given(samples_and_indices(min_size=0, max_size=100))
def test_correctly_create_signals(data: Tuple[List[Tuple[float, AllowedDtype]], int, int, Type[AllowedDtype]]) -> None:
    samples: List[Tuple[float, AllowedDtype]] = data[0]
    a: int = data[1]
    b: int = data[2]
    dtype: Type[AllowedDtype] = data[3]

    note(f"Samples: {samples}")
    signal = argus.signal(dtype, data=samples)
    if len(samples) > 0:
        assert a < len(samples)
        assert b < len(samples)
    else:
        assert signal.is_empty()
        assert signal.at(0) is None
