//! Runtime library for compiled Sage programs.
//!
//! This crate provides the types and functions that generated Rust code
//! depends on. It handles:
//!
//! - Agent spawning and lifecycle
//! - Message passing between agents
//! - LLM inference calls
//! - RFC-0011: Tool execution (Http, Fs, etc.)
//! - Error handling

#![forbid(unsafe_code)]

mod agent;
mod error;
mod llm;
pub mod stdlib;
pub mod tools;

pub use agent::{spawn, AgentContext, AgentHandle};
pub use error::{ErrorKind, SageError, SageResult};
pub use llm::LlmClient;
pub use tools::{HttpClient, HttpResponse};

/// Prelude for generated code.
pub mod prelude {
    pub use crate::agent::{spawn, AgentContext, AgentHandle};
    pub use crate::error::{ErrorKind, SageError, SageResult};
    pub use crate::llm::LlmClient;
    pub use crate::tools::{HttpClient, HttpResponse};
}
