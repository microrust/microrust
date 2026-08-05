use std::{sync::Arc, time::Instant};

use async_trait::async_trait;
use microrust::inject::{get_instance, get_trait_instance, inject_trait, singleton};
use tracing_test::traced_test;

struct MyStruct1 {
    pub creation_time: Instant,
}

struct MyStruct2 {
    pub struct1: Arc<MyStruct1>,
    pub trait1: Arc<dyn MyTrait1>,
    pub trait2: Arc<dyn MyTrait2>,
}

struct MyStruct3 {
    pub struct2: Arc<MyStruct2>,
}

#[async_trait]
trait MyTrait1: Send + Sync {
    async fn creation_time_1(&self) -> Instant;
}

#[async_trait]
trait MyTrait2: Send + Sync {
    async fn creation_time_2(&self) -> Instant;
}

#[singleton(new)]
impl MyStruct1 {
    pub fn new() -> Self {
        Self {
            creation_time: Instant::now(),
        }
    }
}

#[inject_trait]
impl MyTrait1 for MyStruct1 {
    async fn creation_time_1(&self) -> Instant {
        self.creation_time
    }
}

#[inject_trait]
impl MyTrait2 for MyStruct1 {
    async fn creation_time_2(&self) -> Instant {
        self.creation_time
    }
}

#[singleton(new)]
impl MyStruct2 {
    pub fn new() -> Self {
        Self {
            struct1: Arc::clone(&get_instance::<MyStruct1>()),
            trait1: get_trait_instance::<dyn MyTrait1>(),
            trait2: get_trait_instance::<dyn MyTrait2>(),
        }
    }
}

#[singleton(new)]
impl MyStruct3 {
    pub fn new() -> Self {
        Self {
            struct2: Arc::clone(&get_instance::<MyStruct2>()),
        }
    }
}

#[test]
#[traced_test]
fn same_singleton_instance_sync() {
    let inst_struct1 = get_instance::<MyStruct1>();
    let inst_struct3 = get_instance::<MyStruct3>();
    let inst_struct2 = get_instance::<MyStruct2>();
    assert_eq!(
        inst_struct1.creation_time,
        inst_struct2.struct1.creation_time,
    );
    assert_eq!(
        inst_struct1.creation_time,
        inst_struct3.struct2.struct1.creation_time,
    );
}

#[microrust::test]
#[traced_test]
async fn same_singleton_instance_async() {
    let inst_trait1 = get_trait_instance::<dyn MyTrait1>();
    let inst_trait2 = get_trait_instance::<dyn MyTrait2>();
    assert_eq!(
        inst_trait1.creation_time_1().await,
        inst_trait2.creation_time_2().await
    );

    let inst_struct1 = get_instance::<MyStruct1>();
    assert_eq!(
        inst_struct1.creation_time,
        inst_trait2.creation_time_2().await
    );

    let inst_struct2 = get_instance::<MyStruct2>();
    assert_eq!(
        inst_struct2.trait1.creation_time_1().await,
        inst_struct2.trait2.creation_time_2().await
    );
    assert_eq!(
        inst_struct2.struct1.creation_time,
        inst_struct2.trait2.creation_time_2().await
    );

    let inst_struct3 = get_instance::<MyStruct3>();
    assert_eq!(
        inst_struct3.struct2.struct1.creation_time,
        inst_struct2.trait2.creation_time_2().await
    );
}
