use std::{
    any::{Any, TypeId, type_name, type_name_of_val},
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex, OnceLock},
};
use tracing::{debug, instrument};

pub use crate::inject::structs::{__InstanceFactory, __get_function_pointer};

type __AnyArc = Arc<dyn Any + Send + Sync>;
type __AnyCell = Arc<OnceLock<__AnyArc>>;
type __InstanceMap = HashMap<TypeId, __AnyCell>;

static __SINGLETON_INSTANCES: LazyLock<Mutex<__InstanceMap>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn __instance_cell(type_id: TypeId) -> __AnyCell {
    let mut map = __SINGLETON_INSTANCES.lock().unwrap();
    Arc::clone(map.entry(type_id).or_default())
}

#[doc(hidden)]
pub fn __get_instance_by_typeid(type_id: TypeId) -> __AnyArc {
    debug!("__get_instance_by_typeid() called for {:?}", type_id);

    let cell = __instance_cell(type_id);
    if let Some(inst) = cell.get() {
        debug!("singleton instance found for {:?}", type_id);
        return Arc::clone(inst);
    }

    let inst = Arc::clone(
        cell.get_or_init(|| {
            debug!("creating singleton instance for {:?}", type_id);
            __get_function_pointer(type_id)()
        }),
    );

    debug!("singleton instance resolved for {:?}", type_id);
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
