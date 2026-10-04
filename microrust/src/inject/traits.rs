use crosstrait::Cast;
use std::{
    any::{Any, TypeId, type_name, type_name_of_val},
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};
use tracing::{debug, instrument};

use crate::{inject::instance::__get_instance_by_typeid, inject::structs::__InstanceFactory};

static __TRAIT_REGISTRY: LazyLock<Mutex<HashMap<TypeId, TypeId>>> =
    LazyLock::new(|| Mutex::new(HashMap::<_, _>::new()));

#[doc(hidden)]
pub fn __collect_trait<
    T: Send + Sync + ?Sized + 'static,
    U: __InstanceFactory + Send + Sync + 'static,
>() {
    println!("__collect_trait() called");

    let type_id_trait = TypeId::of::<T>();
    println!("Trait TypeName: {:#}", type_name::<T>());
    println!("Trait TypeId: {:?}", type_id_trait);

    // TODO: throw error, if multiple struct implement the same trait
    // exception: they are qualified?
    if __contains_trait(&type_id_trait) {
        println!("trait allready collected");
        return;
    }

    let type_id_struct = TypeId::of::<U>();
    println!("Struct TypeName: {:#}", type_name::<U>());
    println!("Struct TypeId: {:?}", type_id_struct);

    __insert_trait(type_id_trait, type_id_struct);
}

#[doc(hidden)]
pub fn __get_type_id_struct(type_id_trait: &TypeId) -> TypeId {
    let map = __TRAIT_REGISTRY.lock().unwrap();
    let value = map.get(type_id_trait).unwrap();
    *value // value.clone() ?
}

#[doc(hidden)]
pub fn __contains_trait(type_id_trait: &TypeId) -> bool {
    let map = __TRAIT_REGISTRY.lock().unwrap();
    map.contains_key(type_id_trait)
}

#[doc(hidden)]
pub fn __insert_trait(type_id_trait: TypeId, type_id_struct: TypeId) {
    let mut map = __TRAIT_REGISTRY.lock().unwrap();
    map.insert(type_id_trait, type_id_struct);
}

#[instrument]
pub fn get_trait_instance<T: Send + Sync + ?Sized + 'static>() -> Arc<T> {
    debug!("get_trait_instance() called");

    let type_id_trait = TypeId::of::<T>();
    debug!("Input Trait TypeName: {:#}", type_name::<T>());
    debug!("Input Trait TypeId: {:?}", type_id_trait);

    // // TODO: make sure, that trait is implemented for struct
    let type_id_struct = {
        let map = __TRAIT_REGISTRY.lock().unwrap();
        *map.get(&type_id_trait).unwrap()
    };
    let inst = __get_instance_by_typeid(type_id_struct);
    let casted: Arc<T> = inst.cast().unwrap();
    debug!("Output Trait TypeName: {:#}", type_name_of_val(&*casted));
    debug!("Output Trait TypeId: {:?}", (*casted).type_id());
    debug!("Output Trait Ref: {:p}", &(*casted));

    casted
}
