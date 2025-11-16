use crate::error::macro_error;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemImpl, parse_macro_input};

pub fn inject_trait_rwlock_impl(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ItemImpl);
    // println!("{:#?}", input);
    if input.trait_ == None {
        return macro_error("Expecting: impl <trait> for <type>");
    }

    let trait_ = input.trait_.clone().unwrap();
    let trait_ident = trait_.1.get_ident().unwrap();

    let struct_ident = match &*input.self_ty {
        syn::Type::Path(type_path) => type_path.path.segments.first().unwrap().ident.clone(),
        _ => panic!("Expecting: impl <trait> for <type>"),
    };

    let ctor_ident = format_ident!("ctor_{}", trait_ident);

    let tokens = quote! {
    #[async_trait]
    #input

    microrust_inject::ctor_parse!(#[ctor] #[allow(non_snake_case)] fn #ctor_ident() {::microrust_inject::__collect_trait_rwlock::<dyn #trait_ident, #struct_ident>();});

    microrust_inject::register! { ::std::sync::RwLock<#struct_ident> => ::std::sync::RwLock<dyn #trait_ident> }
    };
    // println!("{:#?}", tokens);
    TokenStream::from(tokens)
}
