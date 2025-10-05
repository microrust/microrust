use dashmap::DashMap;
use std::{
    any::{Any, TypeId},
    sync::{Arc, LazyLock},
};

pub use crate::structs::{__InstanceFactory, __get_function_pointer};

static __SINGLETON_INSTANCES: LazyLock<DashMap<TypeId, Arc<dyn Any + Send + Sync>>> =
    LazyLock::new(DashMap::<_, _>::new);

#[doc(hidden)]
pub fn __get_instance_by_typeid(type_id: TypeId) -> Arc<dyn Any + Send + Sync> {
    println!("__get_instance_by_typeid() called");

    if __SINGLETON_INSTANCES.contains_key(&type_id) {
        println!("singleton instance found");
        let inst = __SINGLETON_INSTANCES.get(&type_id).unwrap();
        let clone = inst.clone();
        return clone;
    }

    println!("creating singleton instance");
    let fp = __get_function_pointer(type_id);
    let inst = fp();
    // let clone = inst.clone();
    __SINGLETON_INSTANCES.insert(type_id, inst.clone());
    inst
}

pub fn get_instance<T: __InstanceFactory + Send + Sync + 'static>() -> Arc<T> {
    println!("get_instance() called");

    let type_id = TypeId::of::<T>();
    let inst = __get_instance_by_typeid(type_id);
    let cast = inst.downcast::<T>().unwrap();
    cast
}
