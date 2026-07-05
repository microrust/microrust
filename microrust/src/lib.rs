pub mod inject;

// re-export
pub use microrust_macros::{main, test};

#[doc(hidden)]
pub use ctor::__support::ctor_parse;
