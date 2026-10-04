use std::{
    any::{Any, TypeId, type_name, type_name_of_val},
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};
use tracing::{debug, instrument};

/// Erased resolver for a single trait.
///
/// Turning `Arc<Struct>` into `Arc<dyn Trait>` is an unsizing coercion, which
/// only type-checks where both concrete types are known. So `#[inject_trait]`
/// passes the closure in already monomorphized and this struct only stores it.
/// Being a plain `fn` pointer it is `Copy`, so it is cheap to call per lookup
/// and the singleton registry stays the single owner of the instance.
struct __TraitBridge<T: ?Sized> {
    resolve: fn() -> Arc<T>,
}

/// Erased `__TraitBridge<dyn Trait>`, keyed in the registry by the trait's
/// `TypeId` so heterogeneous traits can share one map.
type __Bridge = Arc<dyn Any + Send + Sync>;

static __TRAIT_REGISTRY: LazyLock<Mutex<HashMap<TypeId, __Bridge>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Registers `resolve` as the way to obtain an `Arc<T>` for trait `T`.
///
/// `resolve` must return `get_instance::<U>()` coerced to `Arc<T>` for the
/// struct `U` that implements `T`, so both views share one instance.
/// Emitted by `#[inject_trait]` and run once at startup.
#[doc(hidden)]
pub fn __collect_trait<T>(resolve: fn() -> Arc<T>)
where
    T: Send + Sync + ?Sized + 'static,
{
    debug!("__collect_trait() called for {}", type_name::<T>());

    let type_id_trait = TypeId::of::<T>();

    // TODO: throw error, if multiple struct implement the same trait
    // exception: they are qualified?
    let mut map = __TRAIT_REGISTRY.lock().unwrap();
    if map.contains_key(&type_id_trait) {
        debug!("trait allready collected");
        return;
    }

    map.insert(
        type_id_trait,
        Arc::new(__TraitBridge::<T> { resolve }) as __Bridge,
    );
}

#[doc(hidden)]
pub fn __contains_trait(type_id_trait: &TypeId) -> bool {
    let map = __TRAIT_REGISTRY.lock().unwrap();
    map.contains_key(type_id_trait)
}

#[instrument]
pub fn get_trait_instance<T: Send + Sync + ?Sized + 'static>() -> Arc<T> {
    debug!("get_trait_instance() called");

    let type_id_trait = TypeId::of::<T>();
    debug!("Input Trait TypeName: {:#}", type_name::<T>());

    // Hold a handle rather than a borrow: `resolve` re-enters the singleton
    // registry, so the trait lock must not be held across the call.
    let bridge: __Bridge = {
        let map = __TRAIT_REGISTRY.lock().unwrap();
        let bridge = map
            .get(&type_id_trait)
            .expect("no implementation is registered for this trait");
        Arc::clone(bridge)
    };

    let resolve = bridge
        .downcast_ref::<__TraitBridge<T>>()
        .expect("registered bridge does not match the requested trait")
        .resolve;

    let inst = resolve();
    debug!("Output Trait TypeName: {:#}", type_name_of_val(&*inst));
    debug!("Output Trait Ref: {:p}", &*inst);

    inst
}
