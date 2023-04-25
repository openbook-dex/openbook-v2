use proc_macro_error::abort;
use quote::quote;
use syn::{spanned::Spanned, BinOp, Expr, ExprBinary, ExprUnary, Ident, Lit, UnOp};

pub fn transform_expr_or_panic(expr: Expr) -> proc_macro2::TokenStream {
    match expr {
        Expr::Group(g) => {
            let expr = transform_expr_or_panic(*g.expr);
            quote! {(#expr)}
        }
        Expr::AssignOp(assign_op) => {
            // Rewrite `left += right` into `left = checked!(left + right).unwrap()`
            let bin_op = Expr::Binary(ExprBinary {
                attrs: vec![],
                left: assign_op.left.clone(),
                right: assign_op.right.clone(),
                op: match assign_op.op {
                    BinOp::AddEq(t) => BinOp::Add(syn::token::Add(t.spans[0])),
                    BinOp::SubEq(t) => BinOp::Sub(syn::token::Sub(t.spans[0])),
                    BinOp::MulEq(t) => BinOp::Mul(syn::token::Star(t.spans[0])),
                    BinOp::DivEq(t) => BinOp::Div(syn::token::Div(t.spans[0])),
                    _ => panic!("unsupported AssignOp.op: {:#?}", assign_op.op),
                },
            });
            let left = assign_op.left;
            let bin_op_tokens = transform_expr(bin_op);
            quote! {
                #left = (#bin_op_tokens).unwrap_or_else(|| panic!("math error"))
            }
        }
        _ => {
            let toks = transform_expr(expr);
            quote! { (#toks).unwrap_or_else(|| panic!("math error")) }
        }
    }
}

pub fn transform_expr(mut expr: Expr) -> proc_macro2::TokenStream {
    match expr {
        Expr::Unary(unary) => transform_unary(unary),
        Expr::Binary(binary) => transform_binary(binary),
        Expr::MethodCall(ref mut mc) => {
            if mc.method == "pow" {
                mc.method = syn::Ident::new("checked_pow", mc.method.span());
                quote! { #mc }
            } else if mc.method == "abs" {
                mc.method = syn::Ident::new("checked_abs", mc.method.span());
                quote! { #mc }
            } else if mc.args.is_empty() {
                quote! { Some(#mc) }
            } else {
                abort!(mc, "method calls with arguments are not supported");
            }
        }
        Expr::Call(c) => {
            let f = c.func.clone();
            let name = quote!(#f).to_string();
            if c.args.is_empty() {
                quote! { Some(#c) }
            } else if (name == "I80F48 :: from" || name == "I80F48 :: from_num")
                && c.args.len() == 1
            {
                let expr = transform_expr(c.args[0].clone());
                quote! { (#expr).map(|v| #f(v)) }
            } else {
                abort!(c, "calls with arguments are not supported");
            }
        }
        Expr::Cast(c) => {
            let ty = *c.ty;
            let expr = transform_expr(*c.expr);
            quote! {
                #expr.map(|v| v as #ty)
            }
        }
        Expr::Paren(p) => {
            let expr = transform_expr(*p.expr);
            quote! {
                (#expr)
            }
        }
        Expr::Group(g) => {
            let expr = transform_expr(*g.expr);
            quote! {
                (#expr)
            }
        }
        Expr::Lit(lit) => match lit.lit {
            Lit::Int(_) | Lit::Float(_) => quote! { Some(#lit) },
            _ => abort!(lit, "unsupported literal"),
        },
        Expr::Path(_) | Expr::Field(_) => {
            quote! { Some(#expr) }
        }
        _ => {
            abort!(expr, "unsupported expr {:?}", expr);
        }
    }
}

fn transform_unary(unary: ExprUnary) -> proc_macro2::TokenStream {
    let expr = transform_expr(*unary.expr);
    let op = unary.op;
    match op {
        UnOp::Neg(_) => {
            quote! {
                {
                    match #expr {
                        Some(e) => e.checked_neg(),
                        None => None
                    }
                }
            }
        }
        UnOp::Deref(_) => quote! { #expr },
        UnOp::Not(_) => abort!(expr, "unsupported unary expr"),
    }
}

fn transform_binary(binary: ExprBinary) -> proc_macro2::TokenStream {
    let left = transform_expr(*binary.left);
    let right = transform_expr(*binary.right);
    let op = binary.op;
    let method_name = match op {
        BinOp::Add(_) => Some("checked_add"),
        BinOp::Sub(_) => Some("checked_sub"),
        BinOp::Mul(_) => Some("checked_mul"),
        BinOp::Div(_) => Some("checked_div"),
        BinOp::Rem(_) => Some("checked_rem"),
        BinOp::Shl(_) => Some("checked_shl"),
        BinOp::Shr(_) => Some("checked_shr"),
        _ => abort!(op, "unsupported binary expr"),
    };
    method_name
        .map(|method_name| {
            let method_name = Ident::new(method_name, op.span());
            quote! {
                {
                    match (#left, #right) {
                        (Some(left), Some(right)) => left.#method_name(right),
                        _ => None
                    }

                }
            }
        })
        .unwrap_or_else(|| {
            quote! {
                match (#left, #right) {
                    (Some(left), Some(right)) => left #op right,
                    _ => None
                }
            }
        })
}
