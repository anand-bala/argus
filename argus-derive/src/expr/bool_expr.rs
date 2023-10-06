use proc_macro::{self, TokenStream};
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::DeriveInput;

/// Implement [`IsBoolExpr`](argus::expr::IsBoolExpr) and other Boolean
/// operations (`Not`, `BitOr`, and `BitAnd`) for the input identifier.
pub fn bool_expr_impl(input: DeriveInput) -> TokenStream {
    let ident = &input.ident;
    let marker_impl = quote! {
        impl ::argus::expr::IsBoolExpr for #ident {}
    };

    let not_impl = impl_bool_not(&input);
    let or_impl = impl_bool_and_or(&input, BoolOp::Or);
    let and_impl = impl_bool_and_or(&input, BoolOp::And);

    let output = quote! {
        #marker_impl
        #not_impl
        #or_impl
        #and_impl
    };

    output.into()
}

fn impl_bool_not(input: &DeriveInput) -> impl ToTokens {
    let ident = &input.ident;
    quote! {
        impl ::core::ops::Not for #ident {
            type Output = ::argus::expr::BoolExpr;

            #[inline]
            fn not(self) -> Self::Output {
                (::argus::expr::Not { arg: Box::new(self.into()) }).into()
            }
        }
    }
}

enum BoolOp {
    And,
    Or,
}

fn impl_bool_and_or(input: &DeriveInput, op: BoolOp) -> impl ToTokens {
    let ident = &input.ident;
    let (trait_fn, trait_name, enum_id) = match op {
        BoolOp::And => {
            let trait_name = Ident::new("BitAnd", Span::call_site());
            let trait_fn = Ident::new("bitand", Span::call_site());
            let enum_id = Ident::new("And", Span::call_site());
            (trait_fn, trait_name, enum_id)
        }
        BoolOp::Or => {
            let trait_name = Ident::new("BitOr", Span::call_site());
            let trait_fn = Ident::new("bitor", Span::call_site());
            let enum_id = Ident::new("Or", Span::call_site());
            (trait_fn, trait_name, enum_id)
        }
    };
    quote! {
        impl ::core::ops::#trait_name for #ident {
            type Output = ::argus::expr::BoolExpr;

            #[inline]
            fn #trait_fn(self, other: Self) -> Self::Output {
                use ::argus::expr::BoolExpr;
                use ::argus::expr::#enum_id;
                let lhs: BoolExpr = self.into();
                let rhs: BoolExpr = other.into();

                let expr = match (lhs, rhs) {
                    (BoolExpr::#enum_id(#enum_id { args: mut left }), BoolExpr::#enum_id(#enum_id { args: mut right })) => {
                        left.append(&mut right);
                        #enum_id { args: left }
                    }
                    (BoolExpr::#enum_id(#enum_id { mut args }), other) | (other, BoolExpr::#enum_id(#enum_id { mut args })) => {
                        args.push(other);
                        #enum_id { args }
                    }
                    (left, right) => {
                        let args = vec![left, right];
                        #enum_id { args }
                    }
                };
                expr.into()
            }
        }
    }
}
