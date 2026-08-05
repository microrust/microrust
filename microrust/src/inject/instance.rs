use std::{
    any::{Any, TypeId, type_name},
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};
use tracing::{debug, instrument};

pub use crate::inject::structs::{__InstanceFactory, __get_function_pointer};

static __SINGLETON_INSTANCES: LazyLock<Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>> =
    LazyLock::new(|| Mutex::new(HashMap::<_, _>::new()));

#[doc(hidden)]
pub fn __get_instance_by_typeid(type_id: TypeId) -> Arc<dyn Any + Send + Sync> {
    let map = __SINGLETON_INSTANCES.lock().unwrap();
    if map.contains_key(&type_id) {
        debug!("singleton instance found");
        let inst = map.get(&type_id).unwrap();
        let clone = Arc::clone(&inst);
        return clone;
    }
    drop(map); // Release lock

    debug!("creating singleton instance");
    let fp = __get_function_pointer(type_id);
    let inst = fp();
    let mut map = __SINGLETON_INSTANCES.lock().unwrap();
    map.insert(type_id, Arc::clone(&inst));
    drop(map); // Release lock
    inst
}

#[instrument]
pub fn get_instance<T: __InstanceFactory + Send + Sync + 'static>() -> Arc<T> {
    debug!("Type: {:#}", type_name::<T>());

    let type_id = TypeId::of::<T>();
    let inst = __get_instance_by_typeid(type_id);
    inst.downcast::<T>().unwrap()
}
