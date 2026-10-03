pub mod inject;

#[doc(hidden)]
pub mod __private;

// re-export
pub use microrust_macros::{main, test};

#[doc(hidden)]
pub use ctor::__support::ctor_parse;
