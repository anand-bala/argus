import logging

import hypothesis.strategies as st
from hypothesis import HealthCheck, given, settings

import argus
from argus.test_utils.expr_gen import argus_expr


@given(data=st.data())
@settings(suppress_health_check=[HealthCheck.too_slow])
def test_correct_expr(data: st.DataObject) -> None:
    spec = data.draw(argus_expr())
    try:
        _ = argus.parse_expr(spec)
    except ValueError as e:
        if "Unable to parse as 64-bit" in str(e):
            return
        logging.critical(f"unable to parse expr: {spec}")
        raise e
    except BaseException as e:
        if "PanicException" in str(type(e)) and "ParseIntError { kind: PosOverflow }" in str(e):
            return
        raise
