# AGENTS.md

Guidance for coding agents working on MicroRust.

## What this is

MicroRust is an alpha-stage (POC, WIP) Rust application framework. Today it
provides **dependency injection only**. The adapters listed in `README.md`
(REST, GraphQL, gRPC, databases, …) are aspirational and not implemented.

Workspace (edition 2024, resolver 3, Rust stable):

| Crate | Path | Role |
|---|---|---|
| `microrust` | `microrust/` | Runtime: registries, resolution, diagnostics. Re-exports the macros. |
| `microrust_macros` | `macros/` | Proc macros: `#[singleton]`, `#[inject_trait]`, `#[main]`, `#[test]`. |

Users depend only on `microrust`. The macros crate is an implementation detail.

## Commands

```sh
cargo check
cargo clippy                 # workspace lints already deny clippy::all
cargo nextest run            # primary test runner
cargo test --doc             # doctests (nextest can't run them)
bacon                        # default job = nextest; shift-d = doctests
```

Before you finish, run **both** `cargo nextest run` and `cargo test --doc`,
plus `cargo clippy`. A change isn't done until all three pass.

Prefer nextest over `cargo test`: it runs each test in its own process, so the
process-global registries can't leak between tests. Retries are disabled
on purpose (`.config/nextest.toml`). A flaky test is a real bug, so don't add
retries.

## Lints (enforced workspace-wide)

- `missing_docs = "deny"`: every public item, including `#[doc(hidden)]`
  plumbing that has to be `pub`, needs a doc comment.
- `rustdoc::broken_intra_doc_links = "deny"`
- `clippy::all = "deny"`

Don't silence these with `#[allow]` unless there's a stated reason in a comment.
The only existing allows are `non_snake_case` / `non_camel_case_types` on
macro-generated items.

## Architecture in one screen

Two process-global registries, both `LazyLock<Mutex<…>>`, filled by `#[ctor]`
functions that run **before `main`**:

| Kind | Key | Registered by | Resolved by |
|---|---|---|---|
| struct | `TypeId::of::<S>()` | `#[singleton(ctor)]` → `__collect_struct` | `get_instance::<S>()` |
| trait | `TypeId::of::<dyn T>()` | `#[inject_trait]` → `__collect_trait` | `get_trait_instance::<dyn T>()` |

- Singletons are lazy: `inject/instance.rs` keeps a `TypeId → Arc<OnceLock>`
  map. Every resolution returns the same `Arc`.
- A trait binding stores a monomorphized `fn() -> Arc<dyn T>` that calls
  `get_instance::<S>()` and coerces. The trait view and struct view share one
  allocation.
- Struct→dependency edges go into a small adjacency-list graph (`__STRUCT_DEPS`) for
  cycle detection (`inject/report.rs`).
- `report_registrations()` logs everything and panics on cycles or ambiguous
  trait bindings. The first `get_instance()` in a process also checks for cycles
  (otherwise a cycle deadlocks inside `OnceLock`).

## Rules that are easy to break

**Macros**
- Generated code must refer to everything through absolute paths:
  `::microrust::…`, `::std::…`, `::core::…`. Third-party crates the expansion
  needs (`async_trait`, `tokio`, `tracing`) are re-exported from
  `microrust::__private`. Never make downstream users add a dependency for
  generated code to compile. tokio's own macros need
  `crate = "::microrust::__private::tokio"` for this to hold.
- Report misuse with `crate::error::macro_error` / `syn::Error::into_compile_error`,
  never with `panic!` or `parse_quote!`. Users should get a spanned diagnostic,
  not "custom attribute panicked".
- Keep the **full** `self_ty` / trait path when generating code; use the last
  segment only in diagnostics. `module_path_test.rs` is the regression test for
  this.
- Emit generated helper items (the `ctor` registration fn) inside an anonymous
  `const _: () = { … };` block with a fixed name. Never derive a module-level
  item name from a type or trait name: those collide for same-named types in
  different modules and for generic traits. `ctor_name_test.rs` covers this.
- Injected parameters are recognized only as `Arc<Struct>` or `Arc<dyn Trait>`
  (`inject/dep.rs::classify_dep`). Extend that function rather than
  pattern-matching types elsewhere.

**Runtime**
- Never do work in a `ctor` that needs logging to be seen, or that can fail
  observably. No `tracing` subscriber exists yet, so anything emitted there is
  dropped. Record the condition and raise it from `report_registrations()`
  (see `__TRAIT_COLLISIONS` for the pattern).
- Never hold a registry `Mutex` while calling a factory or resolver.
  Resolution re-enters the registries recursively. Clone the `Arc`/`fn`
  out of the guard first (see `get_trait_instance`).
- Never take two registry locks at once.
- Don't panic inside a `Once::call_once` body (it poisons the `Once`). Record
  the verdict and panic outside (see `__assert_no_cycles`).
- Struct/trait lookup failures panic by design. Keep it that way unless the
  API is deliberately redesigned.

## Tests

- `microrust/tests/lib.rs` aggregates `tests/inject/*` into **one binary**,
  because the identity assertions depend on the shared registries.
- Fixtures that *intentionally* break the global graph (cycles, trait
  collisions) get their own `[[test]]` target in `microrust/Cargo.toml`. Do the
  same for any new fixture that would poison the shared binary.
- Use `#[traced_test]` (from `tracing-test`) so registry diagnostics show up in
  nextest output.
- Panicking behaviour is tested with `#[should_panic(expected = "…")]`. Keep
  panic messages stable, or update the `expected` strings in the same change.
- Executable examples live as doctests in `microrust/src/lib.rs` and item docs.
  If you change public behaviour, update them.

## Docs

- User docs: `docs/inject.md` (keep it in sync with behaviour and panic
  messages). `docs/diagram.png` is exported from `docs/diagram.drawio`.
- The style of in-code comments is "explain *why*". Match it: comments
  justify non-obvious design choices (ctor timing, lock ordering, boxing)
  rather than restating the code.

## Versioning

All crates share `workspace.package.version`. When bumping, also bump
`workspace.dependencies.microrust_macros.version` to the same value.

## Don't

- Don't add dependencies without a reason; the runtime deliberately avoids
  depending on a `tracing` subscriber. (tokio's `rt-multi-thread` and
  `macros` are deliberate: `#[microrust::main]` / `#[microrust::test]`
  expand through the re-export.)
- Don't commit `target/`, nested `Cargo.lock` files, or `.DS_Store`.
- Don't treat ambiguous trait bindings as overrides; there is no
  qualification syntax yet, and `ctor` order across crates is unspecified.
