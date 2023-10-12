import logging

from hypothesis import given

import argus

from .utils.expr_gen import argus_expr


@given(spec=argus_expr())
def test_correct_expr(spec: str) -> None:
    try:
        _ = argus.parse_expr(spec)
    except ValueError as e:
        logging.critical(f"unable to parse expr: {spec}")
        raise e
