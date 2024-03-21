//! Pacesetter is a framework built upon [axum].
//! TODO: explain what pacesetter does and why it's needed (maybe list of topics it handles)
//! TODO: example
//!
//! [axum]: https://github.com/tokio-rs/axum

#[doc(hidden)]
#[cfg(feature = "cli")]
pub mod cli;

pub mod util;

pub use util::get_env;
pub use util::init_tracing;
pub use util::Environment;
