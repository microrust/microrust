use std::{
    any::{Any, TypeId, type_name, type_name_of_val},
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};
use tracing::{debug, instrument};

pub use crate::inject::structs::{__InstanceFactory, __get_function_pointer};

static __SINGLETON_INSTANCES: LazyLock<Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>> =
    LazyLock::new(|| Mutex::new(HashMap::<_, _>::new()));

#[doc(hidden)]
pub fn __get_instance_by_typeid(type_id: TypeId) -> Arc<dyn Any + Send + Sync> {
    debug!("__get_instance_by_typeid() called");

    debug!("__SINGLETON_INSTANCES map gets locked");
    let mut map = __SINGLETON_INSTANCES.lock().unwrap();
    debug!("__SINGLETON_INSTANCES map locked");
    if map.contains_key(&type_id) {
        debug!("singleton instance found");
        let inst = map.get(&type_id).unwrap();
        let clone = Arc::clone(&inst);
        return clone;
    }
    // drop(map); // Release lock

    debug!("creating singleton instance");
    // let mut map = __SINGLETON_INSTANCES.lock().unwrap();
    let fp = __get_function_pointer(type_id);
    let inst = fp();
    map.insert(type_id, Arc::clone(&inst));
    debug!("__SINGLETON_INSTANCES map gets unlocked");
    drop(map); // Release lock
    debug!("__SINGLETON_INSTANCES map unlocked");
    inst
}

#[instrument]
pub fn get_instance<T: __InstanceFactory + Send + Sync + 'static>() -> Arc<T> {
    debug!("get_instance() called");

    let type_id_struct = TypeId::of::<T>();
    debug!("Input Struct TypeName: {:#}", type_name::<T>());
    debug!("Input Struct TypeId: {:?}", type_id_struct);

    let inst = __get_instance_by_typeid(type_id_struct);
    let casted = inst.downcast::<T>().unwrap();
    debug!("Output Struct TypeName: {:#}", type_name_of_val(&*casted));
    debug!("Output Struct TypeId: {:?}", (*casted).type_id());
    debug!("Output Struct Ref: {:p}", &(*casted));

    casted
}
