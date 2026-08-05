use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

static __STRUCT_REGISTRY: LazyLock<
    Mutex<HashMap<TypeId, Arc<fn() -> Arc<dyn Any + Send + Sync>>>>,
> = LazyLock::new(|| Mutex::new(HashMap::<_, _>::new()));

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
    if __contains_function_pointer(&type_id) {
        println!("struct allready collected");
        return;
    }

    let fp = T::__create_instance;
    __insert_function_pointer(type_id, fp);
}

#[doc(hidden)]
pub fn __get_function_pointer(type_id: TypeId) -> Arc<fn() -> Arc<dyn Any + Send + Sync>> {
    println!("__get_function_pointer() called");

    let map = __STRUCT_REGISTRY.lock().unwrap();
    let value = map.get(&type_id).unwrap();
    Arc::clone(&value)
}

#[doc(hidden)]
pub fn __contains_function_pointer(type_id: &TypeId) -> bool {
    let map = __STRUCT_REGISTRY.lock().unwrap();
    map.contains_key(type_id)
}

#[doc(hidden)]
pub fn __insert_function_pointer(
    type_id: TypeId,
    fp: fn() -> Arc<dyn Any + Send + Sync + 'static>,
) {
    // Todo Arc:pin(fp)
    let mut map = __STRUCT_REGISTRY.lock().unwrap();
    map.insert(type_id, Arc::new(fp));
}
