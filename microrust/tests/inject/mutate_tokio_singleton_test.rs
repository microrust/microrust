use async_trait::async_trait;
use microrust::inject::{get_instance_mutex, singleton_tokio_mutex};
use tracing_test::traced_test;

struct MyStructMutex {
    pub counter: i32,
}

struct MyStructRwLock {
    counter: i32,
}

#[async_trait]
trait MyTraitMutex: Send + Sync {
    async fn print_counter(&self);
    async fn increment_counter(&mut self);
}

#[async_trait]
trait MyTraitRwLock: Send + Sync {
    async fn print_counter(&self);
    async fn increment_counter(&mut self);
}

#[singleton_tokio_mutex(new)]
impl MyStructMutex {
    pub fn new() -> Self {
        Self { counter: 7 }
    }

    fn increment_counter(&mut self) {
        self.counter += 1;
    }

    async fn counter(&self) -> i32 {
        self.counter
    }
}

#[test]
#[traced_test]
fn same_struct_singleton_instance_sync() {
    let inst = get_instance_mutex::<MyStructMutex>();
    let inst2 = get_instance_mutex::<MyStructMutex>();
    assert_eq!(inst.lock().counter, inst2.lock().counter);
}

#[microrust::test]
#[traced_test]
async fn same_struct_singleton_instance_async() {
    let inst = get_instance::<MyStruct>();
    let inst2 = get_instance::<MyStruct>();
    assert_eq!(inst.creation_time().await, inst2.creation_time().await);
}
