//! `#[microrust::main]` / `#[microrust::test]` must work without the calling
//! crate depending on tokio. This crate has no tokio dev-dependency, so the
//! file compiling at all is the regression test; the bodies check that the
//! runtime is always multi-threaded and that other arguments still reach
//! `#[tokio::test]`.

use microrust::__private::tokio::runtime::{Handle, RuntimeFlavor};

// Without an argument on purpose: `#[tokio::test]` alone would give a
// single-threaded runtime, and `#[microrust::test]` must not.
#[microrust::test]
async fn test_macro_defaults_to_multi_thread_runtime() {
    assert_eq!(Handle::current().runtime_flavor(), RuntimeFlavor::MultiThread);
}

// The redundant explicit flavor is accepted rather than rejected as a
// duplicate of the one the macro always emits.
#[microrust::test(flavor = "multi_thread")]
async fn test_macro_accepts_explicit_multi_thread_flavor() {
    assert_eq!(Handle::current().runtime_flavor(), RuntimeFlavor::MultiThread);
}

#[microrust::test(worker_threads = 2)]
async fn test_macro_forwards_other_arguments() {
    let handle = Handle::current();
    assert_eq!(handle.runtime_flavor(), RuntimeFlavor::MultiThread);
    assert_eq!(handle.metrics().num_workers(), 2);
}
