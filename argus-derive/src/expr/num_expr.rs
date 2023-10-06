use proc_macro::{self, TokenStream};
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::DeriveInput;

/// Implement [`IsNumExpr`](argus::expr::IsNumExpr) and other Numean
/// operations (`Neg`, `Add`, `Mul`, `Sub`, and `Div`) for the input identifier.
pub fn num_expr_impl(input: DeriveInput) -> TokenStream {
    let ident = &input.ident;
    let marker_impl = quote! {
        impl ::argus::expr::IsNumExpr for #ident {}
    };

    let neg_impl = impl_num_neg(&input);
    let mul_impl = impl_nary_op(&input, NumOp::Mul);
    let add_impl = impl_nary_op(&input, NumOp::Add);
    let sub_impl = impl_sub(&input);
    let div_impl = impl_div(&input);

    let output = quote! {
        #marker_impl
        #neg_impl
        #mul_impl
        #add_impl
        #sub_impl
        #div_impl
    };

    output.into()
}

fn impl_num_neg(input: &DeriveInput) -> impl ToTokens {
    let ident = &input.ident;
    quote! {
        impl ::core::ops::Neg for #ident {
            type Output = ::argus::expr::NumExpr;

            #[inline]
            fn neg(self) -> Self::Output {
                (::argus::expr::Neg { arg: Box::new(self.into()) }).into()
            }
        }
    }
}

enum NumOp {
    Add,
    Mul,
}

impl NumOp {
    fn get_trait_name(&self) -> Ident {
        match self {
            NumOp::Add => Ident::new("Add", Span::call_site()),
            NumOp::Mul => Ident::new("Mul", Span::call_site()),
        }
    }

    fn get_trait_fn(&self) -> Ident {
        match self {
            NumOp::Add => Ident::new("add", Span::call_site()),
            NumOp::Mul => Ident::new("mul", Span::call_site()),
        }
    }

    fn get_expr_name(&self) -> Ident {
        match self {
            NumOp::Add => Ident::new("Add", Span::call_site()),
            NumOp::Mul => Ident::new("Mul", Span::call_site()),
        }
    }
}

fn impl_nary_op(input: &DeriveInput, op: NumOp) -> impl ToTokens {
    let ident = &input.ident;
    let trait_name = op.get_trait_name();
    let trait_fn = op.get_trait_fn();
    let node_name = op.get_expr_name();
    quote! {
        impl<T> ::core::ops::#trait_name<T> for #ident
        where
            T: ::core::convert::Into<::argus::expr::NumExpr>
        {
            type Output = ::argus::expr::NumExpr;

            #[inline]
            fn #trait_fn(self, other: T) -> Self::Output {
                use ::argus::expr::NumExpr;
                use ::argus::expr::#node_name;
                let lhs: NumExpr = self.into();
                let rhs: NumExpr = other.into();

                let expr = match (lhs, rhs) {
                    (NumExpr::#node_name(#node_name { args: mut left }), NumExpr::#node_name(#node_name { args: mut right })) => {
                        left.append(&mut right);
                        #node_name { args: left }
                    }
                    (NumExpr::#node_name(#node_name { mut args }), other) | (other, NumExpr::#node_name(#node_name { mut args })) => {
                        args.push(other);
                        #node_name { args }
                    }
                    (left, right) => {
                        let args = vec![left, right];
                        #node_name { args }
                    }
                };
                expr.into()
            }
        }
    }
}

fn impl_sub(input: &DeriveInput) -> impl ToTokens {
    let ident = &input.ident;
    quote! {
        impl<T> ::core::ops::Sub<T> for #ident
        where
            T: ::core::convert::Into<::argus::expr::NumExpr>
        {
            type Output = ::argus::expr::NumExpr;

            #[inline]
            fn sub(self, other: T) -> Self::Output {
                use ::argus::expr::Sub;
                let expr = Sub {
                    lhs: Box::new(self.into()),
                    rhs: Box::new(other.into())
                };
                expr.into()
            }
        }
    }
}

fn impl_div(input: &DeriveInput) -> impl ToTokens {
    let ident = &input.ident;
    quote! {
        impl<T> ::core::ops::Div<T> for #ident
        where
            T: ::core::convert::Into<::argus::expr::NumExpr>
        {
            type Output = ::argus::expr::NumExpr;

            #[inline]
            fn div(self, other: T) -> Self::Output {
                use ::argus::expr::Div;
                let expr = Div {
                    dividend: Box::new(self.into()),
                    divisor: Box::new(other.into())
                };
                expr.into()
            }
        }
    }
}
