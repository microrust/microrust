//! Dependency injection for applications that want to depend on interfaces
//! rather than on wiring.
//!
//! A use case declares what it needs; the container decides what it gets.
//! Nothing at the call site names a concrete type, so the same use case can be
//! exercised against a real repository or a test double without being rewritten.
//!
//! # Two ways to inject
//!
//! **By trait.** Implement an interface once and depend on `Arc<dyn Trait>`.
//! This is dependency inversion: the consumer names the abstraction, the
//! implementation is chosen at the edge.
//!
//! ```
//! use microrust::inject::{async_trait, get_trait_instance, inject_trait, singleton};
//!
//! struct InMemoryUserRepo;
//!
//! #[singleton(new)]
//! impl InMemoryUserRepo {
//!     fn new() -> Self {
//!         Self
//!     }
//! }
//!
//! #[async_trait]
//! trait UserRepo: Send + Sync {
//!     fn find(&self, id: u32) -> Option<String>;
//! }
//!
//! #[inject_trait]
//! impl UserRepo for InMemoryUserRepo {
//!     fn find(&self, id: u32) -> Option<String> {
//!         Some(format!("user-{id}"))
//!     }
//! }
//!
//! let repo = get_trait_instance::<dyn UserRepo>();
//! assert_eq!(repo.find(7).as_deref(), Some("user-7"));
//! ```
//!
//! **By struct.** A singleton's constructor parameters are injected directly.
//! No traits needed.
//!
//! ```
//! use std::sync::Arc;
//! use microrust::inject::{get_instance, singleton};
//!
//! struct Config {
//!     dsn: String,
//! }
//!
//! struct UserService {
//!     dsn: String,
//! }
//!
//! #[singleton(new)]
//! impl Config {
//!     fn new() -> Self {
//!         Self { dsn: "postgres://localhost".to_string() }
//!     }
//! }
//!
//! #[singleton(new)]
//! impl UserService {
//!     fn new(config: Arc<Config>) -> Self {
//!         Self { dsn: config.dsn.clone() }
//!     }
//! }
//!
//! let service = get_instance::<UserService>();
//! assert_eq!(service.dsn, "postgres://localhost");
//! ```
//!
//! In both routes a type has exactly one instance per process, resolved lazily
//! on first request and shared thereafter.
//!
//! # Startup
//!
//! Registries are populated by `#[ctor]` functions that run before `main`, when
//! no `tracing` subscriber exists yet, so registration is silent.
//!
//! [`main`] and [`test`] check the wiring before running your code: a
//! dependency cycle or an ambiguous trait binding panics with the offending
//! types named. To also log what was registered, call
//! [`inject::report_registrations`] once the subscriber is installed.
//!
//! ```ignore
//! fn main() {
//!     tracing_subscriber::fmt::init();
//!     microrust::inject::report_registrations();
//! }
//! ```
//!
//! Not shown as compiled above because the subscriber implementation is the
//! application's choice: this crate deliberately does not depend on one.
//!
//! # Runtime
//!
//! [`main`] and [`test`] run async code on tokio's multi-threaded runtime, the
//! only one MicroRust supports, without the application depending on tokio.
//! Their arguments tune that runtime (`worker_threads = 4`); asking for another
//! flavor is a compile error:
//!
//! ```compile_fail
//! #[microrust::main(flavor = "current_thread")]
//! async fn main() {}
//! ```
//!
//! # Scope
//!
//! This crate currently provides dependency injection and nothing else. Adapters
//! for HTTP, GraphQL, gRPC or storage are not implemented.

pub mod inject;

#[doc(hidden)]
pub mod __private;

// re-export
pub use microrust_macros::{main, test};

#[doc(hidden)]
pub use ctor::__support::ctor_parse;