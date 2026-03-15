//! Standard library functions for Sage.
//!
//! This module provides runtime helper functions for the Sage standard library.
//! Most stdlib functions are inlined during codegen, but some require runtime
//! helpers for correct Unicode handling or complex logic.

mod parsing;
mod string;

pub use parsing::*;
pub use string::*;
