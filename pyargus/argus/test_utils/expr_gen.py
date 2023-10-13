"""Hypothesis strategies to generate Argus expressions
"""

import hypothesis.strategies as st
import lark
from hypothesis.extra.lark import from_lark


class Transformer(lark.Transformer):
    def INT(self, tok: lark.Token) -> lark.Token:  # noqa: N802
        """Convert the value of `tok` from string to int, while maintaining line number & column.

        Performs wrapping conversion for 32-bit integers
        """
        return tok.update(value=int(float(tok) // 2**32))


ARGUS_EXPR_GRAMMAR = lark.Lark(
    r"""

TRUE: "true" | "TRUE"
FALSE: "false" | "FALSE"
BOOLEAN: TRUE | FALSE
INT: /[0-9]+/

KEYWORD: "X" | "G" | "F" | "U" | BOOLEAN

ESCAPED_STRING: /(\w|[\t ]){1,20}/
NUM_IDENT: "\"num_" ESCAPED_STRING "\""
         | "num_" CNAME
BOOL_IDENT: "\"bool_" ESCAPED_STRING "\""
          | "bool_" CNAME

num_expr: num_expr "*" num_expr
        | num_expr "/" num_expr
        | num_expr "+" num_expr
        | num_expr "-" num_expr
        | "-" num_expr
        | NUMBER
        | NUM_IDENT
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
         | BOOL_IDENT
         | "(" bool_expr ")"

phi: bool_expr

%import common.CNAME
%import common.NUMBER
%import common.WS
%import common.WS_INLINE
%ignore WS

""",
    start="phi",
    parser="lalr",
    transformer=Transformer(),
)


@st.composite
def argus_expr(draw: st.DrawFn) -> str:
    """Strategy to generate an Argus STL expression from a pre-defined grammar"""
    return draw(
        from_lark(
            ARGUS_EXPR_GRAMMAR,
            start="phi",
        )
    )
