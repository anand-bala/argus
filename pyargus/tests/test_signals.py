from typing import List, Tuple, Type, Union

import pytest
from hypothesis import assume, given, note
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
    draw: st.DrawFn, min_size: int, max_size: int, dtype_: Union[Type[AllowedDtype], dtype]
) -> List[Tuple[float, AllowedDtype]]:
    """
    Generate arbitrary samples for a signal where the time stamps are strictly
    monotonically increasing
    """
    elements = gen_element_fn(dtype_)
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


def empty_signal(*, dtype_: Union[Type[AllowedDtype], dtype]) -> SearchStrategy[argus.Signal]:
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
    return gen_element_fn(dtype_).map(lambda val: argus.signal(dtype_, data=val))


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


@given(st.data())
def test_correct_constant_signals(data: st.DataObject) -> None:
    dtype_ = data.draw(gen_dtype())
    signal = data.draw(constant_signal(dtype_))

    assert not signal.is_empty()
    assert signal.start_time is None
    assert signal.end_time is None


@given(st.data())
def test_correctly_create_signals(data: st.DataObject) -> None:
    dtype_ = data.draw(gen_dtype())
    xs = data.draw(gen_samples(min_size=0, max_size=100, dtype_=dtype_))

    note(f"Samples: {gen_samples}")
    signal = argus.signal(dtype_, data=xs)
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
        new_value = data.draw(gen_element_fn(dtype_))
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
    dtype_ = data.draw(gen_dtype())
    xs = data.draw(gen_samples(min_size=10, max_size=100, dtype_=dtype_))
    a = data.draw(draw_index(xs))
    b = data.draw(draw_index(xs))
    assume(a != b)

    assert len(xs) > 2
    assert a < len(xs)
    assert b < len(xs)
    # Swap two indices in the samples
    xs[b], xs[a] = xs[a], xs[b]

    with pytest.raises(RuntimeError, match=r"trying to create a non-monotonically signal.+"):
        _ = argus.signal(dtype_, data=xs)


@given(st.data())
def test_push_to_empty_signal(data: st.DataObject) -> None:
    dtype_ = data.draw(gen_dtype())
    sig = data.draw(empty_signal(dtype_=dtype_))
    assert sig.is_empty()
    element = data.draw(gen_element_fn(dtype_))
    with pytest.raises(RuntimeError, match="cannot push value to non-sampled signal"):
        sig.push(0.0, element)  # type: ignore[attr-defined]


@given(st.data())
def test_push_to_constant_signal(data: st.DataObject) -> None:
    dtype_ = data.draw(gen_dtype())
    sig = data.draw(constant_signal(dtype_=dtype_))
    assert not sig.is_empty()
    sample = data.draw(gen_samples(min_size=1, max_size=1, dtype_=dtype_))[0]
    with pytest.raises(RuntimeError, match="cannot push value to non-sampled signal"):
        sig.push(*sample)  # type: ignore[attr-defined]
