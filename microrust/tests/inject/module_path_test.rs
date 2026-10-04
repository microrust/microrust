// Regression: `#[singleton]` / `#[inject_trait]` must accept a
// module-qualified `self_ty` and trait path, not only a bare identifier.
// The impls below are registered from a nested module so that `self_ty`
// (`super::InMemoryUserRepo`), the trait (`super::UserRepo`) and a
// constructor dependency (`dyn super::UserRepo`) each carry a module prefix.

use std::sync::Arc;

use async_trait::async_trait;
use microrust::inject::{get_instance, get_trait_instance};
use tracing_test::traced_test;

struct InMemoryUserRepo {
    backend: &'static str,
}

struct AuditService {
    repo: Arc<dyn UserRepo>,
}

#[async_trait]
trait UserRepo: Send + Sync {
    fn backend(&self) -> &'static str;
}

mod qualified {
    use std::sync::Arc;

    use microrust::inject::{inject_trait, singleton};

    #[singleton(new)]
    impl super::InMemoryUserRepo {
        pub fn new() -> Self {
            Self {
                backend: "in-memory",
            }
        }
    }

    #[singleton(new)]
    impl super::AuditService {
        pub fn new(repo: Arc<dyn super::UserRepo>) -> Self {
            Self { repo }
        }
    }

    #[inject_trait]
    impl super::UserRepo for super::InMemoryUserRepo {
        fn backend(&self) -> &'static str {
            self.backend
        }
    }
}

#[test]
#[traced_test]
fn singleton_registered_via_qualified_path_resolves() {
    let repo = get_instance::<InMemoryUserRepo>();
    assert_eq!(repo.backend, "in-memory");
}

#[test]
#[traced_test]
fn trait_registered_via_qualified_path_resolves() {
    let repo = get_trait_instance::<dyn UserRepo>();
    assert_eq!(repo.backend(), "in-memory");
}

#[test]
#[traced_test]
fn qualified_trait_dependency_injects_into_singleton_constructor() {
    let audit = get_instance::<AuditService>();
    assert_eq!(audit.repo.backend(), "in-memory");
}
