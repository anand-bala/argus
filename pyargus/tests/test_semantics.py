from typing import List, Tuple

import hypothesis.strategies as st
import mtl
from hypothesis import given

import argus
from argus.test_utils.signals_gen import gen_samples


@given(
    sample_lists=gen_samples(min_size=3, max_size=50, dtype_=bool, n_lists=2),
    spec=st.one_of(
        [
            st.just(spec)
            for spec in [
                "a",
                "~a",
                "(a & b)",
                "(a | b)",
                "(a -> b)",
                "(a <-> b)",
                "(a ^ b)",
            ]
        ]
    ),
)
def test_boolean_propositional_expr(
    sample_lists: List[List[Tuple[float, bool]]],
    spec: str,
) -> None:
    mtl_spec = mtl.parse(spec)
    argus_spec = argus.parse_expr(spec)
    assert isinstance(argus_spec, argus.BoolExpr)

    a, b = sample_lists
    mtl_data = dict(a=a, b=b)
    argus_data = argus.Trace(
        dict(
            a=argus.BoolSignal.from_samples(a, interpolation_method="constant"),
            b=argus.BoolSignal.from_samples(b, interpolation_method="constant"),
        )
    )

    mtl_rob = mtl_spec(mtl_data, quantitative=False)
    argus_rob = argus.eval_bool_semantics(argus_spec, argus_data)

    assert mtl_rob == argus_rob.at(0), f"{argus_rob=}"


@given(
    sample_lists=gen_samples(min_size=3, max_size=50, dtype_=bool, n_lists=2),
    spec=st.one_of(
        [
            st.just(spec)
            for spec in [
                "F a",
                "G b",
                "(G(a & b))",
                "(F(a | b))",
                # FIXME: `mtl` doesn't contract the signal domain for F[0,2] so it fails if a is True and b is False in the
                # last sample point.
                # "G(a -> F[0,2] b)",
                "G(a -> F b)",
                "G F a -> F G b",
                "(a U b)",
                "(a U[0,2] b)",
            ]
        ]
    ),
)
def test_boolean_temporal_expr(
    sample_lists: List[List[Tuple[float, bool]]],
    spec: str,
) -> None:
    mtl_spec = mtl.parse(spec)
    argus_spec = argus.parse_expr(spec)
    assert isinstance(argus_spec, argus.BoolExpr)

    a = sample_lists[0]
    b = sample_lists[1]
    mtl_data = dict(a=a, b=b)
    argus_data = argus.Trace(
        dict(
            a=argus.BoolSignal.from_samples(a, interpolation_method="constant"),
            b=argus.BoolSignal.from_samples(b, interpolation_method="constant"),
        )
    )

    mtl_rob = mtl_spec(mtl_data, quantitative=False)
    argus_rob = argus.eval_bool_semantics(argus_spec, argus_data)

    assert mtl_rob == argus_rob.at(0), f"{argus_rob=}"
