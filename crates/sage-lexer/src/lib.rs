//! Lexer for the Sage language.
//!
//! This crate provides tokenization for Sage source code using the `logos` crate.
//!
//! # Example
//!
//! ```
//! use sage_lexer::{lex, Token};
//!
//! let result = lex("let x = 42").unwrap();
//! for spanned in result.tokens() {
//!     println!("{:?} at {}..{}", spanned.token, spanned.start, spanned.end);
//! }
//! ```

#![forbid(unsafe_code)]

mod lexer;
mod token;

pub use lexer::{lex, lex_partial, LexError, LexErrorLocation, LexResult, Spanned};
pub use token::Token;

// Re-export logos for downstream use
pub use logos::Logos;
