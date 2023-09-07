from typing import List, Tuple, Type, Union

import pytest
from hypothesis import assume, given, note
from hypothesis import strategies as st
from hypothesis.strategies import SearchStrategy, composite

import argus
from argus import DType

AllowedDtype = Union[bool, int, float]


def gen_element_fn(dtype: Union[Type[AllowedDtype], DType]) -> SearchStrategy[AllowedDtype]:
    new_dtype = DType.convert(dtype)
    if new_dtype == DType.Bool:
        return st.booleans()
    elif new_dtype == DType.Int:
        size = 2**64
        return st.integers(min_value=(-size // 2), max_value=((size - 1) // 2))
    elif new_dtype == DType.UnsignedInt:
        size = 2**64
        return st.integers(min_value=0, max_value=(size - 1))
    elif new_dtype == DType.Float:
        return st.floats(
            width=64,
            allow_nan=False,
            allow_infinity=False,
            allow_subnormal=False,
        )
    else:
        raise ValueError(f"invalid dtype {dtype}")


@composite
def gen_samples(
    draw: st.DrawFn, *, min_size: int, max_size: int, dtype: Union[Type[AllowedDtype], DType]
) -> List[Tuple[float, AllowedDtype]]:
    """
    Generate arbitrary samples for a signal where the time stamps are strictly
    monotonically increasing
    """
    elements = gen_element_fn(dtype)
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


def empty_signal(*, dtype: Union[Type[AllowedDtype], DType]) -> SearchStrategy[argus.Signal]:
    new_dtype: DType = DType.convert(dtype)
    sig: argus.Signal
    if new_dtype == DType.Bool:
        sig = argus.BoolSignal()
        assert sig.kind is bool
    elif new_dtype == DType.UnsignedInt:
        sig = argus.UnsignedIntSignal()
        assert sig.kind is int
    elif new_dtype == DType.Int:
        sig = argus.IntSignal()
        assert sig.kind is int
    elif new_dtype == DType.Float:
        sig = argus.FloatSignal()
        assert sig.kind is float
    else:
        raise ValueError("unknown dtype")
    return st.just(sig)


def constant_signal(dtype: Union[Type[AllowedDtype], DType]) -> SearchStrategy[argus.Signal]:
    return gen_element_fn(dtype).map(lambda val: argus.signal(dtype, data=val))


@composite
def draw_index(draw: st.DrawFn, vec: List) -> int:
    if len(vec) > 0:
        return draw(st.integers(min_value=0, max_value=len(vec) - 1))
    else:
        return draw(st.just(0))


def gen_dtype() -> SearchStrategy[Union[Type[AllowedDtype], DType]]:
    return st.one_of(
        list(map(st.just, [DType.Bool, DType.UnsignedInt, DType.Int, DType.Float, bool, int, float])),  # type: ignore[arg-type]
    )


@given(st.data())
def test_correct_constant_signals(data: st.DataObject) -> None:
    dtype = data.draw(gen_dtype())
    signal = data.draw(constant_signal(dtype))

    assert not signal.is_empty()
    assert signal.start_time is None
    assert signal.end_time is None


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

        a = data.draw(draw_index(xs))
        assert a < len(xs)
        at, expected_val = xs[a]
        actual_val = signal.at(at)

        assert actual_val is not None
        assert actual_val == expected_val

        # generate one more sample
        new_time = actual_end_time + 1
        new_value = data.draw(gen_element_fn(dtype))
        signal.push(new_time, new_value)  # type: ignore[arg-type]

        get_val = signal.at(new_time)
        assert get_val is not None
        assert get_val == new_value

    else:
        assert signal.is_empty()
        assert signal.start_time is None
        assert signal.end_time is None
        assert signal.at(0) is None


@given(st.data())
def test_signal_create_should_fail(data: st.DataObject) -> None:
    dtype = data.draw(gen_dtype())
    xs = data.draw(gen_samples(min_size=10, max_size=100, dtype=dtype))
    a = data.draw(draw_index(xs))
    b = data.draw(draw_index(xs))
    assume(a != b)

    assert len(xs) > 2
    assert a < len(xs)
    assert b < len(xs)
    # Swap two indices in the samples
    xs[b], xs[a] = xs[a], xs[b]

    with pytest.raises(RuntimeError, match=r"trying to create a non-monotonically signal.+"):
        _ = argus.signal(dtype, data=xs)


@given(st.data())
def test_push_to_empty_signal(data: st.DataObject) -> None:
    dtype = data.draw(gen_dtype())
    sig = data.draw(empty_signal(dtype=dtype))
    assert sig.is_empty()
    element = data.draw(gen_element_fn(dtype))
    with pytest.raises(RuntimeError, match="cannot push value to non-sampled signal"):
        sig.push(0.0, element)  # type: ignore[attr-defined]


@given(st.data())
def test_push_to_constant_signal(data: st.DataObject) -> None:
    dtype = data.draw(gen_dtype())
    sig = data.draw(constant_signal(dtype=dtype))
    assert not sig.is_empty()
    sample = data.draw(gen_samples(min_size=1, max_size=1, dtype=dtype))[0]
    with pytest.raises(RuntimeError, match="cannot push value to non-sampled signal"):
        sig.push(*sample)  # type: ignore[attr-defined]
