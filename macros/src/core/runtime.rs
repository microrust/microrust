use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

pub fn core_runtime_main_impl(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let tokens = quote! {
        #[tokio::main]
        #input
    };
    TokenStream::from(tokens)
}

pub fn core_runtime_test_impl(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let tokens = quote! {
        #[tokio::test]
        #input
    };
    TokenStream::from(tokens)
}
