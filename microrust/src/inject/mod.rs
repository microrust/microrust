mod instance;
mod structs;
mod tokio_mutex;
mod tokio_rwlock;
mod traits;

// re-exports
pub use crosstrait::register;
pub use instance::get_instance;
pub use microrust_macros::{
    inject_trait, inject_trait_tokio_mutex, inject_trait_tokio_rwlock, singleton,
    singleton_std_mutex, singleton_std_rwlock, singleton_tokio_mutex, singleton_tokio_rwlock,
};
pub use structs::{__InstanceFactory, __collect_struct};
pub use tokio_mutex::{
    __InstanceMutexFactory, __collect_struct_mutex, __collect_trait_mutex, get_instance_mutex,
    get_trait_instance_mutex,
};
pub use tokio_rwlock::{
    __InstanceRwLockFactory, __collect_struct_rwlock, __collect_trait_rwlock, get_instance_rwlock,
    get_trait_instance_rwlock,
};
pub use traits::{__collect_trait, get_trait_instance};
