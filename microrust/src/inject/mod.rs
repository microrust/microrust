//! Dependency injection: singleton structs, trait bindings, and injectable
//! functions.
//!
//! Three registries back this module, all process-global and keyed differently
//! because the three kinds of dependency have different identities:
//!
//! | Dependency | Key | Resolved by |
//! |---|---|---|
//! | struct | `TypeId` | [`get_instance`] |
//! | trait | `TypeId` of `dyn Trait` | [`get_trait_instance`] |
//! | function | `module_path!() + name` | [`get_function!`] |
//!
//! Registration happens in `#[ctor]` functions emitted by [`singleton`],
//! [`inject_trait`] and [`inject_fn`]; resolution happens on first request.
//! Nothing is constructed during registration.
//!
//! [`report_registrations`] logs what was registered and fails on a dependency
//! cycle or an ambiguous trait binding.

mod error;
mod function;
mod instance;
mod report;
mod structs;
mod traits;

// re-exports
pub use error::{InjectError, InjectResult};
pub use function::{__register_function, get_function_ptr, set_function_ptr};
pub use instance::get_instance;
pub use microrust_macros::{
    get_function, inject_fn, inject_trait, set_fake_async_function, set_fake_function, singleton,
};
pub use report::{dependency_cycles, report_registrations};
pub use structs::{
    __FactoryEntry, __InstanceFactory, __collect_struct, __collect_struct_deps,
    __collect_trait_deps,
};
pub use traits::{__collect_trait, get_trait_instance};
