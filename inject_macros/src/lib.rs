mod error;
mod inject_singleton;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{ItemImpl, parse_macro_input};

use crate::inject_singleton::inject_singleton_impl;
extern crate proc_macro;

#[proc_macro_attribute]
pub fn inject_singleton(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    inject_singleton_impl(args, input)
}

#[proc_macro_attribute]
pub fn inject_singleton_mutex(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    if args.clone().into_iter().size_hint() != (1, Some(1)) {
        panic!("Please provide a constructor method like #[inject_singleton(new)]");
    }
    let constructor_ident = parse_macro_input!(args as Ident);

    let input = parse_macro_input!(input as ItemImpl);
    // println!("{:#?}", input);
    if input.trait_ != None {
        panic!("Please use attribute macro #[inject_trait]");
    }

    let struct_ident = match &*input.self_ty {
        syn::Type::Path(type_path) => type_path.path.segments.first().unwrap().ident.clone(),
        _ => panic!("Expecting: impl <type>"),
    };

    let ctor = "ctor_".to_owned() + &struct_ident.to_string();
    let ctor_ident = Ident::new(&ctor, Span::call_site());

    let tokens = quote! {
    #input

    impl ::microrust_inject::__InstanceMutexFactory for #struct_ident {
        fn __create_instance() -> ::std::sync::Arc<dyn ::std::any::Any + Send + Sync> {
            println!("__create_instance() called");

            let inst = ::std::sync::Arc::new(::std::sync::Mutex::new(#struct_ident::#constructor_ident()));
            let inst_any = inst as ::std::sync::Arc<dyn ::std::any::Any + Send + Sync>;
            inst_any
        }
    }

    microrust_inject::ctor_parse!(#[ctor] fn #ctor_ident() {::microrust_inject::__collect_struct_mutex::<#struct_ident>();});
    };
    // println!("{:#?}", tokens);
    TokenStream::from(tokens)
}

#[proc_macro_attribute]
pub fn inject_singleton_rwlock(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    if args.clone().into_iter().size_hint() != (1, Some(1)) {
        panic!("Please provide a constructor method like #[inject_singleton(new)]");
    }
    let constructor_ident = parse_macro_input!(args as Ident);

    let input = parse_macro_input!(input as ItemImpl);
    // println!("{:#?}", input);
    if input.trait_ != None {
        panic!("Please use attribute macro #[inject_trait]");
    }

    let struct_ident = match &*input.self_ty {
        syn::Type::Path(type_path) => type_path.path.segments.first().unwrap().ident.clone(),
        _ => panic!("Expecting: impl <type>"),
    };

    let ctor = "ctor_".to_owned() + &struct_ident.to_string();
    let ctor_ident = Ident::new(&ctor, Span::call_site());

    let tokens = quote! {
    #input

    impl ::microrust_inject::__InstanceRwLockFactory for #struct_ident {
        fn __create_instance() -> ::std::sync::Arc<dyn ::std::any::Any + Send + Sync> {
            println!("__create_instance() called");

            let inst = ::std::sync::Arc::new(::std::sync::RwLock::new(#struct_ident::#constructor_ident()));
            let inst_any = inst as ::std::sync::Arc<dyn ::std::any::Any + Send + Sync>;
            inst_any
        }
    }

    microrust_inject::ctor_parse!(#[ctor] fn #ctor_ident() {::microrust_inject::__collect_struct_rwlock::<#struct_ident>();});
    };
    // println!("{:#?}", tokens);
    TokenStream::from(tokens)
}

// #[doc(hidden)]
// #[proc_macro_attribute]
// pub fn inject_singleton_tokio_mutex(
//     _args: proc_macro::TokenStream,
//     _input: proc_macro::TokenStream,
// ) -> proc_macro::TokenStream {
//     todo!()
// }

// #[doc(hidden)]
// #[proc_macro_attribute]
// pub fn inject_singleton_tokio_rwlock(
//     _args: proc_macro::TokenStream,
//     _input: proc_macro::TokenStream,
// ) -> proc_macro::TokenStream {
//     todo!()
// }

// #[doc(hidden)]
// #[proc_macro_attribute]
// pub fn inject_request(
//     args: proc_macro::TokenStream,
//     input: proc_macro::TokenStream,
// ) -> proc_macro::TokenStream {
//     if args.clone().into_iter().size_hint() != (1, Some(1)) {
//         panic!("Please provide a constructor method like #[inject_request(new)]");
//     }
//     let constructor_ident = parse_macro_input!(args as Ident);

//     let input = parse_macro_input!(input as ItemImpl);
//     // println!("{:#?}", input);
//     if input.trait_ != None {
//         panic!("Please use attribute macro #[inject_trait]");
//     }

//     // println!("{:#?}", input);

//     todo!()
// }

#[proc_macro_attribute]
pub fn inject_trait(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ItemImpl);
    // println!("{:#?}", input);
    if input.trait_ == None {
        panic!("Expecting: impl <trait> for <type>");
    }

    let trait_ = input.trait_.clone().unwrap();
    let trait_ident = trait_.1.get_ident().unwrap();

    let struct_ident = match &*input.self_ty {
        syn::Type::Path(type_path) => type_path.path.segments.first().unwrap().ident.clone(),
        _ => panic!("Expecting: impl <trait> for <type>"),
    };

    let ctor = "ctor_".to_owned() + &trait_ident.to_string();
    let ctor_ident = Ident::new(&ctor, Span::call_site());

    let tokens = quote! {
    #[async_trait]
    #input

    microrust_inject::ctor_parse!(#[ctor] fn #ctor_ident() {::microrust_inject::__collect_trait::<dyn #trait_ident, #struct_ident>();});

    microrust_inject::register! { #struct_ident => dyn #trait_ident }
    };
    // println!("{:#?}", tokens);
    TokenStream::from(tokens)
}

#[proc_macro_attribute]
pub fn inject_trait_mutex(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ItemImpl);
    // println!("{:#?}", input);
    if input.trait_ == None {
        panic!("Expecting: impl <trait> for <type>");
    }

    let trait_ = input.trait_.clone().unwrap();
    let trait_ident = trait_.1.get_ident().unwrap();

    let struct_ident = match &*input.self_ty {
        syn::Type::Path(type_path) => type_path.path.segments.first().unwrap().ident.clone(),
        _ => panic!("Expecting: impl <trait> for <type>"),
    };

    let ctor = "ctor_".to_owned() + &trait_ident.to_string();
    let ctor_ident = Ident::new(&ctor, Span::call_site());

    let tokens = quote! {
    #[async_trait]
    #input

    microrust_inject::ctor_parse!(#[ctor] fn #ctor_ident() {::microrust_inject::__collect_trait_mutex::<dyn #trait_ident, #struct_ident>();});

    microrust_inject::register! { ::std::sync::Mutex<#struct_ident> => ::std::sync::Mutex<dyn #trait_ident> }
    };
    // println!("{:#?}", tokens);
    TokenStream::from(tokens)
}

#[proc_macro_attribute]
pub fn inject_trait_rwlock(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ItemImpl);
    // println!("{:#?}", input);
    if input.trait_ == None {
        panic!("Expecting: impl <trait> for <type>");
    }

    let trait_ = input.trait_.clone().unwrap();
    let trait_ident = trait_.1.get_ident().unwrap();

    let struct_ident = match &*input.self_ty {
        syn::Type::Path(type_path) => type_path.path.segments.first().unwrap().ident.clone(),
        _ => panic!("Expecting: impl <trait> for <type>"),
    };

    let ctor = "ctor_".to_owned() + &trait_ident.to_string();
    let ctor_ident = Ident::new(&ctor, Span::call_site());

    let tokens = quote! {
    #[async_trait]
    #input

    microrust_inject::ctor_parse!(#[ctor] fn #ctor_ident() {::microrust_inject::__collect_trait_rwlock::<dyn #trait_ident, #struct_ident>();});

    microrust_inject::register! { ::std::sync::RwLock<#struct_ident> => ::std::sync::RwLock<dyn #trait_ident> }
    };
    // println!("{:#?}", tokens);
    TokenStream::from(tokens)
}

// #[doc(hidden)]
// #[proc_macro_attribute]
// pub fn inject_dynosaur(
//     _args: proc_macro::TokenStream,
//     _input: proc_macro::TokenStream,
// ) -> proc_macro::TokenStream {
//     todo!()
// }
