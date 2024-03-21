//! Pacesetter is a framework built upon [axum].
//! TODO: explain what pacesetter does and why it's needed (maybe list of topics it handles)
//! TODO: example
//!
//! [axum]: https://github.com/tokio-rs/axum

#[doc(hidden)]
#[cfg(feature = "cli")]
pub mod cli;

/// Confuguration handling structs and utilities
pub mod config;

pub mod util;

pub use config::load_config;
pub use util::get_env;
pub use util::init_tracing;
pub use util::Environment;
