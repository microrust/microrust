use std::{
    any::{Any, TypeId, type_name},
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

use tracing::debug;

use crate::inject::error::InjectError;

#[doc(hidden)]
pub type __AnyArc = Arc<dyn Any + Send + Sync>;

struct __FnEntry {
    /// `TypeId` of the stored callable, used to reject mismatched mocks.
    sig: TypeId,
    /// Human-readable signature, for diagnostics only.
    sig_name: &'static str,
    value: __AnyArc,
}

type __FnRegistry = HashMap<String, __FnEntry>;

static __FUNCTION_REGISTRY: LazyLock<Mutex<__FnRegistry>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Registers a callable. Emitted by `#[inject_fn]` and run once at startup.
///
/// Generic rather than taking an already-erased `__AnyArc` so the concrete
/// signature is available for `TypeId` and diagnostics.
#[doc(hidden)]
pub fn __register_function<T: Any + Send + Sync + Copy + 'static>(key: &str, f: T) {
    debug!("__register_function() called for {key}");

    let sig = TypeId::of::<T>();
    let sig_name = type_name::<T>();
    let mut map = __FUNCTION_REGISTRY.lock().unwrap();

    if let Some(entry) = map.get(key)
        && entry.sig != sig
    {
        panic!(
            "duplicate microrust function key {key:?}: already registered as {}, \
             cannot re-register as {}",
            entry.sig_name, sig_name
        );
    }

    map.insert(
        key.to_string(),
        __FnEntry {
            sig,
            sig_name,
            value: Arc::new(f),
        },
    );
}

/// Returns the callable registered under `key`, typed as `T`.
///
/// `T` is the hidden `__MicrorustFn_*` alias emitted alongside the
/// `#[inject_fn]` function, which pins the exact signature.
pub fn get_function_ptr<T: Any + Send + Sync + Copy>(key: &str) -> Result<T, InjectError> {
    debug!("get_function_ptr() called for {key}");

    let map = __FUNCTION_REGISTRY.lock().unwrap();
    let entry = map
        .get(key)
        .ok_or_else(|| InjectError::FunctionNotRegistered {
            key: key.to_string(),
        })?;

    entry
        .value
        .downcast_ref::<T>()
        .copied()
        .ok_or_else(|| InjectError::SignatureMismatch {
            key: key.to_string(),
            expected: type_name::<T>(),
            found: entry.sig_name,
        })
}

/// Replaces the callable registered under `key`.
///
/// `T` is pinned by `set_fake_function!` to the hidden alias `#[inject_fn]`
/// emitted, so a mock whose signature does not match fails to compile rather
/// than failing at runtime.
pub fn set_function_ptr<T: Any + Send + Sync + Copy>(
    key: &str,
    f: T,
) -> Result<(), InjectError> {
    debug!("set_function_ptr() called for {key}");

    let sig = TypeId::of::<T>();
    let sig_name = type_name::<T>();
    let mut map = __FUNCTION_REGISTRY.lock().unwrap();

    match map.get_mut(key) {
        Some(entry) => {
            if entry.sig != sig {
                return Err(InjectError::SignatureMismatch {
                    key: key.to_string(),
                    expected: sig_name,
                    found: entry.sig_name,
                });
            }
            entry.value = Arc::new(f);
        }
        None => {
            map.insert(
                key.to_string(),
                __FnEntry {
                    sig,
                    sig_name,
                    value: Arc::new(f),
                },
            );
        }
    }
    Ok(())
}
