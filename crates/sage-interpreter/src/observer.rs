//! Runtime observer for tracking execution events.
//!
//! This module provides a trait for observing runtime events like agent spawns,
//! LLM inference calls, and completions. Useful for building progress indicators
//! and debugging tools.

use std::sync::Arc;
use std::time::Duration;

/// Events emitted by the Sage runtime during execution.
#[derive(Debug, Clone)]
pub enum RuntimeEvent {
    /// An agent has been spawned.
    AgentSpawned {
        /// Name of the agent type.
        agent_type: String,
        /// Unique ID for this agent instance.
        agent_id: u64,
    },
    /// An LLM inference call has started.
    InferStarted {
        /// The prompt being sent to the LLM.
        prompt: String,
        /// Unique ID for this inference call.
        infer_id: u64,
    },
    /// An LLM inference call has completed.
    InferCompleted {
        /// Unique ID for this inference call.
        infer_id: u64,
        /// How long the inference took.
        duration: Duration,
        /// Whether it succeeded.
        success: bool,
    },
    /// An agent has completed execution.
    AgentCompleted {
        /// Unique ID for this agent instance.
        agent_id: u64,
    },
    /// Program execution has started.
    ProgramStarted {
        /// Name of the entry agent.
        entry_agent: String,
    },
    /// Program execution has completed.
    ProgramCompleted {
        /// Total execution duration.
        duration: Duration,
    },
}

/// Trait for observing runtime events.
///
/// Implement this trait to receive callbacks when significant events occur
/// during program execution.
pub trait RuntimeObserver: Send + Sync {
    /// Called when a runtime event occurs.
    fn on_event(&self, event: RuntimeEvent);
}

/// A no-op observer that ignores all events.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoOpObserver;

impl RuntimeObserver for NoOpObserver {
    fn on_event(&self, _event: RuntimeEvent) {}
}

/// An observer that can be shared across threads.
pub type SharedObserver = Arc<dyn RuntimeObserver>;
