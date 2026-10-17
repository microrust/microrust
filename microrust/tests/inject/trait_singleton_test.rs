use std::time::Instant;

use async_trait::async_trait;
use microrust::inject::{get_trait_instance, inject_trait, singleton};
use tracing_test::traced_test;

struct MyStruct {
    pub creation_time: Instant,
}

#[async_trait]
trait MyTrait: Send + Sync {
    fn creation_time_sync(&self) -> Instant;
    async fn creation_time_async(&self) -> Instant;
}

#[singleton(new)]
impl MyStruct {
    pub fn new() -> Self {
        Self {
            creation_time: Instant::now(),
        }
    }
}

#[inject_trait]
impl MyTrait for MyStruct {
    fn creation_time_sync(&self) -> Instant {
        self.creation_time
    }
    async fn creation_time_async(&self) -> Instant {
        self.creation_time
    }
}

#[test]
#[traced_test]
fn same_trait_singleton_instance_sync() {
    let inst = get_trait_instance::<dyn MyTrait>();
    let inst2 = get_trait_instance::<dyn MyTrait>();
    assert_eq!(inst.creation_time_sync(), inst2.creation_time_sync());
}

#[microrust::test]
#[traced_test]
async fn same_trait_singleton_instance_async() {
    let inst = get_trait_instance::<dyn MyTrait>();
    let inst2 = get_trait_instance::<dyn MyTrait>();
    assert_eq!(
        inst.creation_time_async().await,
        inst2.creation_time_async().await
    );
}

/// Bound by nothing, so resolving it must panic.
trait UnboundTrait: Send + Sync {}

/// A failed lookup must not take the registry down with it. The panic used to
/// be raised while the trait registry lock was held, poisoning it, so every
/// later `get_trait_instance` in the process failed with `PoisonError`.
#[test]
#[traced_test]
fn missing_trait_binding_does_not_break_later_lookups() {
    let payload = std::panic::catch_unwind(get_trait_instance::<dyn UnboundTrait>)
        .err()
        .expect("an unbound trait must not resolve");
    let message = payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&str>().copied())
        .unwrap_or_default();
    assert_eq!(message, "no implementation is registered for this trait");

    let inst = get_trait_instance::<dyn MyTrait>();
    let _ = inst.creation_time_sync();
}
