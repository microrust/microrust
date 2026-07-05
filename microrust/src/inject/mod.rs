mod instance;
mod structs;
mod traits;

// re-exports
pub use crosstrait::register;
pub use instance::get_instance;
pub use microrust_macros::{inject_trait, singleton};
pub use structs::{__InstanceFactory, __collect_struct};
pub use traits::{__collect_trait, get_trait_instance};
