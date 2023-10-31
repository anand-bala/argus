import pytest
from hypothesis import assume, given
from hypothesis import strategies as st

import argus
from argus.test_utils.signals_gen import (
    constant_signal,
    draw_index,
    empty_signal,
    gen_dtype,
    gen_element_fn,
    gen_samples,
    sampled_signal,
)


@given(st.data())
def test_correct_constant_signals(data: st.DataObject) -> None:
    dtype_ = data.draw(gen_dtype())
    signal = data.draw(constant_signal(dtype_))
    assert isinstance(signal, argus.Signal)

    assert not signal.is_empty()
    assert signal.start_time is None
    assert signal.end_time is None


@given(st.data())
def test_correctly_create_signals(data: st.DataObject) -> None:
    dtype_ = data.draw(gen_dtype())
    xs = data.draw(gen_samples(min_size=0, max_size=100, dtype_=dtype_))
    signal = sampled_signal(xs, dtype_)  # type: ignore
    assert isinstance(signal, argus.Signal)
    if len(xs) > 0:
        expected_start_time = xs[0][0]
        expected_end_time = xs[-1][0]

        actual_start_time = signal.start_time
        actual_end_time = signal.end_time

        assert actual_start_time is not None
        assert actual_start_time == pytest.approx(expected_start_time)
        assert actual_end_time is not None
        assert actual_end_time == pytest.approx(expected_end_time)

        a = data.draw(draw_index(xs))
        assert a < len(xs)
        at, expected_val = xs[a]
        actual_val = signal.at(at)  # type: ignore

        assert actual_val is not None
        if isinstance(actual_val, float):
            assert actual_val == pytest.approx(expected_val)
        else:
            assert actual_val == expected_val

        # generate one more sample
        new_time = actual_end_time + 1
        new_value = data.draw(gen_element_fn(dtype_))
        signal.push(new_time, new_value)  # type: ignore[arg-type]

        get_val = signal.at(new_time)
        assert get_val is not None
        if isinstance(get_val, float):
            assert get_val == pytest.approx(new_value)
        else:
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
    xs[b], xs[a] = xs[a], xs[b]  # type: ignore

    with pytest.raises(RuntimeError, match=r"trying to create a signal with non-monotonic time points.+"):
        _ = sampled_signal(xs, dtype_)  # type: ignore


@given(st.data())
def test_push_to_empty_signal(data: st.DataObject) -> None:
    dtype_ = data.draw(gen_dtype())
    signal = data.draw(empty_signal(dtype_=dtype_))
    assert isinstance(signal, argus.Signal)
    assert signal.is_empty()
    element = data.draw(gen_element_fn(dtype_))

    signal.push(0.0, element)
    assert signal.at(0.0) == element


@given(st.data())
def test_push_to_constant_signal(data: st.DataObject) -> None:
    dtype_ = data.draw(gen_dtype())
    signal = data.draw(constant_signal(dtype_=dtype_))
    assert isinstance(signal, argus.Signal)
    assert not signal.is_empty()
    sample = data.draw(gen_samples(min_size=1, max_size=1, dtype_=dtype_))[0]
    with pytest.raises(RuntimeError, match="cannot push value to non-sampled signal"):
        signal.push(*sample)  # type: ignore[attr-defined]
