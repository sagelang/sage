//! Mock infrastructure for the Sage testing framework (RFC-0012).
//!
//! This module provides:
//! - `MockResponse` - represents either a value or error response
//! - `MockQueue` - thread-safe queue of mock responses
//! - `MockLlmClient` - mock implementation of LLM inference

use crate::error::{SageError, SageResult};
use serde::de::DeserializeOwned;
use std::sync::{Arc, Mutex};

/// A mock response for an `infer` call.
#[derive(Debug, Clone)]
pub enum MockResponse {
    /// A successful response with the given value.
    Value(serde_json::Value),
    /// A failure response with the given error message.
    Fail(String),
}

impl MockResponse {
    /// Create a successful mock response from a JSON-serializable value.
    pub fn value<T: serde::Serialize>(value: T) -> Self {
        Self::Value(serde_json::to_value(value).expect("failed to serialize mock value"))
    }

    /// Create a successful mock response from a string.
    pub fn string(s: impl Into<String>) -> Self {
        Self::Value(serde_json::Value::String(s.into()))
    }

    /// Create a failure mock response.
    pub fn fail(message: impl Into<String>) -> Self {
        Self::Fail(message.into())
    }
}

/// A thread-safe queue of mock responses.
///
/// Mock responses are consumed in order - the first `infer` call gets
/// the first mock, the second gets the second, etc.
#[derive(Debug, Clone, Default)]
pub struct MockQueue {
    responses: Arc<Mutex<Vec<MockResponse>>>,
}

impl MockQueue {
    /// Create a new empty mock queue.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a mock queue with the given responses.
    pub fn with_responses(responses: Vec<MockResponse>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses)),
        }
    }

    /// Add a mock response to the queue.
    pub fn push(&self, response: MockResponse) {
        self.responses.lock().unwrap().push(response);
    }

    /// Pop the next mock response from the queue.
    ///
    /// Returns `None` if the queue is empty.
    pub fn pop(&self) -> Option<MockResponse> {
        let mut queue = self.responses.lock().unwrap();
        if queue.is_empty() {
            None
        } else {
            Some(queue.remove(0))
        }
    }

    /// Check if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.responses.lock().unwrap().is_empty()
    }

    /// Get the number of remaining mock responses.
    pub fn len(&self) -> usize {
        self.responses.lock().unwrap().len()
    }
}

/// Mock LLM client for testing.
///
/// This client uses a `MockQueue` to return pre-configured responses
/// instead of making real API calls.
#[derive(Debug, Clone)]
pub struct MockLlmClient {
    queue: MockQueue,
}

impl MockLlmClient {
    /// Create a new mock client with an empty queue.
    pub fn new() -> Self {
        Self {
            queue: MockQueue::new(),
        }
    }

    /// Create a mock client with the given responses.
    pub fn with_responses(responses: Vec<MockResponse>) -> Self {
        Self {
            queue: MockQueue::with_responses(responses),
        }
    }

    /// Get a reference to the mock queue for adding responses.
    pub fn queue(&self) -> &MockQueue {
        &self.queue
    }

    /// Call the mock LLM with a prompt and return the raw string response.
    ///
    /// Returns an error if no mock responses are queued.
    pub async fn infer_string(&self, _prompt: &str) -> SageResult<String> {
        match self.queue.pop() {
            Some(MockResponse::Value(value)) => {
                // Convert JSON value to string
                match value {
                    serde_json::Value::String(s) => Ok(s),
                    other => Ok(other.to_string()),
                }
            }
            Some(MockResponse::Fail(msg)) => Err(SageError::Llm(msg)),
            None => Err(SageError::Llm(
                "infer called with no mock available (E054)".to_string(),
            )),
        }
    }

