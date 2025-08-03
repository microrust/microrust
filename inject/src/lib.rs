use crosstrait::Cast;
use dashmap::DashMap;
use std::{
    any::{Any, TypeId},
    sync::{Arc, LazyLock, Mutex, RwLock},
};

// re-export
#[doc(hidden)]
pub use crosstrait::register;
#[doc(hidden)]
pub use ctor::__support::ctor_parse;
pub use microrust_inject_macros::*;
// pub use microrust_inject_macros::{inject_request, inject_singleton, inject_trait};

static __STRUCT_REGISTRY: LazyLock<DashMap<TypeId, Arc<fn() -> Arc<dyn Any + Send + Sync>>>> =
    LazyLock::new(DashMap::<_, _>::new);
static __TRAIT_REGISTRY: LazyLock<DashMap<TypeId, TypeId>> = LazyLock::new(DashMap::<_, _>::new);
static __SINGLETON_INSTANCES: LazyLock<DashMap<TypeId, Arc<dyn Any + Send + Sync>>> =
    LazyLock::new(DashMap::<_, _>::new);

#[doc(hidden)]
pub trait __InstanceFactory: Send + Sync {
    fn __create_instance() -> Arc<dyn Any + Send + Sync>
    where
        Self: Send + Sync + Sized;
}

#[doc(hidden)]
pub trait __InstanceMutexFactory: Send + Sync {
    fn __create_instance() -> Arc<dyn Any + Send + Sync>
    where
        Self: Send + Sync + Sized;
}

#[doc(hidden)]
#[doc(hidden)]
pub trait __InstanceRwLockFactory: Send + Sync {
    fn __create_instance() -> Arc<dyn Any + Send + Sync>
    where
        Self: Send + Sync + Sized;
}

#[doc(hidden)]
pub fn __collect_struct<T: __InstanceFactory + Send + Sync + 'static>() {
    println!("__collect_struct() called");

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
pub fn __collect_struct_mutex<T: __InstanceMutexFactory + Send + Sync + 'static>() {
    println!("__collect_struct_mutex() called");

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

#[doc(hidden)]
pub fn __collect_trait_mutex<
    T: Send + Sync + ?Sized + 'static,
    U: __InstanceMutexFactory + Send + Sync + 'static,
>() {
    println!("__collect_trait_mutex() called");

    let type_id_trait = TypeId::of::<T>();
    // TODO: throw error, if multiple struct implement the same trait
    // exception: they are qualified?
    if __TRAIT_REGISTRY.contains_key(&type_id_trait) {
        println!("trait mutex allready collected");
        return;
    }

    let type_id_struct = TypeId::of::<U>();
    __TRAIT_REGISTRY.insert(type_id_trait, type_id_struct);
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

#[doc(hidden)]
fn __get_function_pointer(type_id: TypeId) -> Arc<fn() -> Arc<dyn Any + Send + Sync>> {
    println!("__get_function_pointer() called");

    let value = __STRUCT_REGISTRY.get(&type_id).unwrap();
    value.clone()
}

#[doc(hidden)]
fn __get_instance_by_typeid(type_id: TypeId) -> Arc<dyn Any + Send + Sync> {
    println!("__get_instance_by_typeid() called");

    if __SINGLETON_INSTANCES.contains_key(&type_id) {
        println!("singleton instance found");
        let inst = __SINGLETON_INSTANCES.get(&type_id).unwrap();
        let clone = inst.clone();
        return clone;
    }

    println!("creating singleton instance");
    let fp = __get_function_pointer(type_id);
    let inst = fp();
    // let clone = inst.clone();
    __SINGLETON_INSTANCES.insert(type_id, inst.clone());
    inst
}

pub fn get_instance<T: __InstanceFactory + Send + Sync + 'static>() -> Arc<T> {
    println!("get_instance() called");

    let type_id = TypeId::of::<T>();
    let inst = __get_instance_by_typeid(type_id);
    let cast = inst.downcast::<T>().unwrap();
    cast
}

pub fn get_instance_mutex<T: __InstanceMutexFactory + Send + Sync + 'static>() -> Arc<Mutex<T>> {
    println!("get_instance_mutex() called");

    let type_id = TypeId::of::<T>();
    let inst = __get_instance_by_typeid(type_id);
    let cast = inst.downcast::<Mutex<T>>().unwrap();
    cast
}

pub fn get_instance_rwlock<T: __InstanceRwLockFactory + Send + Sync + 'static>() -> Arc<RwLock<T>> {
    println!("get_instance_rwlock() called");

    let type_id = TypeId::of::<T>();
    let inst = __get_instance_by_typeid(type_id);
    let cast = inst.downcast::<RwLock<T>>().unwrap();
    cast
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

pub fn get_trait_instance_mutex<T: Send + Sync + ?Sized + 'static>() -> Arc<Mutex<T>> {
    println!("get_trait_instance_mutex() called");

    let type_id_trait = TypeId::of::<T>();
    // // TODO: make sure, that trait is implemented for struct
    let type_id_struct = __TRAIT_REGISTRY.get(&type_id_trait).unwrap();
    let inst = __get_instance_by_typeid(*type_id_struct);
    // let cast = inst.downcast::<T>().unwrap();
    // cast
    let cast: Arc<Mutex<T>> = inst.cast().unwrap();
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
