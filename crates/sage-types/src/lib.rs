//! Shared type definitions for the Sage language.
//!
//! This crate provides the foundational types used across all Sage compiler
//! passes: source spans for error reporting, identifiers, and type expressions.

#![forbid(unsafe_code)]

mod span;
mod ty;

pub use span::{Ident, Span};
pub use ty::TypeExpr;
