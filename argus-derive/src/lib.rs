use proc_macro::{self, TokenStream};
use syn::parse_macro_input;

mod expr;

use expr::{bool_expr_impl, num_expr_impl};

#[proc_macro_derive(BoolExpr)]
pub fn bool_expr(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    bool_expr_impl(input)
}

#[proc_macro_derive(NumExpr)]
pub fn num_expr(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    num_expr_impl(input)
}
