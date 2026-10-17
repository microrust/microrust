//! Singletons for concrete instantiations of a generic type.
//!
//! `impl Wrapper<u32>` works; `impl<T> Wrapper<T>` is rejected at compile
//! time, since registration needs one concrete type.

use std::sync::Arc;

use microrust::inject::{get_instance, singleton};
use microrust_examples::init_logging;

struct Wrapper<T>(T);

#[singleton(new)]
impl Wrapper<u32> {
    fn new() -> Self {
        Self(42)
    }
}

// A different instantiation is a different type, and so a separate singleton
// that can even depend on the first.
#[singleton(new)]
impl Wrapper<String> {
    fn new(number: Arc<Wrapper<u32>>) -> Self {
        Self(format!("the answer is {}", number.0))
    }
}

fn main() {
    init_logging();

    println!("{}", get_instance::<Wrapper<u32>>().0);
    println!("{}", get_instance::<Wrapper<String>>().0);
}
