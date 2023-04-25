extern crate proc_macro;

mod transform;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::parse_macro_input;

/// Produces a semantically equivalent expression as the one provided
/// except that each math call is substituted with the equivalent version
/// of the `checked` API.
///
/// Examples:
/// - `checked_math!{ 1 }` will become `Some(1)`
/// - `checked_math!{ a + b }` will become `a.checked_add(b)`
///
/// The macro is intened to be used for arithmetic expressions only and
/// significantly restricts the available syntax.
#[proc_macro]
#[proc_macro_error]
pub fn checked_math(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::Expr);
    let expanded = transform::checked::transform_expr(input);

    TokenStream::from(expanded)
}

/// Like checked_math(), but panics with "math error" on None results
#[proc_macro]
#[proc_macro_error]
pub fn checked_math_or_panic(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::Expr);
    let expanded = transform::checked::transform_expr_or_panic(input);

    TokenStream::from(expanded)
}
