pub mod agent;
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
pub use tools::render::{render_preview, render_export};
pub use tools::validate::{validate_syntax, validate_variables};
pub use tools::file::edit_file;
pub use tools::convert::convert_to_template;
pub use tools::recommend::recommend_template;
pub use tools::diff::render_diff;

// Re-export agent
pub use agent::{Agent, AgentResult, create_default_agent};
