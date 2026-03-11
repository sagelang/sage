//! Runtime errors for the Sage interpreter.

use miette::{Diagnostic, SourceSpan};
use sage_types::Span;
use thiserror::Error;

/// Convert a Sage span to a miette `SourceSpan`.
fn to_source_span(span: &Span) -> SourceSpan {
    (span.start, span.end - span.start).into()
}

/// A runtime error during interpretation.
#[derive(Debug, Clone, Error, Diagnostic)]
pub enum RuntimeError {
    #[error("undefined variable `{name}`")]
    #[diagnostic(code(sage::runtime::undefined_variable))]
    UndefinedVariable {
        name: String,
        #[label("not defined")]
        span: SourceSpan,
    },

    #[error("undefined belief `{name}`")]
    #[diagnostic(code(sage::runtime::undefined_belief))]
    UndefinedBelief {
        name: String,
        #[label("agent has no belief with this name")]
        span: SourceSpan,
    },

    #[error("type error: expected {expected}, got {found}")]
    #[diagnostic(code(sage::runtime::type_error))]
    TypeError {
        expected: String,
        found: String,
        #[label("wrong type")]
        span: SourceSpan,
    },

    #[error("division by zero")]
    #[diagnostic(code(sage::runtime::division_by_zero))]
    DivisionByZero {
        #[label("division here")]
        span: SourceSpan,
    },

    #[error("agent `{name}` not found")]
    #[diagnostic(code(sage::runtime::agent_not_found))]
    AgentNotFound {
        name: String,
        #[label("no agent with this name")]
        span: SourceSpan,
    },

    #[error("function `{name}` not found")]
    #[diagnostic(code(sage::runtime::function_not_found))]
    FunctionNotFound {
        name: String,
        #[label("no function with this name")]
        span: SourceSpan,
    },

    #[error("agent already awaited")]
    #[diagnostic(code(sage::runtime::already_awaited))]
    AlreadyAwaited {
        #[label("this agent was already awaited")]
        span: SourceSpan,
    },

    #[error("agent panicked without emitting a value")]
    #[diagnostic(code(sage::runtime::agent_panicked))]
    AgentPanicked {
        #[label("awaiting this agent")]
        span: SourceSpan,
    },

    #[error("send failed: agent has stopped")]
    #[diagnostic(code(sage::runtime::send_failed))]
    SendFailed {
        #[label("sending to this agent")]
        span: SourceSpan,
    },

    #[error("LLM inference failed: {message}")]
    #[diagnostic(code(sage::runtime::llm_error))]
    LlmError {
        message: String,
        #[label("infer call here")]
        span: SourceSpan,
    },

    #[error("return from top level")]
    #[diagnostic(code(sage::runtime::return_from_top_level))]
    ReturnFromTopLevel {
        #[label("return statement outside function")]
        span: SourceSpan,
    },

    #[error("index out of bounds: {index} >= {len}")]
    #[diagnostic(code(sage::runtime::index_out_of_bounds))]
    IndexOutOfBounds {
        index: usize,
        len: usize,
        #[label("index access here")]
        span: SourceSpan,
    },

    #[error("emit called outside agent")]
    #[diagnostic(code(sage::runtime::emit_outside_agent))]
    EmitOutsideAgent {
        #[label("emit call here")]
        span: SourceSpan,
    },

    #[error("internal error: {message}")]
    #[diagnostic(code(sage::runtime::internal_error))]
    InternalError {
        message: String,
        #[label("here")]
        span: SourceSpan,
    },
}

impl RuntimeError {
    /// Create an undefined variable error.
    #[must_use]
    pub fn undefined_variable(name: impl Into<String>, span: &Span) -> Self {
        Self::UndefinedVariable {
            name: name.into(),
            span: to_source_span(span),
        }
    }

    /// Create an undefined belief error.
    #[must_use]
    pub fn undefined_belief(name: impl Into<String>, span: &Span) -> Self {
        Self::UndefinedBelief {
            name: name.into(),
            span: to_source_span(span),
        }
    }

    /// Create a type error.
    #[must_use]
    pub fn type_error(expected: impl Into<String>, found: impl Into<String>, span: &Span) -> Self {
        Self::TypeError {
            expected: expected.into(),
            found: found.into(),
            span: to_source_span(span),
        }
    }

    /// Create a division by zero error.
    #[must_use]
    pub fn division_by_zero(span: &Span) -> Self {
        Self::DivisionByZero {
            span: to_source_span(span),
        }
    }

    /// Create an agent not found error.
    #[must_use]
    pub fn agent_not_found(name: impl Into<String>, span: &Span) -> Self {
        Self::AgentNotFound {
            name: name.into(),
            span: to_source_span(span),
        }
    }

    /// Create a function not found error.
    #[must_use]
    pub fn function_not_found(name: impl Into<String>, span: &Span) -> Self {
        Self::FunctionNotFound {
            name: name.into(),
            span: to_source_span(span),
        }
    }

    /// Create an already awaited error.
    #[must_use]
    pub fn already_awaited(span: &Span) -> Self {
        Self::AlreadyAwaited {
            span: to_source_span(span),
        }
    }

    /// Create an agent panicked error.
    #[must_use]
    pub fn agent_panicked(span: &Span) -> Self {
        Self::AgentPanicked {
            span: to_source_span(span),
        }
    }

    /// Create a send failed error.
    #[must_use]
    pub fn send_failed(span: &Span) -> Self {
        Self::SendFailed {
            span: to_source_span(span),
        }
    }

    /// Create an LLM error.
    #[must_use]
    pub fn llm_error(message: impl Into<String>, span: &Span) -> Self {
        Self::LlmError {
            message: message.into(),
            span: to_source_span(span),
        }
    }

    /// Create a return from top level error.
    #[must_use]
    pub fn return_from_top_level(span: &Span) -> Self {
        Self::ReturnFromTopLevel {
            span: to_source_span(span),
        }
    }

    /// Create an emit outside agent error.
    #[must_use]
    pub fn emit_outside_agent(span: &Span) -> Self {
        Self::EmitOutsideAgent {
            span: to_source_span(span),
        }
    }

    /// Create an internal error.
    #[must_use]
    pub fn internal(message: impl Into<String>, span: &Span) -> Self {
        Self::InternalError {
            message: message.into(),
            span: to_source_span(span),
        }
    }
}

/// Result type for runtime operations.
pub type RuntimeResult<T> = Result<T, RuntimeError>;
