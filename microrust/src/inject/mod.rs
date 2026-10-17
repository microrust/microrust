//! Dependency injection: singleton structs and trait bindings.
//!
//! Two registries back this module, both process-global and keyed by `TypeId`:
//!
//! | Dependency | Key | Resolved by |
//! |---|---|---|
//! | struct | `TypeId` | [`get_instance`] |
//! | trait | `TypeId` of `dyn Trait` | [`get_trait_instance`] |
//!
//! Registration happens in `#[ctor]` functions emitted by [`singleton`] and
//! [`inject_trait`]; resolution happens on first request.
//! Nothing is constructed during registration.
//!
//! [`report_registrations`] logs what was registered and fails on a dependency
//! cycle or an ambiguous trait binding.

mod instance;
mod report;
mod structs;
mod traits;

// re-exports
pub use instance::get_instance;
pub use microrust_macros::{inject_trait, singleton};
pub use report::{dependency_cycles, report_registrations};
pub use structs::{
    __FactoryEntry, __InstanceFactory, __collect_struct, __collect_struct_deps,
    __collect_trait_deps,
};
pub use traits::{__collect_trait, get_trait_instance};
