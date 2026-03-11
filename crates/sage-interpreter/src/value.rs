//! Runtime values for the Sage interpreter.

use std::fmt;
use std::sync::Arc;
use tokio::sync::oneshot;

/// A runtime value in Sage.
#[derive(Debug, Clone)]
pub enum Value {
    /// 64-bit signed integer.
    Int(i64),
    /// 64-bit floating point.
    Float(f64),
    /// Boolean.
    Bool(bool),
    /// UTF-8 string.
    String(String),
    /// Unit value (void equivalent).
    Unit,
    /// Homogeneous list.
    List(Vec<Value>),
    /// Optional value.
    Option(Option<Box<Value>>),
    /// Handle to a running agent.
    Agent(AgentHandle),
}

impl Value {
    /// Check if this value is truthy (for conditionals).
    #[must_use]
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::String(s) => !s.is_empty(),
            Value::List(l) => !l.is_empty(),
            Value::Option(o) => o.is_some(),
            Value::Unit => false,
            _ => true,
        }
    }

    /// Try to get this value as an integer.
    #[must_use]
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(n) => Some(*n),
            _ => None,
        }
    }

    /// Try to get this value as a float.
    #[must_use]
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// Try to get this value as a boolean.
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Try to get this value as a string.
    #[must_use]
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get this value as a list.
    #[must_use]
    pub fn as_list(&self) -> Option<&[Value]> {
        match self {
            Value::List(l) => Some(l),
            _ => None,
        }
    }

    /// Try to get this value as an agent handle.
    #[must_use]
    pub fn as_agent(&self) -> Option<&AgentHandle> {
        match self {
            Value::Agent(h) => Some(h),
            _ => None,
        }
    }

    /// Get a type name for error messages.
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "Int",
            Value::Float(_) => "Float",
            Value::Bool(_) => "Bool",
            Value::String(_) => "String",
            Value::Unit => "Unit",
            Value::List(_) => "List",
            Value::Option(_) => "Option",
            Value::Agent(_) => "Agent",
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{n}"),
            Value::Float(n) => write!(f, "{n}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Unit => write!(f, "()"),
            Value::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
            Value::Option(Some(v)) => write!(f, "Some({v})"),
            Value::Option(None) => write!(f, "None"),
            Value::Agent(h) => write!(f, "Agent<{}>", h.agent_name),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Unit, Value::Unit) => true,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Option(a), Value::Option(b)) => a == b,
            // Agent handles are equal if they point to the same agent
            (Value::Agent(a), Value::Agent(b)) => Arc::ptr_eq(&a.inner, &b.inner),
            _ => false,
        }
    }
}

/// Handle to a running agent, allowing await and send operations.
#[derive(Debug, Clone)]
pub struct AgentHandle {
    /// The agent's type name.
    pub agent_name: String,
    /// Shared inner state.
    pub(crate) inner: Arc<AgentHandleInner>,
}

/// Inner state of an agent handle.
#[derive(Debug)]
pub(crate) struct AgentHandleInner {
    /// Channel to send messages to the agent.
    pub(crate) message_tx: tokio::sync::mpsc::Sender<Value>,
    /// Channel to receive the agent's emit value (one-shot).
    pub(crate) result_rx: tokio::sync::Mutex<Option<oneshot::Receiver<Value>>>,
}

impl AgentHandle {
    /// Create a new agent handle.
    pub(crate) fn new(
        agent_name: String,
        message_tx: tokio::sync::mpsc::Sender<Value>,
        result_rx: oneshot::Receiver<Value>,
    ) -> Self {
        Self {
            agent_name,
            inner: Arc::new(AgentHandleInner {
                message_tx,
                result_rx: tokio::sync::Mutex::new(Some(result_rx)),
            }),
        }
    }

    /// Send a message to this agent.
    ///
    /// # Errors
    ///
    /// Returns `SendError::AgentStopped` if the agent has already terminated.
    pub async fn send(&self, message: Value) -> Result<(), SendError> {
        self.inner
            .message_tx
            .send(message)
            .await
            .map_err(|_| SendError::AgentStopped)
    }

    /// Await the agent's emit value.
    ///
    /// # Errors
    ///
    /// Returns `AwaitError::AlreadyAwaited` if called more than once, or
    /// `AwaitError::AgentPanicked` if the agent terminated without emitting.
    pub async fn await_result(&self) -> Result<Value, AwaitError> {
        let mut guard = self.inner.result_rx.lock().await;
        let rx = guard.take().ok_or(AwaitError::AlreadyAwaited)?;
        rx.await.map_err(|_| AwaitError::AgentPanicked)
    }
}

/// Error when sending a message to an agent.
#[derive(Debug, Clone, Copy)]
pub enum SendError {
    /// The agent has already stopped.
    AgentStopped,
}

/// Error when awaiting an agent.
#[derive(Debug, Clone, Copy)]
pub enum AwaitError {
    /// The agent was already awaited.
    AlreadyAwaited,
    /// The agent panicked without emitting a value.
    AgentPanicked,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_display() {
        assert_eq!(Value::Int(42).to_string(), "42");
        assert_eq!(Value::Float(3.14).to_string(), "3.14");
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(Value::String("hello".into()).to_string(), "hello");
        assert_eq!(Value::Unit.to_string(), "()");
        assert_eq!(
            Value::List(vec![Value::Int(1), Value::Int(2)]).to_string(),
            "[1, 2]"
        );
    }

    #[test]
    fn value_equality() {
        assert_eq!(Value::Int(42), Value::Int(42));
        assert_ne!(Value::Int(42), Value::Int(43));
        assert_eq!(Value::String("a".into()), Value::String("a".into()));
        assert_ne!(Value::Int(1), Value::String("1".into()));
    }

    #[test]
    fn value_type_name() {
        assert_eq!(Value::Int(0).type_name(), "Int");
        assert_eq!(Value::String("".into()).type_name(), "String");
        assert_eq!(Value::List(vec![]).type_name(), "List");
    }
}
