//! Constructor injection between plain structs, no traits involved.
//!
//! `main` is synchronous on purpose: resolution needs no async runtime.

use std::sync::Arc;

use microrust::inject::{get_instance, singleton};
use microrust_examples::init_logging;

struct Config {
    dsn: String,
}

#[singleton(new)]
impl Config {
    fn new() -> Self {
        Self {
            dsn: "postgres://localhost/app".to_string(),
        }
    }
}

struct Database {
    config: Arc<Config>,
}

#[singleton(new)]
impl Database {
    fn new(config: Arc<Config>) -> Self {
        Self { config }
    }
}

struct UserService {
    db: Arc<Database>,
    config: Arc<Config>,
}

#[singleton(new)]
impl UserService {
    fn new(db: Arc<Database>, config: Arc<Config>) -> Self {
        Self { db, config }
    }
}

fn main() {
    init_logging();

    let service = get_instance::<UserService>();
    println!("connected to {}", service.db.config.dsn);

    // `Config` is reached twice in the graph but built once.
    assert!(Arc::ptr_eq(&service.config, &service.db.config));
    assert!(Arc::ptr_eq(&service.config, &get_instance::<Config>()));
    println!("Config is a single shared instance");
}
