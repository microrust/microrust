# MicroRust Dependency Injection

## Layers and Dependency Inversion

You have two ways to take advantage of dependency injection:
- Dependency inversion using *trait*
- Dependency injection using *struct*

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

The constructor named in `#[singleton(...)]` must be a synchronous associated
function that returns `Self`, and every parameter must be `Arc<Struct>` or
`Arc<dyn Trait>`. Other methods on the type may be `async`, as `print` is above.
A constructor that returns another type, or is `async`, is a compile error.

## Startup diagnostics

Registries are filled from `#[ctor]` functions, which run in `.init_array`
**before `main`**. At that point no `tracing` subscriber exists: the global max
level starts at `OFF` and the global dispatch starts as `NO_SUBSCRIBER`, so any
event emitted from a `ctor` is discarded before its arguments are formatted.
Registration therefore stays silent, and the diagnostics are emitted on demand
instead.

Call this once, after installing your subscriber:

```rust
fn main() {
    tracing_subscriber::fmt::init();

    microrust::inject::report_registrations();
}
```

It logs every registered struct and trait binding, plus a
dependency-edge count, at `DEBUG`.

### Dependency cycles

The same pass walks the struct dependency graph. If two singletons depend on
each other, resolving them would re-enter a `OnceLock` that is still
initializing and **deadlock** rather than fail, so the cycle is reported up
front with the offending types named:

```text
[microrust] found dependency cycle(s) detected in the injection graph:
    cycle_test::CycleA -> cycle_test::CycleB -> cycle_test::CycleA
```

Cycles that pass through a trait binding are caught too. A dependency on
`Arc<dyn Trait>` is followed to the struct bound to that trait with
`#[inject_trait]`, and the trait shows up in the reported path:

```text
[microrust] found dependency cycle(s) detected in the injection graph:
    trait_cycle_test::TraitCycleA -> dyn trait_cycle_test::TraitCycleService
    -> trait_cycle_test::TraitCycleB -> trait_cycle_test::TraitCycleA
```

### Ambiguous trait bindings

If two structs `#[inject_trait]` the same trait, the first registration wins and
the clash is reported:

```text
[microrust] found 1 ambiguous trait binding(s):
    dyn trait_collision_test::SharedRepo is implemented by both
    trait_collision_test::BetaRepo and trait_collision_test::AlphaRepo
```

Which struct wins is **not a stable contract**: `#[ctor]` ordering across crates
is unspecified, so the resolved instance can differ between builds of the same
source. There is no qualification syntax yet, so treat every such clash as a
wiring mistake rather than an override.

`report_registrations()` panics in both cases, so call it during startup rather
than on a request path. `microrust::inject::dependency_cycles()` returns the
cycle information as data if you would rather inspect it yourself.

As a safety net for applications that never call `report_registrations()`, the
first `get_instance()` in a process performs the cycle check and fails with the
same message instead of deadlocking. Ambiguous traits have no such net: an
application that skips the report can still resolve one silently.
