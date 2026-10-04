//! Ambiguous-trait fixture.
//!
//! Own `[[test]]` target because `report_registrations()` is expected to abort
//! here. The two `ctor` functions only record the clash, so this binary loads
//! and runs normally; the failure is raised on demand.
//!
//! Deliberately cycle-free, so the panic can only come from the ambiguous
//! binding and not from the dependency check.

use async_trait::async_trait;
use microrust::inject::{get_trait_instance, inject_trait, report_registrations, singleton};
use tracing_test::traced_test;

struct AlphaRepo;

struct BetaRepo;

#[async_trait]
trait SharedRepo: Send + Sync {
    fn id(&self) -> &'static str;
}

#[singleton(new)]
impl AlphaRepo {
    fn new() -> Self {
        Self
    }
}

#[singleton(new)]
impl BetaRepo {
    fn new() -> Self {
        Self
    }
}

#[inject_trait]
impl SharedRepo for AlphaRepo {
    fn id(&self) -> &'static str {
        "alpha"
    }
}

#[inject_trait]
impl SharedRepo for BetaRepo {
    fn id(&self) -> &'static str {
        "beta"
    }
}

#[test]
#[traced_test]
#[should_panic(expected = "ambiguous trait binding")]
fn report_registrations_panics_on_ambiguous_trait() {
    report_registrations();
}

/// First registration wins, but *which* one is not a stable contract: `ctor`
/// ordering across crates is unspecified. Asserting that the winner is one of
/// the two implementations is deliberate; asserting a specific one would make
/// this test depend on link order.
#[test]
#[traced_test]
fn ambiguous_binding_resolves_to_exactly_one_impl() {
    let resolved = get_trait_instance::<dyn SharedRepo>();
    assert!(
        resolved.id() == "alpha" || resolved.id() == "beta",
        "unexpected impl resolved: {}",
        resolved.id()
    );
}