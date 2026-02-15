# MicroRust Dependency Injection

Work in progress...

## Features

* struct
* (async) trait
* Mutex
* RwLock

## Example

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

#[async_std::main]
async fn main() {
    let inst = get_instance::<MyStruct>();
    //    let inst: Arc<MyStruct> = get_instance();
    inst.print().await;
}
```

[microrust_inject_example](https://github.com/microrust/microrust_inject_example)
