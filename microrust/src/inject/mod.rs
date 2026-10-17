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
//!
//! # Constructor rules
//!
//! A `#[singleton]` constructor must return `Self`. Anything else is rejected
//! at compile time rather than surfacing as a failed downcast on the first
//! [`get_instance`]:
//!
//! ```compile_fail,E0308
//! use microrust::inject::singleton;
//!
//! struct Other;
//! struct Service;
//!
//! #[singleton(new)]
//! impl Service {
//!     fn new() -> Other {
//!         Other
//!     }
//! }
//! ```
//!
//! It must also be synchronous. Resolution can happen outside any async
//! runtime, so there is nothing to drive a constructor's future:
//!
//! ```compile_fail
//! use microrust::inject::singleton;
//!
//! struct Service;
//!
//! #[singleton(new)]
//! impl Service {
//!     async fn new() -> Self {
//!         Self
//!     }
//! }
//! ```
//!
//! A generic `impl` is rejected, since registration needs one concrete type.
//! Implement it for a concrete instantiation instead (`impl Wrapper<u32>`):
//!
//! ```compile_fail
//! use microrust::inject::singleton;
//!
//! struct Wrapper<T>(T);
//!
//! #[singleton(new)]
//! impl<T: Default + Send + Sync + 'static> Wrapper<T> {
//!     fn new() -> Self {
//!         Self(T::default())
//!     }
//! }
//! ```
//!
//! A trait dependency names exactly one trait. `Arc<dyn Trait + Send + Sync>`
//! is a different type from `Arc<dyn Trait>` and would never match the
//! binding, so it is rejected; declare `Send + Sync` as supertraits instead:
//!
//! ```compile_fail
//! use std::sync::Arc;
//! use microrust::inject::singleton;
//!
//! trait Repo {}
//! struct Service;
//!
//! #[singleton(new)]
//! impl Service {
//!     fn new(repo: Arc<dyn Repo + Send + Sync>) -> Self {
//!         Self
//!     }
//! }
//! ```
//!
//! `#[inject_trait]` takes no arguments:
//!
//! ```compile_fail
//! use microrust::inject::{inject_trait, singleton};
//!
//! trait Repo: Send + Sync {}
//! struct Postgres;
//!
//! #[singleton(new)]
//! impl Postgres {
//!     fn new() -> Self {
//!         Self
//!     }
//! }
//!
//! #[inject_trait(primary)]
//! impl Repo for Postgres {}
//! ```

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
