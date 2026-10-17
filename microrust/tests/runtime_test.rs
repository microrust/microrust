//! `#[microrust::main]` / `#[microrust::test]` must work without the calling
//! crate depending on tokio. This crate has no tokio dev-dependency, so the
//! file compiling at all is the regression test; the bodies check that the
//! runtime is real and that forwarded arguments reach `#[tokio::test]`.

#[microrust::test]
async fn test_macro_runs_on_current_thread_runtime() {
    // Spawning panics outside a runtime, so this proves one is driving the
    // body, not just that it compiled.
    let task = microrust::__private::tokio::spawn(async { 21 * 2 });
    assert_eq!(task.await.unwrap(), 42);
}

#[microrust::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_macro_forwards_flavor() {
    let handle = microrust::__private::tokio::runtime::Handle::current();
    assert_eq!(
        handle.runtime_flavor(),
        microrust::__private::tokio::runtime::RuntimeFlavor::MultiThread
    );
}
