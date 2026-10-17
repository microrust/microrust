//! Dependency inversion: a service depends on `Arc<dyn UserRepo>`, and the
//! container supplies whichever struct is bound to that trait.

use std::sync::Arc;

use microrust::inject::{async_trait, get_instance, get_trait_instance, inject_trait, singleton};
use microrust_examples::init_logging;

#[async_trait]
trait UserRepo: Send + Sync {
    async fn find(&self, id: u32) -> Option<String>;
}

struct InMemoryUserRepo;

#[singleton(new)]
impl InMemoryUserRepo {
    fn new() -> Self {
        Self
    }
}

#[inject_trait]
impl UserRepo for InMemoryUserRepo {
    async fn find(&self, id: u32) -> Option<String> {
        Some(format!("user-{id}"))
    }
}

struct UserService {
    repo: Arc<dyn UserRepo>,
}

#[singleton(new)]
impl UserService {
    fn new(repo: Arc<dyn UserRepo>) -> Self {
        Self { repo }
    }
}

#[microrust::main]
async fn main() {
    init_logging();

    let service = get_instance::<UserService>();
    let user = service.repo.find(7).await;
    println!("found: {user:?}");

    // The service's repo, the trait view and the struct view are one object.
    let repo = get_trait_instance::<dyn UserRepo>();
    let concrete = get_instance::<InMemoryUserRepo>();
    assert!(Arc::ptr_eq(&service.repo, &repo));
    assert_eq!(
        Arc::as_ptr(&repo) as *const (),
        Arc::as_ptr(&concrete) as *const ()
    );
    println!("trait and struct views share one instance");
}
