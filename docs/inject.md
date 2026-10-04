# MicroRust Dependency Injection

## Layers and Dependency Inversion

You have two ways to take advantage of dependency injection:
- Dependency inversion using *trait*
- Dependency injection using *struct* or *function*

![Dependency Management](diagram.png)

## trait

## struct

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

#[microrust::main]
async fn main() {
    let inst = get_instance::<MyStruct>();
    // let inst: Arc<MyStruct> = get_instance();
    inst.print().await;
}
```

## function

`#[inject_fn]` makes a plain function injectable and registers it in a global
callable registry. Parameters marked `#[inject]` are removed from the signature
and resolved from the singleton registry on each call, so callers only pass
what they own. `get_function!` hands back the callable, typed by a hidden alias
generated alongside it.

```rust
use std::sync::Arc;

use microrust::inject::{get_function, inject_fn, inject_trait, singleton};

struct UserRepo;

#[async_trait]
trait UserService: Send + Sync {
    async fn find(&self, name: &str) -> Option<String>;
}

#[singleton(new)]
impl UserRepo {
    pub fn new() -> Self {
        Self
    }
}

#[inject_trait]
impl UserService for UserRepo {
    async fn find(&self, name: &str) -> Option<String> {
        Some(name.to_string())
    }
}

#[inject_fn]
async fn load_user(name: String, #[inject] repo: Arc<dyn UserService>) -> Option<String> {
    repo.find(&name).await
}

#[microrust::test]
async fn injected_fn_is_callable_through_get_function() {
    // `repo` is not a parameter here - the registry supplied it.
    let f = get_function!(load_user).unwrap();
    assert_eq!(f("steve".to_string()).await, Some("steve".to_string()));
}
```

`#[inject_fn]` rewrites the function above to:

```rust
async fn load_user(name: String) -> Option<String> {
    let repo: Arc<dyn UserService> = get_trait_instance::<dyn UserService>();
    repo.find(&name).await
}
```

### Mocking

Because `get_function!` returns a `fn` pointer, a test can swap the registered
implementation. `set_fake_async_function!` accepts an `async fn` and boxes it;
`set_fake_function!` accepts a value that already has the registered
signature. A mock with the wrong signature is a compile error, because both
macros pin the same generated alias.

```rust
async fn mock_load_user(name: String) -> Option<String> {
    Some(format!("mocked:{name}"))
}

#[test]
fn mock_replaces_the_registered_callable() {
    set_fake_async_function!(load_user, mock_load_user).unwrap();

    let f = get_function!(load_user).unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();
    assert_eq!(
        rt.block_on(f("steve".to_string())),
        Some("mocked:steve".to_string())
    );
}
```

> **Warning**
> Mocks are global state. A `set_fake_*` call in one test affects every other
> test in the same binary, and there is no built-in reset. Keep mocked
> functions in their own integration-test file so they get their own process.

### Constraints

- `#[inject_fn]` applies to safe, non-generic, non-variadic free functions.
  Methods are rejected, because `self` cannot appear in a `fn` pointer type.
- It must be the **outermost** attribute. On an `#[async_trait]` method,
  `#[async_trait]` would desugar first and the rewrite would see a boxed-future
  signature instead of the real parameters.
- An `async fn`'s future is boxed on every call. This is unavoidable: `impl
  Future` in return position cannot leave its defining module, so the callable
  has to be erased to a nameable `Pin<Box<dyn Future + Send>>`. Sync
  `#[inject_fn]` functions skip the boxing.
- An `#[inject]` parameter must be `Arc<Struct>` or `Arc<dyn Trait>`, and must
  bind to a plain identifier.
