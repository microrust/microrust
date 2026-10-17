use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

/// Wraps `input` in tokio's `attr` macro, resolved through `microrust`.
///
/// `crate = ...` tells tokio's expansion to name its runtime through the
/// re-export too; without it the generated body refers to a bare `tokio` and
/// still needs a direct dependency. The caller's own arguments (`flavor`,
/// `worker_threads`, ...) are forwarded after it rather than dropped.
fn wrap_in_tokio(attr: TokenStream2, args: TokenStream, input: TokenStream) -> TokenStream {
    let args = TokenStream2::from(args);
    let input = parse_macro_input!(input as ItemFn);
    let args = if args.is_empty() {
        quote! { crate = "::microrust::__private::tokio" }
    } else {
        quote! { crate = "::microrust::__private::tokio", #args }
    };
    let tokens = quote! {
        #[::microrust::__private::tokio::#attr(#args)]
        #input
    };
    TokenStream::from(tokens)
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
