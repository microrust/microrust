mod error;
mod function;
mod instance;
mod structs;
mod traits;

// re-exports
pub use crosstrait::register;
pub use error::{InjectError, InjectResult};
pub use function::{__register_function, get_function_ptr, set_function_ptr};
pub use instance::get_instance;
pub use microrust_macros::{
    get_function, inject_fake_trait, inject_fn, inject_trait, set_fake_async_function,
    set_fake_function, singleton,
};
pub use structs::{
    __InstanceFactory, __collect_struct, __collect_struct_deps, __collect_trait_deps,
};
pub use traits::{__FakeTrait, __collect_trait, get_trait_instance, set_fake_trait_instance};
