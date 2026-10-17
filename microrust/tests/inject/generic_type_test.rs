// Regression: `#[singleton]` on a concrete instantiation of a generic type
// (`impl Wrapper<u32>`) used to emit `Wrapper<u32>::new(..)`, which parses as a
// chain of comparisons. Two instantiations of the same generic are registered
// here to also show they are distinct singletons, keyed by distinct `TypeId`s.

use std::sync::Arc;

use microrust::inject::{get_instance, singleton};
use tracing_test::traced_test;

struct Wrapper<T>(T);

#[singleton(new)]
impl Wrapper<u32> {
    fn new() -> Self {
        Self(7)
    }
}

#[singleton(new)]
impl Wrapper<&'static str> {
    fn new(number: Arc<Wrapper<u32>>) -> Self {
        Self(if number.0 == 7 { "seven" } else { "other" })
    }
}

#[test]
#[traced_test]
fn concrete_generic_singleton_resolves() {
    assert_eq!(get_instance::<Wrapper<u32>>().0, 7);
}

#[test]
#[traced_test]
fn concrete_generic_singleton_injects_another_instantiation() {
    assert_eq!(get_instance::<Wrapper<&'static str>>().0, "seven");
}

#[test]
#[traced_test]
fn concrete_generic_singleton_is_shared() {
    let first = get_instance::<Wrapper<u32>>();
    let second = get_instance::<Wrapper<u32>>();
    assert!(Arc::ptr_eq(&first, &second));
}
