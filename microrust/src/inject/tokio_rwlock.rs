use crosstrait::Cast;
use std::{
    any::{Any, TypeId, type_name},
    sync::Arc,
};
use tokio::sync::RwLock;
use tracing::{debug, instrument};

use crate::{
    inject::instance::__get_instance_by_typeid,
    inject::structs::{__contains_function_pointer, __insert_function_pointer},
    inject::traits::{__contains_trait, __get_type_id_struct, __insert_trait},
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
    if __contains_function_pointer(&type_id) {
        println!("struct allready collected");
        return;
    }

    let fp = T::__create_instance;
    // Todo Arc:pin(fp)
    __insert_function_pointer(type_id, fp);
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
    if __contains_trait(&type_id_trait) {
        println!("trait rwlock allready collected");
        return;
    }

    let type_id_struct = TypeId::of::<U>();
    __insert_trait(type_id_trait, type_id_struct);
}

#[instrument]
pub fn get_instance_rwlock<T: __InstanceRwLockFactory + Send + Sync + 'static>() -> Arc<RwLock<T>> {
    debug!("Type: {:#}", type_name::<T>());

    let type_id = TypeId::of::<T>();
    let inst = __get_instance_by_typeid(type_id);
    inst.downcast::<RwLock<T>>().unwrap()
}

#[instrument]
pub fn get_trait_instance_rwlock<T: Send + Sync + ?Sized + 'static>() -> Arc<RwLock<T>> {
    debug!("Type: {:#}", type_name::<T>());

    let type_id_trait = TypeId::of::<T>();
    // // TODO: make sure, that trait is implemented for struct
    let type_id_struct = __get_type_id_struct(&type_id_trait);
    let inst = __get_instance_by_typeid(type_id_struct);
    // let cast = inst.downcast::<T>().unwrap();
    // cast
    let cast: Arc<RwLock<T>> = inst.cast().unwrap();
    cast
}
