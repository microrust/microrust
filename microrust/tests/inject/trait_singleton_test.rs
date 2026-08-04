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
