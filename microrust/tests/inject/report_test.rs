use microrust::inject::{dependency_cycles, report_registrations};
use tracing_test::traced_test;

/// Runs in the shared test binary, so the graph under test is the one built by
/// every other module here (`complex_singleton_test` alone contributes six
/// nodes and their edges). Traversal therefore runs over a real graph rather
/// than an empty one.
#[test]
#[traced_test]
fn acyclic_injection_graph_reports_no_cycles() {
    assert!(dependency_cycles().is_empty());
}

#[test]
#[traced_test]
fn report_registrations_succeeds_for_acyclic_graph() {
    report_registrations();
}
/// The startup checks must pass over the full, valid graph of the shared
/// binary, or every `#[microrust::test]` in a correctly wired crate would fail.
#[microrust::test]
async fn test_macro_startup_checks_pass_for_valid_wiring() {
    assert!(dependency_cycles().is_empty());
}
