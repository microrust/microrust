//! Procedural macros for [`microrust`](https://docs.rs/microrust).
//!
//! These are re-exported by the `microrust` crate and are not meant to be
//! depended on directly: the generated code resolves its helpers through
//! `microrust`, so using it adds no transitive dependencies of its own.
//!
//! Every macro reports misuse as a `compile_error!` rather than panicking, so a
//! mistake surfaces as a normal compiler diagnostic with a span instead of an
//! opaque "custom attribute panicked".

mod core;
mod error;
mod inject;

use crate::{
    core::runtime::{core_runtime_main_impl, core_runtime_test_impl},
    inject::{singleton::inject_singleton_impl, traits::inject_trait_impl},
};
extern crate proc_macro;

// core

/// Runs `main` on a multi-threaded Tokio runtime.
///
/// ```ignore
/// #[microrust::main]
/// async fn main() { /* ... */ }
/// ```
#[proc_macro_attribute]
pub fn main(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    core_runtime_main_impl(args, input)
}

/// Runs a test body on a single-threaded Tokio runtime.
///
/// ```ignore
/// #[microrust::test]
/// async fn my_test() { /* ... */ }
/// ```
#[proc_macro_attribute]
pub fn test(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    core_runtime_test_impl(args, input)
}

// inject

/// Registers the `impl` block's named associated function as the constructor
/// for a singleton.
///
/// ```ignore
/// #[microrust::singleton(new)]
/// impl Repo {
///     fn new(config: Arc<Config>) -> Self { /* ... */ }
/// }
/// ```
///
/// Every `Arc<...>` constructor parameter is injected; the struct is built
/// lazily on the first `get_instance::<Self>()`.
///
/// # Errors
///
/// Emits a `compile_error!` if the attribute does not take exactly one argument,
/// or is applied to a trait `impl` (that is [`inject_trait`]'s job).
#[proc_macro_attribute]
pub fn singleton(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    inject_singleton_impl(args, input)
}

/// Binds a struct as the implementation of a trait, so it can be requested as
/// `Arc<dyn Trait>`.
///
/// ```ignore
/// #[microrust::inject_trait]
/// impl UserRepo for PostgresRepo {
///     /* ... */
/// }
/// ```
///
/// Applies `#[async_trait]` to the `impl` itself, so the trait needs no
/// `Cargo.toml` entry for `async-trait`.
///
/// # Errors
///
/// Emits a `compile_error!` if applied to anything other than a trait `impl`.
#[proc_macro_attribute]
pub fn inject_trait(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    inject_trait_impl(_args, input)
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
