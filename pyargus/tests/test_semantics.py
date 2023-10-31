from typing import List, Literal, Tuple

import hypothesis.strategies as st
import mtl
import rtamt
from hypothesis import given

import argus
from argus.test_utils.signals_gen import gen_samples


def _create_rtamt_spec(spec: str) -> rtamt.StlDenseTimeSpecification:
    rtamt_spec = rtamt.StlDenseTimeSpecification()
    rtamt_spec.name = "STL Spec"
    rtamt_spec.declare_var("x", "float")
    rtamt_spec.declare_var("y", "float")
    rtamt_spec.add_sub_spec("a = (x > 0)")
    rtamt_spec.add_sub_spec("b = (y > 0)")
    rtamt_spec.spec = spec
    rtamt_spec.parse()
    return rtamt_spec


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
    interpolation_method=st.one_of(st.just("constant"), st.just("linear")),
)
def test_boolean_propositional_expr(
    sample_lists: List[List[Tuple[float, bool]]],
    spec: str,
    interpolation_method: Literal["linear", "constant"],
) -> None:
    mtl_spec = mtl.parse(spec)
    argus_spec = argus.parse_expr(spec)
    assert isinstance(argus_spec, argus.BoolExpr)

    a, b = sample_lists
    mtl_data = dict(a=a, b=b)
    argus_data = argus.Trace(
        dict(
            a=argus.BoolSignal.from_samples(a, interpolation_method=interpolation_method),
            b=argus.BoolSignal.from_samples(b, interpolation_method=interpolation_method),
        )
    )

    mtl_rob = mtl_spec(mtl_data, quantitative=False)
    argus_rob = argus.eval_bool_semantics(argus_spec, argus_data, interpolation_method=interpolation_method)

    assert mtl_rob == argus_rob.at(0), f"{argus_rob=}"


@given(
    sample_lists=gen_samples(min_size=3, max_size=50, dtype_=bool, n_lists=2),
    spec=st.one_of(
        [
            st.just(spec)
            for spec in [
                "F[0,2] a",
                "G[0,2] b",
                "F[0,10] a",
                "G[0,10] b",
                "(G(a & b))",
                "(F(a | b))",
                # FIXME: `mtl` doesn't contract the signal domain for F[0,2] so it fails if a is True and b is False in the
                # last sample point.
                # "G(a -> F[0,2] b)",
                "G(a -> F b)",
                "(G F a -> F G b)",
                "(a U b)",
                "(a U[0,2] b)",
            ]
        ]
    ),
    interpolation_method=st.one_of(st.just("constant"), st.just("linear")),
)
def test_boolean_temporal_expr(
    sample_lists: List[List[Tuple[float, bool]]],
    spec: str,
    interpolation_method: Literal["linear", "constant"],
) -> None:
    mtl_spec = mtl.parse(spec)
    argus_spec = argus.parse_expr(spec)
    assert isinstance(argus_spec, argus.BoolExpr)

    a, b = sample_lists
    mtl_data = dict(a=a, b=b)
    argus_data = argus.Trace(
        dict(
            a=argus.BoolSignal.from_samples(a, interpolation_method=interpolation_method),
            b=argus.BoolSignal.from_samples(b, interpolation_method=interpolation_method),
        )
    )

    mtl_rob = mtl_spec(mtl_data, quantitative=False)
    argus_rob = argus.eval_bool_semantics(argus_spec, argus_data, interpolation_method=interpolation_method)

    assert mtl_rob == argus_rob.at(0), f"{argus_rob=}"
