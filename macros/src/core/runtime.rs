use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Expr, ExprLit, ItemFn, Lit, Meta, Token, parse::Parser, parse_macro_input,
    punctuated::Punctuated,
};

/// The only runtime flavor MicroRust supports.
const FLAVOR: &str = "multi_thread";

/// Wraps `input` in tokio's `attr` macro, resolved through `microrust`.
///
/// `crate = ...` tells tokio's expansion to name its runtime through the
/// re-export too; without it the generated body refers to a bare `tokio` and
/// still needs a direct dependency.
///
/// `flavor` is always set to `multi_thread`, which matters most for `test`:
/// `#[tokio::test]` defaults to the single-threaded runtime. Every other
/// argument (`worker_threads`, ...) is forwarded.
///
/// The body is prefixed with the startup checks, so broken wiring (a cycle, an
/// ambiguous trait binding) fails before any user code runs, in tests as in
/// the application.
fn wrap_in_tokio(attr: TokenStream2, args: TokenStream, input: TokenStream) -> TokenStream {
    let args = match Punctuated::<Meta, Token![,]>::parse_terminated.parse(args) {
        Ok(args) => args,
        Err(err) => return err.into_compile_error().into(),
    };
    let mut input = parse_macro_input!(input as ItemFn);

    let mut forwarded: Vec<&Meta> = Vec::new();
    for arg in &args {
        // Only valid on the single-threaded runtime. Rejected here because
        // tokio's own error for it advises `flavor = "current_thread"`, which
        // microrust doesn't offer.
        if arg.path().is_ident("start_paused") {
            return syn::Error::new_spanned(
                arg,
                "`start_paused` requires tokio's single-threaded runtime, which \
                 microrust doesn't support",
            )
            .into_compile_error()
            .into();
        }
        if !arg.path().is_ident("flavor") {
            forwarded.push(arg);
            continue;
        }
        // An explicit `flavor = "multi_thread"` is redundant but harmless, so
        // it is accepted and dropped in favour of the one emitted below;
        // forwarding it as well would make tokio reject a duplicate argument.
        if !is_str_value(arg, FLAVOR) {
            return syn::Error::new_spanned(
                arg,
                "microrust only supports tokio's multi-threaded runtime; \
                 remove the `flavor` argument",
            )
            .into_compile_error()
            .into();
        }
    }

    // Only the body is replaced, so every other part of the item (attributes,
    // signature, anything syn adds later) passes through untouched. The
    // original block is nested rather than spliced so its tail expression stays
    // the function's return value (`main` may return a `Result`).
    let body = &input.block;
    let checked = quote! {
        {
            ::microrust::inject::__startup_checks();
            #body
        }
    };
    match syn::parse2(checked) {
        Ok(block) => *input.block = block,
        Err(err) => return err.into_compile_error().into(),
    }

    let tokens = quote! {
        #[::microrust::__private::tokio::#attr(
            crate = "::microrust::__private::tokio",
            flavor = #FLAVOR
            #(, #forwarded)*
        )]
        #input
    };
    TokenStream::from(tokens)
}

/// Whether `meta` is `name = "<expected>"`.
fn is_str_value(meta: &Meta, expected: &str) -> bool {
    let Meta::NameValue(name_value) = meta else {
        return false;
    };
    matches!(
        &name_value.value,
        Expr::Lit(ExprLit { lit: Lit::Str(value), .. }) if value.value() == expected
    )
}

pub fn core_runtime_main_impl(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    wrap_in_tokio(quote!(main), args, input)
}

pub fn core_runtime_test_impl(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    wrap_in_tokio(quote!(test), args, input)
}
