//! RFC-0011: Tool implementations for Sage agents.
//!
//! This module provides the built-in tools that agents can use via
//! `use ToolName` declarations and `ToolName.method()` calls.

mod http;

pub use http::{HttpClient, HttpResponse};
