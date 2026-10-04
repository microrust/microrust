use std::{
    any::{Any, TypeId, type_name, type_name_of_val},
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex, OnceLock},
};
use tracing::{debug, instrument};

use crate::inject::{
    report::__assert_no_cycles,
    structs::{__InstanceFactory, __factory_entry},
};

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

    __assert_no_cycles();

    let cell = __instance_cell(type_id);
    if let Some(inst) = cell.get() {
        debug!("singleton instance found for {:?}", type_id);
        return Arc::clone(inst);
    }

    let inst = Arc::clone(
        cell.get_or_init(|| {
            debug!("creating singleton instance for {:?}", type_id);
            (__factory_entry(type_id).factory)()
        }),
    );

    debug!("singleton instance resolved for {:?}", type_id);
    inst
}

/// Returns the process-wide instance of `T`, constructing it on first request.
///
/// `T` must carry a `#[singleton]` attribute naming its constructor; the
/// constructor's own `Arc<...>` parameters are resolved recursively, so a graph
/// of singletons can be requested from any depth. Every call returns the same
/// allocation.
///
/// ```no_run
/// use microrust::inject::get_instance;
///
/// # struct Config;
/// # #[microrust::inject::singleton(new)]
/// # impl Config {
/// #     fn new() -> Self {
/// #         Self
/// #     }
/// # }
/// let config = get_instance::<Config>();
/// ```
///
/// # Panics
///
/// Panics if `T` has no `#[singleton]` registration, which means the type was
/// never wired into the container. Registration is emitted by the macro into a
/// `#[ctor]` function, so this only happens for a type the container has never
/// heard of.
///
/// Panics if the injection graph contains a dependency cycle. A cycle would
/// otherwise re-enter a `OnceLock` that is still initializing and deadlock
/// rather than fail; the check is why it surfaces as a message naming the
/// offending types. See [`report_registrations`](crate::inject::report_registrations).
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
