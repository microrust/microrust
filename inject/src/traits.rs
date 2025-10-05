use crosstrait::Cast;
use dashmap::DashMap;
use std::{
    any::TypeId,
    sync::{Arc, LazyLock},
};

use crate::{instance::__get_instance_by_typeid, structs::__InstanceFactory};

pub static __TRAIT_REGISTRY: LazyLock<DashMap<TypeId, TypeId>> =
    LazyLock::new(DashMap::<_, _>::new);

#[doc(hidden)]
pub fn __collect_trait<
    T: Send + Sync + ?Sized + 'static,
    U: __InstanceFactory + Send + Sync + 'static,
>() {
    println!("__collect_trait() called");

    let type_id_trait = TypeId::of::<T>();
    // TODO: throw error, if multiple struct implement the same trait
    // exception: they are qualified?
    if __TRAIT_REGISTRY.contains_key(&type_id_trait) {
        println!("trait allready collected");
        return;
    }

    let type_id_struct = TypeId::of::<U>();
    __TRAIT_REGISTRY.insert(type_id_trait, type_id_struct);
}

pub fn get_trait_instance<T: Send + Sync + ?Sized + 'static>() -> Arc<T> {
    println!("get_trait_instance() called");

    let type_id_trait = TypeId::of::<T>();
    // // TODO: make sure, that trait is implemented for struct
    let type_id_struct = __TRAIT_REGISTRY.get(&type_id_trait).unwrap();
    let inst = __get_instance_by_typeid(*type_id_struct);
    // let cast = inst.downcast::<T>().unwrap();
    // cast
    let cast: Arc<T> = inst.cast().unwrap();
    cast
}
