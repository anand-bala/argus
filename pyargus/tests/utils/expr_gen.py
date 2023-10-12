"""Hypothesis strategies to generate Argus expressions
"""
import hypothesis.strategies as st
from hypothesis.extra.lark import from_lark
from lark import Lark, Transformer


class T(Transformer):
    def INT(self, tok):  # noqa: N802,ANN # type: ignore
        "Convert the value of `tok` from string to int, while maintaining line number & column."
        return tok.update(value=int(tok) // 2**64)


ARGUS_EXPR_GRAMMAR = Lark(
    r"""

TRUE: "true" | "TRUE"
FALSE: "false" | "FALSE"
BOOLEAN: TRUE | FALSE

IDENT: ESCAPED_STRING | CNAME

num_expr: num_expr "*" num_expr
        | num_expr "/" num_expr
        | num_expr "+" num_expr
        | num_expr "-" num_expr
        | "-" num_expr
        | NUMBER
        | IDENT
        | "(" num_expr ")"

cmp_expr: num_expr ">=" num_expr
        | num_expr "<=" num_expr
        | num_expr "<" num_expr
        | num_expr ">" num_expr
        | num_expr "==" num_expr
        | num_expr "!=" num_expr

INTERVAL: "[" INT? "," INT? "]"

bool_expr: bool_expr "&&" bool_expr
         | bool_expr "||" bool_expr
         | bool_expr "<=>" bool_expr
         | bool_expr "->" bool_expr
         | bool_expr "^" bool_expr
         | bool_expr WS_INLINE "U" WS_INLINE INTERVAL? bool_expr
         | "!" bool_expr
         | "G" WS_INLINE INTERVAL? bool_expr
         | "F" WS_INLINE INTERVAL? bool_expr
         | cmp_expr
         | BOOLEAN
         | IDENT
         | "(" bool_expr ")"

phi: bool_expr

%import common.ESCAPED_STRING
%import common.CNAME
%import common.NUMBER
%import common.INT
%import common.WS
%import common.WS_INLINE
%ignore WS

""",
    start="phi",
)


@st.composite
def argus_expr(draw: st.DrawFn) -> str:
    """Strategy to generate an Argus STL expression from a pre-defined grammar"""
    return draw(from_lark(ARGUS_EXPR_GRAMMAR, start="phi"))
