use std::{
    ops::Add,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use microrust::inject::{
    get_trait_instance, inject_fake_trait, inject_trait, set_fake_trait_instance, singleton,
};
use tracing_test::traced_test;

struct MyStruct {
    pub creation_time: Instant,
}

struct FakeMyStruct {
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

#[singleton(new)]
impl FakeMyStruct {
    pub fn new() -> Self {
        Self {
            creation_time: Instant::now().add(Duration::from_hours(1)),
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

#[inject_fake_trait]
impl MyTrait for FakeMyStruct {
    fn creation_time_sync(&self) -> Instant {
        self.creation_time
    }
    async fn creation_time_async(&self) -> Instant {
        self.creation_time
    }
}

#[test]
#[traced_test]
fn not_same_trait_singleton_instance_sync() {
    let inst = get_trait_instance::<dyn MyTrait>();
    set_fake_trait_instance::<dyn MyTrait, FakeMyStruct>();
    let inst2 = get_trait_instance::<dyn MyTrait>();
    assert_ne!(inst.creation_time_sync(), inst2.creation_time_sync());
    // ToDo: reset trait instance for further tests
    // set_fake_trait_instance::<dyn MyTrait, MyStruct>();
}

#[microrust::test]
#[traced_test]
async fn not_same_trait_singleton_instance_async() {
    let inst = get_trait_instance::<dyn MyTrait>();
    set_fake_trait_instance::<dyn MyTrait, FakeMyStruct>();
    let inst2 = get_trait_instance::<dyn MyTrait>();
    assert_ne!(
        inst.creation_time_async().await,
        inst2.creation_time_async().await
    );
}
