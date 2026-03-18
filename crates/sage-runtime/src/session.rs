//! Phase 3: Session types infrastructure for protocol verification.
//!
//! This module provides runtime support for session types, enabling
//! protocol verification at runtime. It includes:
//!
//! - `SessionId`: Unique identifier for protocol sessions
//! - `SenderHandle`: Handle for replying to messages within a session
//! - `SessionRegistry`: Per-agent registry of active sessions
//! - `ProtocolStateMachine`: Trait for protocol state machines

use crate::error::{SageError, SageResult};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Unique identifier for a protocol session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(u64);

impl SessionId {
    /// Create a new session ID with the given value.
    #[must_use]
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the raw session ID value.
    #[must_use]
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "session-{}", self.0)
    }
}

/// Handle for sending replies within a protocol session.
///
/// This is used by `reply()` to send messages back to the sender
/// within the context of a session.
#[derive(Debug, Clone)]
pub struct SenderHandle {
    /// Channel for sending reply messages.
    reply_tx: mpsc::Sender<crate::agent::Message>,
    /// The protocol this session belongs to (if any).
    pub protocol: Option<String>,
    /// The session ID for this message exchange.
    pub session_id: Option<SessionId>,
}

impl SenderHandle {
    /// Create a new sender handle.
    #[must_use]
    pub fn new(
        reply_tx: mpsc::Sender<crate::agent::Message>,
        protocol: Option<String>,
        session_id: Option<SessionId>,
    ) -> Self {
        Self {
            reply_tx,
            protocol,
            session_id,
        }
    }

    /// Send a reply message.
    pub async fn send<M: serde::Serialize>(&self, msg: M) -> SageResult<()> {
        let message = crate::agent::Message::new(msg)?;
        self.reply_tx
            .send(message)
            .await
            .map_err(|e| SageError::Agent(format!("Failed to send reply: {e}")))
    }
}

/// State of an active protocol session.
#[derive(Debug)]
pub struct SessionState {
    /// The protocol this session is following.
    pub protocol: String,
    /// The current state of the protocol state machine.
    pub state: Box<dyn ProtocolStateMachine>,
    /// The role this agent plays in the protocol.
    pub role: String,
    /// Handle to send messages to the session partner.
    pub partner: SenderHandle,
}

/// Registry of active protocol sessions for an agent.
///
/// Each agent maintains its own session registry to track
/// ongoing protocol sessions.
#[derive(Debug, Default)]
pub struct SessionRegistry {
    /// Active sessions indexed by session ID.
    sessions: HashMap<SessionId, SessionState>,
    /// Counter for generating unique session IDs.
    next_session_id: AtomicU64,
}

impl SessionRegistry {
    /// Create a new empty session registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Generate a new unique session ID.
    pub fn next_id(&self) -> SessionId {
        SessionId(self.next_session_id.fetch_add(1, Ordering::SeqCst))
    }

    /// Start a new protocol session.
    pub fn start_session(
        &mut self,
        session_id: SessionId,
        protocol: String,
        role: String,
        state: Box<dyn ProtocolStateMachine>,
        partner: SenderHandle,
    ) {
        self.sessions.insert(
            session_id,
            SessionState {
                protocol,
                state,
                role,
                partner,
            },
        );
    }

    /// Get a session by ID.
    #[must_use]
    pub fn get(&self, session_id: &SessionId) -> Option<&SessionState> {
        self.sessions.get(session_id)
    }

    /// Get a mutable reference to a session by ID.
    pub fn get_mut(&mut self, session_id: &SessionId) -> Option<&mut SessionState> {
        self.sessions.get_mut(session_id)
    }

    /// Remove and return a session (e.g., when protocol completes).
    pub fn remove(&mut self, session_id: &SessionId) -> Option<SessionState> {
        self.sessions.remove(session_id)
    }

    /// Check if a session exists.
    #[must_use]
    pub fn has(&self, session_id: &SessionId) -> bool {
        self.sessions.contains_key(session_id)
    }

    /// Get the number of active sessions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    /// Check if there are no active sessions.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }
}

/// Protocol violation error details.
#[derive(Debug, Clone)]
pub enum ProtocolViolation {
    /// Received an unexpected message for the current protocol state.
    UnexpectedMessage {
        /// The protocol that was violated.
        protocol: String,
        /// The expected message type(s).
        expected: String,
        /// The received message type.
        received: String,
        /// The current state when the violation occurred.
        state: String,
    },

