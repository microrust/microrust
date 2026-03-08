#[doc(hidden)]
pub mod inject;

// re-export
#[doc(hidden)]
pub use ctor::__support::ctor_parse;
pub use microrust_macros::{main, test};
