//! Shared setup for the MicroRust examples in `src/bin/`.
//!
//! Run one with `cargo run -p microrust_examples --bin <name>`. Debug builds
//! log the container's activity at `DEBUG`; `--release` runs silently.

/// Installs a `DEBUG` subscriber in debug builds; release builds stay silent.
///
/// Call it first in `main`. Registration happens in `ctor` functions before
/// `main`, so its logs are never seen; resolution, which happens after this
/// call, is.
pub fn init_logging() {
    #[cfg(debug_assertions)]
    tracing_subscriber::fmt()
        .with_max_level(tracing_subscriber::filter::LevelFilter::DEBUG)
        .init();
}
