//! Startup diagnostics: log what was registered and fail fast on wiring
//! errors, before serving anything.
//!
//! Run a debug build to see the report; it is logged at `DEBUG`.

use std::sync::Arc;

use microrust::inject::{
    async_trait, dependency_cycles, get_instance, inject_trait, report_registrations, singleton,
};
use microrust_examples::init_logging;

#[async_trait]
trait Mailer: Send + Sync {
    async fn send(&self, to: &str);
}

struct SmtpMailer;

#[singleton(new)]
impl SmtpMailer {
    fn new() -> Self {
        Self
    }
}

#[inject_trait]
impl Mailer for SmtpMailer {
    async fn send(&self, to: &str) {
        println!("mail sent to {to}");
    }
}

struct SignupService {
    mailer: Arc<dyn Mailer>,
}

#[singleton(new)]
impl SignupService {
    fn new(mailer: Arc<dyn Mailer>) -> Self {
        Self { mailer }
    }
}

#[microrust::main]
async fn main() {
    // The subscriber must exist before the report, or the report is dropped.
    init_logging();

    // Panics with the offending types named if the graph has a cycle or a
    // trait is bound twice; this graph has neither.
    report_registrations();

    // The same cycle information, as data instead of a panic.
    let cycles = dependency_cycles();
    println!("dependency cycles: {}", cycles.len());

    get_instance::<SignupService>()
        .mailer
        .send("new-user@example.com")
        .await;
}
