import logging

from hypothesis import HealthCheck, given, settings

import argus
from argus.test_utils.expr_gen import argus_expr


@given(spec=argus_expr())
@settings(suppress_health_check=[HealthCheck.too_slow])
def test_correct_expr(spec: str) -> None:
    try:
        _ = argus.parse_expr(spec)
    except ValueError as e:
        logging.critical(f"unable to parse expr: {spec}")
        raise e
