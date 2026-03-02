use std::time::Instant;

use microrust::inject::{get_instance, singleton};
use tracing_test::traced_test;

struct MyStruct {
    pub creation_time: Instant,
}

#[singleton(new)]
impl MyStruct {
    pub fn new() -> Self {
        Self {
            creation_time: Instant::now(),
        }
    }

    async fn creation_time(&self) -> Instant {
        self.creation_time
    }
}

#[tokio::test]
#[traced_test]
async fn same_struct_singleton_instance() {
    let inst = get_instance::<MyStruct>();
    let inst2 = get_instance::<MyStruct>();
    assert_eq!(inst.creation_time, inst2.creation_time);
    assert_eq!(inst.creation_time().await, inst2.creation_time().await);
}
