//! Development-time tracing for debugging the runner.
//!
//! # Separation of Concerns
//!
//! - **Tracing (this module)**: Dev diagnostics via `RUST_LOG`, output to stderr.
//!   Not persisted, not part of runner product output.
//!
//! - **Iteration logging (`io/iteration_log`)**: Product artifacts in
//!   `.runner/iterations/`. Always written, unaffected by `RUST_LOG`.

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize tracing subscriber for development logging.
///
/// Reads `RUST_LOG` env var. Defaults to `warn` if unset.
/// Output: stderr, compact format.
///
/// # Example
/// ```bash
/// RUST_LOG=runner=debug cargo run -- step
/// ```
pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_writer(std::io::stderr).compact())
        .init();
}
