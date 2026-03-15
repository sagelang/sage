//! Standard library functions for Sage.
//!
//! This module provides runtime helper functions for the Sage standard library.
//! Most stdlib functions are inlined during codegen, but some require runtime
//! helpers for correct Unicode handling or complex logic.

mod io;
mod json;
mod parsing;
mod string;
mod time;

pub use io::*;
pub use json::*;
pub use parsing::*;
pub use string::*;
pub use time::*;
