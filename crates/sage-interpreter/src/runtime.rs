//! Runtime entry point for executing Sage programs.

use crate::env::Environment;
use crate::error::{RuntimeError, RuntimeResult};
use crate::eval::{eval_block, ControlFlow, EvalContext};
use crate::llm::{LlmClient, LlmConfig};
use crate::observer::{NoOpObserver, SharedObserver};
use crate::value::Value;
use sage_parser::{EventKind, Program};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

/// Configuration for the Sage runtime.
#[derive(Clone)]
pub struct RuntimeConfig {
    /// LLM configuration.
    pub llm: LlmConfig,
    /// Observer for runtime events.
    pub observer: SharedObserver,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            llm: LlmConfig::default(),
            observer: Arc::new(NoOpObserver),
        }
    }
}

impl std::fmt::Debug for RuntimeConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeConfig")
            .field("llm", &self.llm)
            .field("observer", &"<observer>")
            .finish()
    }
}

/// The Sage runtime for executing programs.
pub struct Runtime {
    #[allow(dead_code)]
    config: RuntimeConfig,
    llm: Arc<LlmClient>,
    #[allow(dead_code)] // Will be used for event callbacks
    observer: SharedObserver,
}

impl Runtime {
    /// Create a new runtime with the given configuration.
    #[must_use]
    pub fn new(config: RuntimeConfig) -> Self {
        let llm = Arc::new(LlmClient::new(config.llm.clone()));
        let observer = Arc::clone(&config.observer);
        Self {
            config,
            llm,
            observer,
        }
    }

    /// Create a runtime with mock LLM for testing.
    #[must_use]
    pub fn mock() -> Self {
        Self::new(RuntimeConfig {
            llm: LlmConfig::mock(),
            observer: Arc::new(NoOpObserver),
        })
    }

    /// Create a runtime with an observer for tracking events.
    #[must_use]
    pub fn with_observer(observer: SharedObserver) -> Self {
        Self::new(RuntimeConfig {
            llm: LlmConfig::default(),
            observer,
        })
    }

    /// Run a program and return the result.
    ///
    /// # Errors
    ///
    /// Returns an error if the entry agent is not found, has no start handler,
    /// or any runtime error occurs during execution.
    pub async fn run(&self, program: Program) -> RuntimeResult<Value> {
        let program = Arc::new(program);

        // Find the entry agent
        let entry_name = &program.run_agent.name;
        let entry_agent = program
            .agents
            .iter()
            .find(|a| a.name.name == *entry_name)
            .ok_or_else(|| {
                RuntimeError::agent_not_found(entry_name, &program.run_agent.span)
            })?
            .clone();

        // Set up channels for the entry agent
        let (_message_tx, message_rx) = mpsc::channel::<Value>(32);
        let (result_tx, _result_rx) = oneshot::channel::<Value>();

        // Create the evaluation context for the entry agent
        let ctx = EvalContext::for_agent(
            Arc::clone(&program),
            Arc::clone(&self.llm),
            result_tx,
            message_rx,
        );

        // Entry agents have no beliefs
        let mut env = Environment::with_beliefs(HashMap::new());

        // Find and run the start handler
        let start_handler = entry_agent
            .handlers
            .iter()
            .find(|h| matches!(h.event, EventKind::Start))
            .ok_or_else(|| {
                RuntimeError::internal(
                    format!("Entry agent {entry_name} has no start handler"),
                    &entry_agent.span,
                )
            })?;

        // Run the start handler
        match eval_block(&start_handler.body, &mut env, &ctx).await? {
            ControlFlow::Emit(val) => {
                // Send the emit value
                if let Some(tx) = ctx.emit_tx {
                    let _ = tx.send(val.clone());
                }
                Ok(val)
            }
            ControlFlow::Continue => {
                // No emit - return Unit
                if let Some(tx) = ctx.emit_tx {
                    let _ = tx.send(Value::Unit);
                }
                Ok(Value::Unit)
            }
            ControlFlow::Return(_) => {
                // Return in entry agent - treat as Unit
                if let Some(tx) = ctx.emit_tx {
                    let _ = tx.send(Value::Unit);
                }
                Ok(Value::Unit)
            }
        }
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new(RuntimeConfig::default())
    }
}

/// Run a program with the default runtime configuration.
///
/// # Errors
///
/// Returns an error if the entry agent is not found or any runtime error occurs.
pub async fn run(program: Program) -> RuntimeResult<Value> {
    Runtime::default().run(program).await
}

