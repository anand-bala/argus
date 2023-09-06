from typing import List, Type, Union

import pytest
from hypothesis import Verbosity, given, note, settings
from hypothesis import strategies as st
from hypothesis.strategies import SearchStrategy, composite

import argus

AllowedDtype = Union[bool, int, float]


@composite
def gen_samples(draw: st.DrawFn, *, min_size: int, max_size: int, dtype: Type[AllowedDtype]):
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
    xs = draw(
        st.lists(
            st.integers(min_value=0, max_value=2**32 - 1),
            unique=True,
            min_size=len(values),
            max_size=len(values),
        )
        .map(lambda t: map(float, sorted(set(t))))
        .map(lambda t: list(zip(t, values)))
    )
    return xs


@composite
def draw_index(draw: st.DrawFn, vec: List) -> int:
    if len(vec) > 0:
        return draw(st.integers(min_value=0, max_value=len(vec) - 1))
    else:
        return draw(st.just(0))


def gen_dtype() -> SearchStrategy[Type[AllowedDtype]]:
    return st.one_of(st.just(bool), st.just(int), st.just(float))


@settings(verbosity=Verbosity.verbose)
@given(st.data())
def test_correctly_create_signals(data: st.DataObject) -> None:
    dtype = data.draw(gen_dtype())
    xs = data.draw(gen_samples(min_size=0, max_size=100, dtype=dtype))

    note(f"Samples: {gen_samples}")
    signal = argus.signal(dtype, data=xs)
    if len(xs) > 0:
        expected_start_time = xs[0][0]
        expected_end_time = xs[-1][0]

        actual_start_time = signal.start_time
        actual_end_time = signal.end_time

        assert actual_start_time is not None
        assert actual_start_time == expected_start_time
        assert actual_end_time is not None
        assert actual_end_time == expected_end_time

    else:
        assert signal.is_empty()
        assert signal.at(0) is None


@settings(verbosity=Verbosity.verbose)
@given(st.data())
def test_signal_at(data: st.DataObject) -> None:
    dtype = data.draw(gen_dtype())
    xs = data.draw(gen_samples(min_size=10, max_size=100, dtype=dtype))
    a = data.draw(draw_index(xs))

    assert len(xs) > 2
    assert a < len(xs)

    signal = argus.signal(dtype, data=xs)

    at, expected_val = xs[a]
    actual_val = signal.at(at)

    assert actual_val is not None
    assert actual_val == expected_val


@given(st.data())
def test_signal_create_should_fail(data: st.DataObject) -> None:
    dtype = data.draw(gen_dtype())
    xs = data.draw(gen_samples(min_size=10, max_size=100, dtype=dtype))
    a = data.draw(draw_index(xs))
    b = data.draw(draw_index(xs))

    assert len(xs) > 2
    assert a < len(xs)
    assert b < len(xs)
    # Swap two indices in the samples
    xs[b], xs[a] = xs[a], xs[b]

    with pytest.raises(RuntimeError):
        _ = argus.signal(dtype, data=xs)
