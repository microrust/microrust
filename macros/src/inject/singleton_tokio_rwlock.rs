use crate::error::macro_error;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{ItemImpl, parse_macro_input};

pub fn inject_singleton_tokio_rwlock_impl(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    if args.clone().into_iter().size_hint() != (1, Some(1)) {
        return macro_error(
            "Please provide a constructor method like #[inject_singleton_rwlock(new)]",
        );
    }
    let constructor_ident = parse_macro_input!(args as Ident);

    let input = parse_macro_input!(input as ItemImpl);
    // println!("{:#?}", input);
    if input.trait_.is_some() {
        panic!("Please use attribute macro #[inject_trait]");
    }

    let struct_ident = match &*input.self_ty {
        syn::Type::Path(type_path) => type_path.path.segments.first().unwrap().ident.clone(),
        _ => panic!("Expecting: impl <type>"),
    };

    let ctor_ident = format_ident!("ctor_{}", struct_ident);

    let tokens = quote! {
    #input

    impl ::microrust::inject::__InstanceRwLockFactory for #struct_ident {
        fn __create_instance() -> ::std::sync::Arc<dyn ::std::any::Any + Send + Sync> {
            println!("__create_instance() called");

            let inst = ::std::sync::Arc::new(::tokio::sync::RwLock::new(#struct_ident::#constructor_ident()));
            let inst_any = inst as ::std::sync::Arc<dyn ::std::any::Any + Send + Sync>;
            inst_any
        }
    }

    microrust::ctor_parse!(#[ctor] #[allow(non_snake_case)] fn #ctor_ident() {::microrust::inject::__collect_struct_rwlock::<#struct_ident>();});
    };
    // println!("{:#?}", tokens);
    TokenStream::from(tokens)
}
