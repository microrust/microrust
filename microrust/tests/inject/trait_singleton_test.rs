use std::time::Instant;

use async_trait::async_trait;
use microrust::inject::{get_trait_instance, inject_trait, singleton};
use tracing_test::traced_test;

struct MyStruct {
    pub creation_time: Instant,
}

#[async_trait]
trait MyTrait: Send + Sync {
    async fn creation_time(&self) -> Instant;
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
    async fn creation_time(&self) -> Instant {
        self.creation_time
    }
}

#[microrust::test]
#[traced_test]
async fn same_trait_singleton_instance() {
    let inst = get_trait_instance::<dyn MyTrait>();
    let inst2 = get_trait_instance::<dyn MyTrait>();
    assert_eq!(inst.creation_time().await, inst2.creation_time().await);
}
