use std::{
    any::{Any, TypeId, type_name},
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

use petgraph::graph::DiGraph;
use tracing::debug;

#[doc(hidden)]
pub type __AnyArc = Arc<dyn Any + Send + Sync>;
#[doc(hidden)]
pub type __FactoryFn = fn() -> __AnyArc;
#[doc(hidden)]
pub type __FactoryMap = HashMap<TypeId, Arc<__FactoryFn>>;

static __STRUCT_REGISTRY: LazyLock<Mutex<__FactoryMap>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static __STRUCT_DEPS: LazyLock<Mutex<DiGraph<TypeId, ()>>> =
    LazyLock::new(|| Mutex::new(DiGraph::new()));

#[doc(hidden)]
pub trait __InstanceFactory: Send + Sync {
    fn __create_instance() -> __AnyArc
    where
        Self: Send + Sync + Sized;
}

#[doc(hidden)]
pub fn __collect_struct<T: __InstanceFactory + Send + Sync + 'static>() {
    println!("__collect_struct() called");

    let type_id_struct = TypeId::of::<T>();
    println!("Struct TypeName: {:#}", type_name::<T>());
    println!("Struct TypeId: {:?}", type_id_struct);

    if __contains_function_pointer(&type_id_struct) {
        println!("struct allready collected");
        return;
    }

    let fp = T::__create_instance;
    __insert_function_pointer(type_id_struct, fp);
}

#[doc(hidden)]
pub fn __collect_struct_deps<
    T: __InstanceFactory + Send + Sync + 'static,
    U: __InstanceFactory + Send + Sync + 'static,
>() {
    println!("__collect_struct_deps() called");

    let type_id_struct_from = TypeId::of::<T>();
    println!("From Struct TypeName: {:#}", type_name::<T>());
    println!("From Struct TypeId: {:?}", type_id_struct_from);

    let type_id_struct_to = TypeId::of::<U>();
    println!("To Struct TypeName: {:#}", type_name::<U>());
    println!("To Struct TypeId: {:?}", type_id_struct_to);

    let mut graph = __STRUCT_DEPS.lock().unwrap();
    let from = graph.add_node(type_id_struct_from);
    let to = graph.add_node(type_id_struct_to);
    graph.add_edge(from, to, ());
}

#[doc(hidden)]
pub fn __get_function_pointer(type_id: TypeId) -> Arc<__FactoryFn> {
    debug!("__get_function_pointer() called");

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
    fp: __FactoryFn,
) {
    // Todo Arc:pin(fp)
    let mut map = __STRUCT_REGISTRY.lock().unwrap();
    map.insert(type_id, Arc::new(fp));
}