    /// Call the mock LLM with a prompt and parse the response as the given type.
    ///
    /// Returns an error if no mock responses are queued.
    pub async fn infer<T>(&self, _prompt: &str) -> SageResult<T>
    where
        T: DeserializeOwned,
    {
        match self.queue.pop() {
            Some(MockResponse::Value(value)) => serde_json::from_value(value).map_err(|e| {
                SageError::Llm(format!("failed to deserialize mock value: {e}"))
            }),
            Some(MockResponse::Fail(msg)) => Err(SageError::Llm(msg)),
            None => Err(SageError::Llm(
                "infer called with no mock available (E054)".to_string(),
            )),
        }
    }

    /// Call the mock LLM with schema-injected prompt for structured output.
    ///
    /// Returns an error if no mock responses are queued.
    pub async fn infer_structured<T>(&self, _prompt: &str, _schema: &str) -> SageResult<T>
    where
        T: DeserializeOwned,
    {
        // Same as infer - the schema is ignored for mocks
        match self.queue.pop() {
            Some(MockResponse::Value(value)) => serde_json::from_value(value).map_err(|e| {
                SageError::Llm(format!("failed to deserialize mock value: {e}"))
            }),
            Some(MockResponse::Fail(msg)) => Err(SageError::Llm(msg)),
            None => Err(SageError::Llm(
                "infer called with no mock available (E054)".to_string(),
            )),
        }
    }
}

impl Default for MockLlmClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_infer_string_returns_value() {
        let client = MockLlmClient::with_responses(vec![MockResponse::string("hello world")]);
        let result = client.infer_string("test").await.unwrap();
        assert_eq!(result, "hello world");
    }

    #[tokio::test]
    async fn mock_infer_string_returns_fail() {
        let client = MockLlmClient::with_responses(vec![MockResponse::fail("test error")]);
        let result = client.infer_string("test").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("test error"));
    }

    #[tokio::test]
    async fn mock_infer_empty_queue_returns_error() {
        let client = MockLlmClient::new();
        let result = client.infer_string("test").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("E054"));
    }

    #[tokio::test]
    async fn mock_queue_fifo_order() {
        let client = MockLlmClient::with_responses(vec![
            MockResponse::string("first"),
            MockResponse::string("second"),
            MockResponse::string("third"),
        ]);

        assert_eq!(client.infer_string("a").await.unwrap(), "first");
        assert_eq!(client.infer_string("b").await.unwrap(), "second");
        assert_eq!(client.infer_string("c").await.unwrap(), "third");
        assert!(client.infer_string("d").await.is_err());
    }

    #[tokio::test]
    async fn mock_infer_typed_value() {
        #[derive(Debug, serde::Deserialize, PartialEq)]
        struct Person {
            name: String,
            age: i32,
        }

        let client = MockLlmClient::with_responses(vec![MockResponse::value(
            serde_json::json!({ "name": "Ward", "age": 42 }),
        )]);

        let person: Person = client.infer("test").await.unwrap();
        assert_eq!(person.name, "Ward");
        assert_eq!(person.age, 42);
    }

    #[test]
    fn mock_queue_thread_safe() {
        use std::thread;

        let queue = MockQueue::with_responses(vec![
            MockResponse::string("1"),
            MockResponse::string("2"),
            MockResponse::string("3"),
        ]);

        let queue_clone = queue.clone();
        let handle = thread::spawn(move || {
            queue_clone.pop();
            queue_clone.pop();
        });

        handle.join().unwrap();
        assert_eq!(queue.len(), 1);
    }

    #[tokio::test]
    async fn mock_infer_structured() {
        #[derive(Debug, serde::Deserialize, PartialEq)]
        struct Summary {
            text: String,
            confidence: f64,
        }

        let client = MockLlmClient::with_responses(vec![MockResponse::value(serde_json::json!({
            "text": "A summary",
            "confidence": 0.95
        }))]);

        let summary: Summary = client.infer_structured("summarize", "schema").await.unwrap();
        assert_eq!(summary.text, "A summary");
        assert!((summary.confidence - 0.95).abs() < 0.001);
    }
}
