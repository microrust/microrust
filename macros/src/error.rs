use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::Error;

pub fn macro_error(msg: &str) -> TokenStream {
    Error::new(Span::call_site(), msg)
        .into_compile_error()
        .into()
}
