mod instance;
mod mutex;
mod rwlock;
mod structs;
mod traits;

// re-exports
pub use crosstrait::register;
pub use instance::get_instance;
pub use microrust_macros::{
    inject_trait, inject_trait_mutex, inject_trait_rwlock, singleton, singleton_mutex,
    singleton_rwlock,
};
pub use mutex::{
    __InstanceMutexFactory, __collect_struct_mutex, __collect_trait_mutex, get_instance_mutex,
    get_trait_instance_mutex,
};
pub use rwlock::{
    __InstanceRwLockFactory, __collect_struct_rwlock, __collect_trait_rwlock, get_instance_rwlock,
    get_trait_instance_rwlock,
};
pub use structs::{__InstanceFactory, __collect_struct};
pub use traits::{__collect_trait, get_trait_instance};
