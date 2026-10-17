//! Ambiguous-trait fixture.
//!
//! Own `[[test]]` target because `report_registrations()` is expected to abort
//! here. The two `ctor` functions only record the clash, so this binary loads
//! and runs normally; the failure is raised on demand.
//!
//! Deliberately cycle-free, so the panic can only come from the ambiguous
//! binding and not from the dependency check.

use microrust::inject::async_trait;
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
/// The report must be repeatable. Raising the collision panic while still
/// holding the collision lock poisoned it, so a second call (a retry, or any
/// call after `catch_unwind`) died with `PoisonError` instead of naming the
/// clash.
#[test]
#[traced_test]
fn report_registrations_panics_with_same_message_when_repeated() {
    let message = |result: std::thread::Result<()>| {
        let payload = result.expect_err("report_registrations should panic");
        payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default()
    };

    let first = message(std::panic::catch_unwind(report_registrations));
    let second = message(std::panic::catch_unwind(report_registrations));

    assert!(first.contains("ambiguous trait binding"), "first: {first}");
    assert_eq!(first, second);
}

/// `#[microrust::test]` runs the startup checks before the body, so the
/// ambiguous binding fails the test without the body ever running. The body's
/// own panic doesn't match `expected`, so reaching it would fail the test.
#[microrust::test]
#[should_panic(expected = "ambiguous trait binding")]
async fn test_macro_rejects_ambiguous_binding_before_body() {
    panic!("body ran despite broken wiring");
}
