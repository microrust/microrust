use std::sync::Arc;

use async_trait::async_trait;
use microrust::inject::{
    get_function, get_instance, inject_fn, inject_trait, set_fake_async_function,
    singleton,
};
use tracing_test::traced_test;

#[derive(Debug, PartialEq, Eq)]
struct User {
    name: String,
}

#[async_trait]
trait UserRepo: Send + Sync {
    async fn find(&self, name: &str) -> Option<User>;
}

struct RealRepo;

#[singleton(new)]
impl RealRepo {
    pub fn new() -> Self {
        Self
    }
}

#[inject_trait]
impl UserRepo for RealRepo {
    async fn find(&self, name: &str) -> Option<User> {
        if name == "steve" {
            Some(User {
                name: name.to_string(),
            })
        } else {
            None
        }
    }
}

// An async injectable fn: `repo` is removed from the signature and resolved
// from the singleton registry on every call.
#[inject_fn]
async fn load_user(name: String, #[inject] repo: Arc<dyn UserRepo>) -> Option<User> {
    repo.find(&name).await
}

// A sync injectable fn taking both a struct dep and a trait dep.
struct Counter {
    total: u32,
}

#[singleton(new)]
impl Counter {
    pub fn new() -> Self {
        Self { total: 0 }
    }
}

#[inject_fn]
fn describe(
    #[inject] counter: Arc<Counter>,
    #[inject] repo: Arc<dyn UserRepo>,
) -> (u32, bool) {
    (counter.total, repo_is_real(repo))
}

fn repo_is_real(_repo: Arc<dyn UserRepo>) -> bool {
    true
}

#[microrust::test]
#[traced_test]
async fn registered_async_fn_resolves_its_own_dependencies() {
    let f = get_function!(load_user).unwrap();
    assert_eq!(f("steve".to_string()).await, Some(User { name: "steve".into() }));
    assert_eq!(f("nobody".to_string()).await, None);
}

#[test]
#[traced_test]
fn registered_fn_returns_typed_callable_and_resolves_struct_dep() {
    let f = get_function!(describe).unwrap();
    assert_eq!(f(), (0, true));
}

#[test]
#[traced_test]
fn returned_callable_is_stable_across_calls() {
    let a = get_function!(load_user).unwrap();
    let b = get_function!(load_user).unwrap();

    // Repeated lookups resolve to the same registered target, and the pointer
    // type is nameable outside the defining module.
    assert!(std::ptr::fn_addr_eq(a, b));

    let rt = ::tokio::runtime::Runtime::new().unwrap();
    assert_eq!(
        rt.block_on(a("steve".to_string())),
        Some(User { name: "steve".into() })
    );
    assert_eq!(
        rt.block_on(b("steve".to_string())),
        Some(User { name: "steve".into() })
    );
}

#[test]
#[traced_test]
fn returned_callable_can_be_stored_in_a_struct_field() {
    type LoadUser = fn(String) -> ::std::pin::Pin<
        Box<dyn ::core::future::Future<Output = Option<User>> + Send>,
    >;

    struct Handler {
        load: LoadUser,
    }

    let h = Handler {
        load: get_function!(load_user).unwrap(),
    };

    let out = ::tokio::runtime::Runtime::new()
        .unwrap()
        .block_on((h.load)("steve".to_string()));
    assert_eq!(out, Some(User { name: "steve".into() }));
}

#[test]
#[traced_test]
fn singleton_and_get_function_share_the_same_instance() {
    let via_singleton = get_instance::<Counter>();
    let f = get_function!(describe).unwrap();
    assert_eq!(f().0, via_singleton.total);
}

// --- mocking --------------------------------------------------------------

async fn mock_load_user(name: String) -> Option<User> {
    Some(User {
        name: format!("mocked:{name}"),
    })
}

#[test]
#[traced_test]
fn set_fake_async_function_swaps_the_registered_implementation() {
    let before = get_function!(load_user).unwrap();
    let fut = ::tokio::runtime::Runtime::new().unwrap().block_on(before("steve".to_string()));
    assert_eq!(fut, Some(User { name: "steve".into() }));

    set_fake_async_function!(load_user, mock_load_user).unwrap();

    let after = get_function!(load_user).unwrap();
    let fut = ::tokio::runtime::Runtime::new().unwrap().block_on(after("steve".to_string()));
    assert_eq!(fut, Some(User { name: "mocked:steve".into() }));

    // ToDo: reset function instance for further tests
    // set_fake_async_function!(load_user, load_user_impl).unwrap();
}
