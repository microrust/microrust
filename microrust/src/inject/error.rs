use thiserror::Error;

/// Errors returned by the injectable-function registry.
#[derive(Debug, Error)]
pub enum InjectError {
    #[error("no callable is registered under `{key}`")]
    FunctionNotRegistered { key: String },

    #[error(
        "callable registered under `{key}` has signature `{found}`, \
         but `{expected}` was requested"
    )]
    SignatureMismatch {
        key: String,
        expected: &'static str,
        found: &'static str,
    },
}

/// Convenience alias for fallible DI lookups.
pub type InjectResult<T> = Result<T, InjectError>;