/// Run a program with a mock LLM (for testing).
///
/// # Errors
///
/// Returns an error if the entry agent is not found or any runtime error occurs.
pub async fn run_mock(program: Program) -> RuntimeResult<Value> {
    Runtime::mock().run(program).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use sage_lexer::lex;
    use sage_parser::parse;
    use std::sync::Arc as StdArc;

    async fn run_source(source: &str) -> RuntimeResult<Value> {
        let lex_result = lex(source).expect("lexing failed");
        let source_arc: StdArc<str> = StdArc::from(source);
        let (program, errors) = parse(lex_result.tokens(), source_arc);
        assert!(errors.is_empty(), "parse errors: {errors:?}");
        let program = program.expect("should parse");
        run_mock(program).await
    }

    #[tokio::test]
    async fn run_minimal_program() {
        let source = r#"
            agent Main {
                on start {
                    emit(42);
                }
            }
            run Main;
        "#;

        let result = run_source(source).await.expect("should run");
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn run_with_arithmetic() {
        let source = r#"
            agent Main {
                on start {
                    let x = 2 + 3 * 4;
                    emit(x);
                }
            }
            run Main;
        "#;

        let result = run_source(source).await.expect("should run");
        assert_eq!(result, Value::Int(14));
    }

    #[tokio::test]
    async fn run_with_function() {
        let source = r#"
            fn add(a: Int, b: Int) -> Int {
                return a + b;
            }

            agent Main {
                on start {
                    emit(add(10, 20));
                }
            }
            run Main;
        "#;

        let result = run_source(source).await.expect("should run");
        assert_eq!(result, Value::Int(30));
    }

    #[tokio::test]
    async fn run_with_if() {
        let source = r#"
            agent Main {
                on start {
                    let x = 10;
                    if x > 5 {
                        emit(1);
                    } else {
                        emit(0);
                    }
                }
            }
            run Main;
        "#;

        let result = run_source(source).await.expect("should run");
        assert_eq!(result, Value::Int(1));
    }

    #[tokio::test]
    async fn run_with_for_loop() {
        let source = r#"
            agent Main {
                on start {
                    let sum = 0;
                    for x in [1, 2, 3, 4, 5] {
                        sum = sum + x;
                    }
                    emit(sum);
                }
            }
            run Main;
        "#;

        let result = run_source(source).await.expect("should run");
        assert_eq!(result, Value::Int(15));
    }

    #[tokio::test]
    async fn run_with_while_loop() {
        let source = r#"
            agent Main {
                on start {
                    let count = 0;
                    let sum = 0;
                    while count < 5 {
                        count = count + 1;
                        sum = sum + count;
                    }
                    emit(sum);
                }
            }
            run Main;
        "#;

        let result = run_source(source).await.expect("should run");
        assert_eq!(result, Value::Int(15)); // 1+2+3+4+5
    }

    #[tokio::test]
    async fn run_with_string_concat() {
        let source = r#"
            agent Main {
                on start {
                    let msg = "Hello, " ++ "World!";
                    emit(msg);
                }
            }
            run Main;
        "#;

        let result = run_source(source).await.expect("should run");
        assert_eq!(result, Value::String("Hello, World!".into()));
    }

    #[tokio::test]
    async fn run_with_list_ops() {
        let source = r#"
            agent Main {
                on start {
                    let items = [1, 2, 3];
                    let count = len(items);
                    emit(count);
                }
            }
            run Main;
        "#;

        let result = run_source(source).await.expect("should run");
        assert_eq!(result, Value::Int(3));
    }

    #[tokio::test]
    async fn run_spawn_and_await() {
        let source = r#"
            agent Worker {
                belief value: Int

                on start {
                    emit(self.value * 2);
                }
            }

            agent Main {
                on start {
                    let w = spawn Worker { value: 21 };
                    let result = await w;
                    emit(result);
                }
            }
            run Main;
        "#;

        let result = run_source(source).await.expect("should run");
        assert_eq!(result, Value::Int(42));
    }

    #[tokio::test]
    async fn run_with_infer() {
        let source = r#"
            agent Main {
                on start {
                    let response = infer("What is 2+2?");
                    emit(response);
                }
            }
            run Main;
        "#;

        let result = run_source(source).await.expect("should run");
        // Mock LLM returns a placeholder
        assert!(matches!(result, Value::String(_)));
    }

    #[tokio::test]
    async fn run_recursive_function() {
        let source = r#"
            fn factorial(n: Int) -> Int {
                if n <= 1 {
                    return 1;
                }
                return n * factorial(n - 1);
            }

            agent Main {
                on start {
                    emit(factorial(5));
                }
            }
            run Main;
        "#;

        let result = run_source(source).await.expect("should run");
        assert_eq!(result, Value::Int(120));
    }

    #[tokio::test]
    async fn run_with_string_interpolation() {
        let source = r#"
            agent Main {
                on start {
                    let name = "Sage";
                    let version = 1;
                    let msg = "Hello from {name} v{version}!";
                    emit(msg);
                }
            }
            run Main;
        "#;

        let result = run_source(source).await.expect("should run");
        assert_eq!(result, Value::String("Hello from Sage v1!".into()));
    }
}
