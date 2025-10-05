use crosstrait::Cast;
use std::{
    any::{Any, TypeId},
    sync::{Arc, RwLock},
};

use crate::{
    instance::__get_instance_by_typeid, structs::__STRUCT_REGISTRY, traits::__TRAIT_REGISTRY,
};

#[doc(hidden)]
pub trait __InstanceRwLockFactory: Send + Sync {
    fn __create_instance() -> Arc<dyn Any + Send + Sync>
    where
        Self: Send + Sync + Sized;
}

#[doc(hidden)]
pub fn __collect_struct_rwlock<T: __InstanceRwLockFactory + Send + Sync + 'static>() {
    println!("__collect_struct_rwlock() called");

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
pub fn __collect_trait_rwlock<
    T: Send + Sync + ?Sized + 'static,
    U: __InstanceRwLockFactory + Send + Sync + 'static,
>() {
    println!("__collect_trait_rwlock() called");

    let type_id_trait = TypeId::of::<T>();
    // TODO: throw error, if multiple struct implement the same trait
    // exception: they are qualified?
    if __TRAIT_REGISTRY.contains_key(&type_id_trait) {
        println!("trait rwlock allready collected");
        return;
    }

    let type_id_struct = TypeId::of::<U>();
    __TRAIT_REGISTRY.insert(type_id_trait, type_id_struct);
}

pub fn get_instance_rwlock<T: __InstanceRwLockFactory + Send + Sync + 'static>() -> Arc<RwLock<T>> {
    println!("get_instance_rwlock() called");

    let type_id = TypeId::of::<T>();
    let inst = __get_instance_by_typeid(type_id);
    let cast = inst.downcast::<RwLock<T>>().unwrap();
    cast
}

pub fn get_trait_instance_rwlock<T: Send + Sync + ?Sized + 'static>() -> Arc<RwLock<T>> {
    println!("get_trait_instance_rwlock() called");

    let type_id_trait = TypeId::of::<T>();
    // // TODO: make sure, that trait is implemented for struct
    let type_id_struct = __TRAIT_REGISTRY.get(&type_id_trait).unwrap();
    let inst = __get_instance_by_typeid(*type_id_struct);
    // let cast = inst.downcast::<T>().unwrap();
    // cast
    let cast: Arc<RwLock<T>> = inst.cast().unwrap();
    cast
}
