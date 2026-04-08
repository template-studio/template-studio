pub mod client;
pub mod config;
pub mod context;
pub mod prompts;
pub mod tools;
pub mod types;

pub use client::AiClient;
pub use config::AiConfig;
pub use context::ProjectContext;
pub use types::*;

// Re-export convenience functions
pub use tools::variable::{analyze_variables, fill_variables, extract_variables_regex};
