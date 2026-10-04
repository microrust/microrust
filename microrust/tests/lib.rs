//! Integration tests, aggregated into one binary so they share a process.
//!
//! Sharing matters here: every singleton test asserts on identity, so the
//! registries under test are the ones built by all the modules together.
//! Fixtures that must fail loudly — a dependency cycle, an ambiguous trait
//! binding — get their own `[[test]]` targets in `Cargo.toml` instead, since
//! they would take this binary down with them.

mod inject;