use thiserror::Error;

/// Errors returned by the injectable-function registry.
///
/// Only the function registry reports errors. Struct and trait lookups panic on
/// a missing or ambiguous registration instead, because there is no meaningful
/// value to hand back: the caller asked for a dependency the application was
/// supposed to have wired, and continuing would hand back the wrong object.
#[derive(Debug, Error)]
pub enum InjectError {
    /// Nothing is registered under the requested key.
    ///
    /// Usually means the key was written by hand rather than produced by
    /// [`get_function!`](crate::inject::get_function), which derives it from a
    /// function name and therefore cannot drift.
    #[error("no callable is registered under `{key}`")]
    FunctionNotRegistered {
        /// The registry key that was requested.
        key: String,
    },

    /// The registered callable does not have the requested signature.
    ///
    /// Reaching this at runtime means `T` was supplied by hand. The
    /// `set_fake_*!` macros pin `T` to the alias emitted alongside the
    /// `#[inject_fn]` function, so a mismatched mock is a compile error.
    #[error(
        "callable registered under `{key}` has signature `{found}`, \
         but `{expected}` was requested"
    )]
    SignatureMismatch {
        /// The registry key that was requested.
        key: String,
        /// Signature the caller asked for.
        expected: &'static str,
        /// Signature actually registered under `key`.
        found: &'static str,
    },
}

/// Convenience alias for fallible DI lookups.
pub type InjectResult<T> = Result<T, InjectError>;