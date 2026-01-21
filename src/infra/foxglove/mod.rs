// Foxglove FlatBuffer schemas
#[allow(unused_imports, dead_code, non_snake_case, clippy::all)]
pub mod time_generated;
#[allow(unused_imports, dead_code, non_snake_case, clippy::all)]
pub mod log_generated;

// Re-export for convenience
pub use time_generated::foxglove::Time;
pub use time_generated::foxglove::TimeArgs;
pub use log_generated::foxglove::*;
