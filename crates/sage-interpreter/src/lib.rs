//! Tree-walking interpreter and runtime for the Sage language.
//!
//! This crate provides the runtime execution environment for Sage programs,
//! including:
//! - Expression and statement evaluation
//! - Agent spawning and message passing
//! - LLM backend for `infer` expressions
//! - Built-in functions (print, len, push, etc.)
//!
//! # Example
//!
//! ```
//! use sage_lexer::lex;
//! use sage_parser::parse;
//! use sage_interpreter::{Runtime, RuntimeConfig};
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let source = r#"
//!     agent Main {
//!         on start {
//!             emit(42)
//!         }
//!     }
//!     run Main
//! "#;
//!
//! let lex_result = lex(source)?;
//! let source_arc: Arc<str> = Arc::from(source);
//! let (program, errors) = parse(lex_result.tokens(), source_arc);
//!
//! if let Some(program) = program {
//!     let runtime = Runtime::mock(); // Use mock() for testing without LLM
//!     let result = runtime.run(program).await?;
//!     println!("Result: {result}");
//! }
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]

mod builtins;
mod env;
mod error;
mod eval;
mod llm;
mod runtime;
mod value;

pub use env::Environment;
pub use error::{RuntimeError, RuntimeResult};
pub use eval::{eval_block, eval_expr, eval_stmt, ControlFlow, EvalContext};
pub use llm::{LlmClient, LlmConfig};
pub use runtime::{run, run_mock, Runtime, RuntimeConfig};
pub use value::{AgentHandle, AwaitError, SendError, Value};
