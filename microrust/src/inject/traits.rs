use std::{
    any::{Any, TypeId, type_name, type_name_of_val},
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};
use tracing::{debug, instrument, warn};

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

/// A registered trait binding: the erased bridge plus the names to report it
/// under. The registry is keyed by `TypeId`, so without captured names the
/// binding would be unprintable once registered.
pub(crate) struct __TraitEntry {
    pub name: &'static str,
    pub impl_name: &'static str,
    pub bridge: __Bridge,
}

/// A trait claimed by more than one struct.
///
/// Recorded rather than rejected at registration: `ctor` functions run before
/// `main` with no `tracing` subscriber installed, so nothing said here would be
/// seen. [`report_registrations`](crate::inject::report_registrations) raises
/// these alongside dependency cycles, once a subscriber is listening.
pub(crate) struct __TraitCollision {
    pub trait_name: &'static str,
    pub first_impl: &'static str,
    pub second_impl: &'static str,
}

pub(crate) static __TRAIT_COLLISIONS: LazyLock<Mutex<Vec<__TraitCollision>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

pub(crate) static __TRAIT_REGISTRY: LazyLock<Mutex<HashMap<TypeId, __TraitEntry>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Registers `resolve` as the way to obtain an `Arc<T>` for trait `T`.
///
/// `resolve` must return `get_instance::<U>()` coerced to `Arc<T>` for the
/// struct `U` that implements `T`, so both views share one instance.
/// `impl_name` is `U`'s name, used to report an ambiguous binding.
/// Emitted by `#[inject_trait]` and run once at startup.
///
/// If a second struct implements the same trait, the first registration is kept
/// and the clash is recorded. There is no qualification syntax yet, so every
/// such clash is a wiring mistake rather than an intentional override.
#[doc(hidden)]
pub fn __collect_trait<T>(impl_name: &'static str, resolve: fn() -> Arc<T>)
where
    T: Send + Sync + ?Sized + 'static,
{
    debug!("__collect_trait() called for {}", type_name::<T>());

    let type_id_trait = TypeId::of::<T>();

    // The registry lock is released before recording a collision so the two
    // mutexes are never held at once.
    let colliding_with = {
        let mut map = __TRAIT_REGISTRY.lock().unwrap();
        match map.get(&type_id_trait) {
            Some(existing) => Some(existing.impl_name),
            None => {
                map.insert(
                    type_id_trait,
                    __TraitEntry {
                        name: type_name::<T>(),
                        impl_name,
                        bridge: Arc::new(__TraitBridge::<T> { resolve }) as __Bridge,
                    },
                );
                None
            }
        }
    };

    if let Some(first_impl) = colliding_with {
        __record_trait_collision(__TraitCollision {
            trait_name: type_name::<T>(),
            first_impl,
            second_impl: impl_name,
        });
    }
}

fn __record_trait_collision(collision: __TraitCollision) {
    // Discarded in the `ctor` phase for the reason given on
    // `__TRAIT_COLLISIONS`; kept so the clash is still visible if registration
    // ever moves to runtime.
    warn!(
        "trait {} is implemented by both {} and {}; only the first is reachable \
         through the trait",
        collision.trait_name,
        collision.first_impl,
        collision.second_impl,
    );

    __TRAIT_COLLISIONS.lock().unwrap().push(collision);
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
        let entry = map
            .get(&type_id_trait)
            .expect("no implementation is registered for this trait");
        Arc::clone(&entry.bridge)
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
