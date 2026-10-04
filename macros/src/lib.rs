mod core;
mod error;
mod inject;

use crate::{
    core::runtime::{core_runtime_main_impl, core_runtime_test_impl},
    inject::{
        function::{
            get_function_impl, inject_fn_impl, set_fake_async_function_impl,
            set_fake_function_impl,
        },
        singleton::inject_singleton_impl,
        traits::{inject_fake_trait_impl, inject_trait_impl},
    },
};
extern crate proc_macro;

// core

#[proc_macro_attribute]
pub fn main(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    core_runtime_main_impl(args, input)
}

#[proc_macro_attribute]
pub fn test(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    core_runtime_test_impl(args, input)
}

// inject

#[proc_macro_attribute]
pub fn singleton(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    inject_singleton_impl(args, input)
}

#[proc_macro_attribute]
pub fn inject_trait(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    inject_trait_impl(_args, input)
}

#[proc_macro_attribute]
pub fn inject_fake_trait(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    inject_fake_trait_impl(_args, input)
}

#[doc(hidden)]
#[proc_macro_attribute]
pub fn inject_fn(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    inject_fn_impl(_args, input)
}

/// `get_function!(name)` -> `Result<fn pointer, InjectError>`
#[proc_macro]
pub fn get_function(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    get_function_impl(input)
}

/// `set_fake_function!(name, path)` - swap in a signature-compatible callable.
#[proc_macro]
pub fn set_fake_function(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    set_fake_function_impl(input)
}

/// `set_fake_async_function!(name, path)` - swap in an `async fn` mock.
#[proc_macro]
pub fn set_fake_async_function(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    set_fake_async_function_impl(input)
}

// #[doc(hidden)]
// #[proc_macro_attribute]
// pub fn transiant(
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

// #[doc(hidden)]
// #[proc_macro_attribute]
// pub fn inject_dynosaur(
//     _args: proc_macro::TokenStream,
//     _input: proc_macro::TokenStream,
// ) -> proc_macro::TokenStream {
//     todo!()
// }
