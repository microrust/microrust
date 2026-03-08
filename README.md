# MicroRust
MicroRust Application Framework

A modern rust application framework, asynchronous and with batteries included.

Inspired by frameworks like Spring Boot, Micronaut and Quarkus.

## Features
* Dependency Injection

## Examples

### Dependency Injection
```rust
use microrust::inject::{get_instance, singleton};

struct MyStruct {}

#[singleton(new)]
impl MyStruct {
    pub fn new() -> Self {
        println!("MyStruct::new() called");
        Self {}
    }

    async fn print(&self) {
        println!("Hello, MyStruct");
    }
}

#[tokio::main]
async fn main() {
    let inst = get_instance::<MyStruct>();
    // let inst: Arc<MyStruct> = get_instance();
    inst.print().await;
}
```
