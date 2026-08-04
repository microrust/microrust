use crate::error::macro_error;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{ItemImpl, parse_macro_input};

pub fn inject_singleton_impl(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    if args.clone().into_iter().size_hint() != (1, Some(1)) {
        return macro_error("Please provide a constructor method like #[singleton(new)]");
    }
    let constructor_ident = parse_macro_input!(args as Ident);

    let input = parse_macro_input!(input as ItemImpl);
    // println!("{:#?}", input);
    if input.trait_.is_some() {
        return macro_error("Please use attribute macro #[inject_trait]");
    }

    let struct_ident = match &*input.self_ty {
        syn::Type::Path(type_path) => type_path.path.segments.first().unwrap().ident.clone(),
        _ => return macro_error("Expecting: impl <type>"),
    };

    let constructor_fn: Option<&syn::ImplItemFn> = input
        .items
        .iter()
        .filter_map(|item| {
            if let syn::ImplItem::Fn(func) = item {
                if func.sig.ident == constructor_ident {
                    Some(func)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .next();

    if constructor_fn.is_none() {
        return macro_error("constructor method not found");
    }

    // let constructor_inputs: Vec<&syn::PatType> = constructor_fn
    //     .unwrap()
    //     .sig
    //     .inputs
    //     .iter()
    //     .filter_map(|input| {
    //         if let syn::FnArg::Typed(pat_type) = input {
    //             Some(pat_type)
    //         } else {
    //             None
    //         }
    //     })
    //     .collect();
    // let has_inputs = !constructor_inputs.is_empty();
    // // println!("{:#?}", constructor_inputs);

    let ctor_ident = format_ident!("ctor_{}", struct_ident);

    // if !has_inputs {
    let tokens = quote! {
    #input

    impl ::microrust::inject::__InstanceFactory for #struct_ident {
        fn __create_instance() -> ::std::sync::Arc<dyn ::std::any::Any + Send + Sync> {
            println!("__create_instance() called");

            let inst = ::std::sync::Arc::new(#struct_ident::#constructor_ident());
            let inst_any = inst as ::std::sync::Arc<dyn ::std::any::Any + Send + Sync>;
            inst_any
        }
    }

    microrust::ctor_parse!(#[ctor(unsafe)] #[allow(non_snake_case)] fn #ctor_ident() {::microrust::inject::__collect_struct::<#struct_ident>();});
    };
    // println!("{:#?}", tokens);
    // return TokenStream::from(tokens);
    // }

    // let arg1 = constructor_inputs.first().unwrap().ty.as_ref();
    // // println!("{:#?}", arg1);
    // // println!("{}", arg1.to_string());

    // let tokens = quote! {
    // #input

    // impl ::microrust::inject::__InstanceFactory for #struct_ident {
    //     fn __create_instance() -> ::std::sync::Arc<dyn ::std::any::Any + Send + Sync> {
    //         println!("__create_instance() called");

    //         let arg1: ::std::sync::#arg1 = get_instance();

    //         let inst = ::std::sync::Arc::new(#struct_ident::#constructor_ident(arg1));
    //         let inst_any = inst as ::std::sync::Arc<dyn ::std::any::Any + Send + Sync>;
    //         inst_any
    //     }
    // }

    // microrust::ctor_parse!(#[ctor(unsafe)] #[allow(non_snake_case)] fn #ctor_ident() {::microrust::inject::__collect_struct::<#struct_ident>();});
    // };
    // println!("{:#?}", tokens);
    TokenStream::from(tokens)
}
