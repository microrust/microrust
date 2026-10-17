use crate::error::macro_error;
use proc_macro::TokenStream;
use quote::quote;
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
    let syn::Type::Path(self_path) = &*input.self_ty else {
        return macro_error("Expecting: impl <trait> for <type>");
    };

    // The coercion from `Arc<Struct>` to `Arc<dyn Trait>` is only provable
    // here, where both concrete types are in scope, so the resolver is built
    // at the impl site and handed over as a plain `fn` pointer.
    let tokens = quote! {
    #[::microrust::__private::async_trait]
    #input

    // Scoped in an anonymous `const` for the same reason as in `#[singleton]`:
    // a name built from the trait and struct idents collided for generic
    // traits implemented twice, and for structs named like `<Trait>_<Struct>`.
    const _: () = {
    ::microrust::ctor_parse!(#[ctor(unsafe)] fn __microrust_register() {
        ::microrust::inject::__collect_trait::<dyn #trait_path, #self_path>(
            || {
                let inst: ::std::sync::Arc<#self_path> =
                    ::microrust::inject::get_instance::<#self_path>();
                let as_trait: ::std::sync::Arc<dyn #trait_path> = inst;
                as_trait
            },
        );
    });
    };
    };
    // println!("{:#?}", tokens);
    TokenStream::from(tokens)
}
