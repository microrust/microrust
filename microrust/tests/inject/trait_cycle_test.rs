//! Dependency-cycle fixture that closes through a trait binding.
//!
//! Own `[[test]]` target for the same reason as `cycle_test`: the graph is
//! deliberately cyclic. The difference is that no struct names the other
//! directly. `TraitCycleA` asks for `Arc<dyn TraitCycleService>`, and the struct
//! bound to that trait asks for `TraitCycleA`. The cycle is only visible if the
//! graph knows which struct a trait resolves to; without that edge, the first
//! `get_instance` re-enters `TraitCycleA`'s `OnceLock` and deadlocks.

use std::sync::Arc;

use microrust::inject::async_trait;
use microrust::inject::{
    dependency_cycles, get_instance, inject_trait, report_registrations, singleton,
};
use tracing_test::traced_test;

#[async_trait]
trait TraitCycleService: Send + Sync {}

#[allow(dead_code)]
struct TraitCycleA {
    service: Arc<dyn TraitCycleService>,
}

#[allow(dead_code)]
struct TraitCycleB {
    a: Arc<TraitCycleA>,
}

#[singleton(new)]
impl TraitCycleA {
    fn new(service: Arc<dyn TraitCycleService>) -> Self {
        Self { service }
    }
}

#[singleton(new)]
impl TraitCycleB {
    fn new(a: Arc<TraitCycleA>) -> Self {
        Self { a }
    }
}

#[inject_trait]
impl TraitCycleService for TraitCycleB {}

#[test]
#[traced_test]
fn cycle_through_trait_is_detected() {
    let cycles = dependency_cycles();
    assert_eq!(cycles.len(), 1, "expected exactly one cycle, got {cycles:?}");

    let cycle = &cycles[0];
    assert_eq!(cycle.first(), cycle.last(), "cycle path must close: {cycle:?}");
    for participant in [
        "trait_cycle_test::TraitCycleA",
        "dyn trait_cycle_test::TraitCycleService",
        "trait_cycle_test::TraitCycleB",
    ] {
        assert!(
            cycle.contains(&participant),
            "cycle must name {participant}: {cycle:?}"
        );
    }
}

#[test]
#[traced_test]
#[should_panic(expected = "found 1 dependency cycle(s) in the injection graph")]
fn report_registrations_panics_on_trait_cycle() {
    report_registrations();
}

#[test]
#[traced_test]
#[should_panic(expected = "found 1 dependency cycle(s) in the injection graph")]
fn first_resolution_panics_on_trait_cycle() {
    get_instance::<TraitCycleA>();
}
