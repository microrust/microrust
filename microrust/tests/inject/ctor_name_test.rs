// Regression: the `ctor` functions the macros generate used to be named from
// the last path segment alone (`ctor_<Struct>`, `ctor_<Trait>_<Struct>`), so
// valid registrations below collided as "defined multiple times". Each case is
// a different way to produce the same name.

#![allow(non_camel_case_types)]

use async_trait::async_trait;
use microrust::inject::{get_instance, get_trait_instance, inject_trait, singleton};
use tracing_test::traced_test;

// Same struct name in two modules, both registered from here: both used to
// emit `ctor_Repo`.
mod first {
    pub struct Repo {
        pub id: &'static str,
    }
}

mod second {
    pub struct Repo {
        pub id: &'static str,
    }
}

#[singleton(new)]
impl first::Repo {
    fn new() -> Self {
        Self { id: "first" }
    }
}

#[singleton(new)]
impl second::Repo {
    fn new() -> Self {
        Self { id: "second" }
    }
}

// One struct implementing a generic trait twice: the two bindings are distinct
// traits to the registry, but both used to emit `ctor_Store_Db`.
#[async_trait]
trait Store<T>: Send + Sync {
    fn kind(&self) -> &'static str;
}

struct Db;

#[singleton(new)]
impl Db {
    fn new() -> Self {
        Self
    }
}

#[inject_trait]
impl Store<u32> for Db {
    fn kind(&self) -> &'static str {
        "u32"
    }
}

#[inject_trait]
impl Store<String> for Db {
    fn kind(&self) -> &'static str {
        "string"
    }
}

// Across the two macros: struct `Port_Adapter` and `impl Port for Adapter`
// both used to emit `ctor_Port_Adapter`.
#[async_trait]
trait Port: Send + Sync {
    fn name(&self) -> &'static str;
}

struct Adapter;

struct Port_Adapter;

#[singleton(new)]
impl Adapter {
    fn new() -> Self {
        Self
    }
}

#[singleton(new)]
impl Port_Adapter {
    fn new() -> Self {
        Self
    }
}

#[inject_trait]
impl Port for Adapter {
    fn name(&self) -> &'static str {
        "adapter"
    }
}

#[test]
#[traced_test]
fn same_named_structs_in_different_modules_both_resolve() {
    assert_eq!(get_instance::<first::Repo>().id, "first");
    assert_eq!(get_instance::<second::Repo>().id, "second");
}

#[test]
#[traced_test]
fn generic_trait_implemented_twice_resolves_each_binding() {
    assert_eq!(get_trait_instance::<dyn Store<u32>>().kind(), "u32");
    assert_eq!(get_trait_instance::<dyn Store<String>>().kind(), "string");
}

#[test]
#[traced_test]
fn struct_and_trait_binding_with_overlapping_names_both_register() {
    let _ = get_instance::<Port_Adapter>();
    assert_eq!(get_trait_instance::<dyn Port>().name(), "adapter");
}
