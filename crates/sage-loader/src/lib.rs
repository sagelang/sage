//! Module loader for Sage projects.
//!
//! This crate handles:
//! - Loading single-file programs
//! - Loading multi-file projects with `sage.toml`
//! - Module tree construction from `mod` declarations
//! - Cycle detection in imports

#![forbid(unsafe_code)]

mod error;
mod manifest;
mod tree;

pub use error::LoadError;
pub use manifest::ProjectManifest;
pub use tree::{load_project, load_single_file, ModulePath, ModuleTree, ParsedModule};
