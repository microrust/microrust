use dashmap::DashMap;
use std::{
    any::{Any, TypeId},
    sync::{Arc, LazyLock},
};

pub static __STRUCT_REGISTRY: LazyLock<DashMap<TypeId, Arc<fn() -> Arc<dyn Any + Send + Sync>>>> =
    LazyLock::new(DashMap::<_, _>::new);

#[doc(hidden)]
pub trait __InstanceFactory: Send + Sync {
    fn __create_instance() -> Arc<dyn Any + Send + Sync>
    where
        Self: Send + Sync + Sized;
}

#[doc(hidden)]
pub fn __collect_struct<T: __InstanceFactory + Send + Sync + 'static>() {
    println!("__collect_struct() called");

    let type_id = TypeId::of::<T>();
    if __STRUCT_REGISTRY.contains_key(&type_id) {
        println!("struct allready collected");
        return;
    }

    let fp = T::__create_instance;
    // Todo Arc:pin(fp)
    __STRUCT_REGISTRY.insert(type_id, Arc::new(fp));
}

#[doc(hidden)]
pub fn __get_function_pointer(type_id: TypeId) -> Arc<fn() -> Arc<dyn Any + Send + Sync>> {
    println!("__get_function_pointer() called");

    let value = __STRUCT_REGISTRY.get(&type_id).unwrap();
    value.clone()
}
