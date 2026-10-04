use crate::error::macro_error;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemImpl, parse_macro_input};

pub fn inject_trait_impl(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ItemImpl);
    // println!("{:#?}", input);

    // Both paths are kept whole for codegen; `get_ident()` would reject any
    // module-qualified path, and taking only the final segment of `self_ty`
    // would lose the qualification.
    let Some((trait_path, _)) = &input.trait_ else {
        return macro_error("Expecting: impl <trait> for <type>");
    };
    let Some(last_trait_segment) = trait_path.segments.last() else {
        return macro_error("Expecting: impl <trait> for <type>");
    };
    let syn::Type::Path(self_path) = &*input.self_ty else {
        return macro_error("Expecting: impl <trait> for <type>");
    };
    let Some(last_struct_segment) = self_path.path.segments.last() else {
        return macro_error("Expecting: impl <trait> for <type>");
    };

    // Generated `ctor_*` fn names must stay a bare identifier.
    let trait_ident = last_trait_segment.ident.clone();
    let struct_ident = last_struct_segment.ident.clone();

    let ctor_ident = format_ident!("ctor_{}_{}", trait_ident, struct_ident);

    // The coercion from `Arc<Struct>` to `Arc<dyn Trait>` is only provable
    // here, where both concrete types are in scope, so the resolver is built
    // at the impl site and handed over as a plain `fn` pointer.
    let tokens = quote! {
    #[::microrust::__private::async_trait]
    #input

    ::microrust::ctor_parse!(#[ctor(unsafe)] #[allow(non_snake_case)] fn #ctor_ident() {
        ::microrust::inject::__collect_trait::<dyn #trait_path>(|| {
            let inst: ::std::sync::Arc<#self_path> =
                ::microrust::inject::get_instance::<#self_path>();
            let as_trait: ::std::sync::Arc<dyn #trait_path> = inst;
            as_trait
        });
    });
    };
    // println!("{:#?}", tokens);
    TokenStream::from(tokens)
}
