mod core;
mod error;
mod inject;

use crate::{
    core::runtime::{core_runtime_main_impl, core_runtime_test_impl},
    inject::{
        singleton::inject_singleton_impl, singleton_mutex::inject_singleton_mutex_impl,
        singleton_rwlock::inject_singleton_rwlock_impl, trait_mutex::inject_trait_mutex_impl,
        trait_rwlock::inject_trait_rwlock_impl, traits::inject_trait_impl,
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
pub fn singleton_mutex(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    inject_singleton_mutex_impl(args, input)
}

#[proc_macro_attribute]
pub fn singleton_rwlock(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    inject_singleton_rwlock_impl(args, input)
}

// #[doc(hidden)]
// #[proc_macro_attribute]
// pub fn singleton_tokio_mutex(
//     _args: proc_macro::TokenStream,
//     _input: proc_macro::TokenStream,
// ) -> proc_macro::TokenStream {
//     todo!()
// }

// #[doc(hidden)]
// #[proc_macro_attribute]
// pub fn singleton_tokio_rwlock(
//     _args: proc_macro::TokenStream,
//     _input: proc_macro::TokenStream,
// ) -> proc_macro::TokenStream {
//     todo!()
// }

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

#[proc_macro_attribute]
pub fn inject_trait(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    inject_trait_impl(_args, input)
}

#[proc_macro_attribute]
pub fn inject_trait_mutex(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    inject_trait_mutex_impl(_args, input)
}

#[proc_macro_attribute]
pub fn inject_trait_rwlock(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    inject_trait_rwlock_impl(_args, input)
}

// #[doc(hidden)]
// #[proc_macro_attribute]
// pub fn inject_dynosaur(
//     _args: proc_macro::TokenStream,
//     _input: proc_macro::TokenStream,
// ) -> proc_macro::TokenStream {
//     todo!()
// }
