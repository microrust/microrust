//! Dependency-cycle fixture.
//!
//! Lives in its own `[[test]]` target because the two singletons below are
//! wired to depend on each other. Their `ctor` functions only *record* the
//! cycle, so this binary loads and runs normally; the failure is raised on
//! demand by `report_registrations()` and by the first `get_instance`, both of
//! which are assertable here.

use std::sync::Arc;

use microrust::inject::{dependency_cycles, get_instance, report_registrations, singleton};
use tracing_test::traced_test;

#[allow(dead_code)]
struct CycleA {
    b: Arc<CycleB>,
}

#[allow(dead_code)]
struct CycleB {
    a: Arc<CycleA>,
}

#[singleton(new)]
impl CycleA {
    fn new(b: Arc<CycleB>) -> Self {
        Self { b }
    }
}

#[singleton(new)]
impl CycleB {
    fn new(a: Arc<CycleA>) -> Self {
        Self { a }
    }
}

#[test]
#[traced_test]
fn mutual_dependency_is_detected() {
    let cycles = dependency_cycles();
    assert_eq!(cycles.len(), 1, "expected exactly one cycle, got {cycles:?}");

    let cycle = &cycles[0];
    assert_eq!(cycle.first(), cycle.last(), "cycle path must close: {cycle:?}");
    assert!(
        cycle.contains(&"cycle_test::CycleA") && cycle.contains(&"cycle_test::CycleB"),
        "cycle must name both participants: {cycle:?}"
    );
}

#[test]
#[traced_test]
#[should_panic(expected = "dependency cycle")]
fn report_registrations_panics_on_cycle() {
    report_registrations();
}

#[test]
#[traced_test]
#[should_panic(expected = "dependency cycle")]
fn first_resolution_panics_on_cycle() {
    get_instance::<CycleA>();
}