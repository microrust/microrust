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