    /// Protocol terminated early (session ended before completion).
    EarlyTermination {
        /// The protocol that was violated.
        protocol: String,
        /// The state when termination occurred.
        state: String,
    },

    /// Message received from wrong sender role.
    WrongSender {
        /// The protocol that was violated.
        protocol: String,
        /// The expected sender role.
        expected_role: String,
        /// The actual sender role.
        actual_role: String,
    },

    /// No session found for the given session ID.
    NoSession {
        /// The missing session ID.
        session_id: SessionId,
    },

    /// Attempt to reply outside of a message handler.
    ReplyOutsideHandler,
}

impl std::fmt::Display for ProtocolViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolViolation::UnexpectedMessage {
                protocol,
                expected,
                received,
                state,
            } => write!(
                f,
                "unexpected message in protocol '{}': expected '{}', got '{}' (state: {})",
                protocol, expected, received, state
            ),
            ProtocolViolation::EarlyTermination { protocol, state } => {
                write!(
                    f,
                    "protocol '{}' terminated early in state '{}'",
                    protocol, state
                )
            }
            ProtocolViolation::WrongSender {
                protocol,
                expected_role,
                actual_role,
            } => write!(
                f,
                "wrong sender in protocol '{}': expected role '{}', got '{}'",
                protocol, expected_role, actual_role
            ),
            ProtocolViolation::NoSession { session_id } => {
                write!(f, "no session found with id {}", session_id)
            }
            ProtocolViolation::ReplyOutsideHandler => {
                write!(f, "reply() called outside of message handler")
            }
        }
    }
}

impl From<ProtocolViolation> for SageError {
    fn from(v: ProtocolViolation) -> Self {
        SageError::Protocol(v.to_string())
    }
}

/// Trait for protocol state machines.
///
/// This trait is implemented by generated code for each protocol declaration.
/// It tracks the current state and validates message transitions.
pub trait ProtocolStateMachine: Send + Sync + std::fmt::Debug {
    /// Get the name of the current state.
    fn state_name(&self) -> &str;

    /// Check if a message type can be sent from the given role in the current state.
    fn can_send(&self, msg_type: &str, from_role: &str) -> bool;

    /// Check if a message type can be received by the given role in the current state.
    fn can_receive(&self, msg_type: &str, to_role: &str) -> bool;

    /// Transition the state machine based on a message.
    ///
    /// # Errors
    ///
    /// Returns a `ProtocolViolation` if the transition is invalid.
    fn transition(&mut self, msg_type: &str) -> Result<(), ProtocolViolation>;

    /// Check if the protocol has reached a terminal (accepting) state.
    fn is_terminal(&self) -> bool;

    /// Get the protocol name.
    fn protocol_name(&self) -> &str;

    /// Clone the state machine into a boxed trait object.
    fn clone_box(&self) -> Box<dyn ProtocolStateMachine>;
}

/// Thread-safe shared session registry.
pub type SharedSessionRegistry = Arc<RwLock<SessionRegistry>>;

/// Create a new shared session registry.
#[must_use]
pub fn shared_registry() -> SharedSessionRegistry {
    Arc::new(RwLock::new(SessionRegistry::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_id_display() {
        let id = SessionId::new(42);
        assert_eq!(format!("{}", id), "session-42");
        assert_eq!(id.value(), 42);
    }

    #[test]
    fn session_registry_basic() {
        let registry = SessionRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        let id1 = registry.next_id();
        let id2 = registry.next_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn protocol_violation_display() {
        let violation = ProtocolViolation::UnexpectedMessage {
            protocol: "PingPong".to_string(),
            expected: "Pong".to_string(),
            received: "Ping".to_string(),
            state: "AwaitingPong".to_string(),
        };
        let msg = format!("{}", violation);
        assert!(msg.contains("PingPong"));
        assert!(msg.contains("Pong"));
        assert!(msg.contains("Ping"));
    }

    #[test]
    fn protocol_violation_to_error() {
        let violation = ProtocolViolation::ReplyOutsideHandler;
        let error: SageError = violation.into();
        assert!(matches!(error, SageError::Protocol(_)));
    }
}
