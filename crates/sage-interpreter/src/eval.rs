//! Expression and statement evaluation for the Sage interpreter.

use crate::builtins::{call_builtin, is_builtin};
use crate::env::Environment;
use crate::error::{RuntimeError, RuntimeResult};
use crate::llm::LlmClient;
use crate::value::{AgentHandle, AwaitError, SendError, Value};
use futures::future::BoxFuture;
use futures::FutureExt;
use sage_parser::{
    AgentDecl, BinOp, Block, Expr, FnDecl, Literal, Program, Stmt, StringPart, UnaryOp,
};
use sage_types::Span;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};

/// Control flow signals for statement execution.
#[derive(Debug)]
pub enum ControlFlow {
    /// Normal execution continues.
    Continue,
    /// Return from function with optional value.
    Return(Value),
    /// Emit value from agent.
    Emit(Value),
}

/// Context for evaluating expressions and statements.
pub struct EvalContext {
    /// The program being executed.
    pub program: Arc<Program>,
    /// LLM client for infer expressions.
    pub llm: Arc<LlmClient>,
    /// Whether we're in an agent context (allows self.* and emit).
    pub in_agent: bool,
    /// Channel to send emit values (only set in agent context).
    pub emit_tx: Option<oneshot::Sender<Value>>,
    /// Message receiver for this agent (only in agent context).
    pub message_rx: Option<Arc<Mutex<mpsc::Receiver<Value>>>>,
}

impl EvalContext {
    /// Create a new context for the main program.
    #[must_use]
    pub fn new(program: Arc<Program>, llm: Arc<LlmClient>) -> Self {
        Self {
            program,
            llm,
            in_agent: false,
            emit_tx: None,
            message_rx: None,
        }
    }

    /// Create a context for an agent.
    #[must_use]
    pub fn for_agent(
        program: Arc<Program>,
        llm: Arc<LlmClient>,
        emit_tx: oneshot::Sender<Value>,
        message_rx: mpsc::Receiver<Value>,
    ) -> Self {
        Self {
            program,
            llm,
            in_agent: true,
            emit_tx: Some(emit_tx),
            message_rx: Some(Arc::new(Mutex::new(message_rx))),
        }
    }

    /// Find an agent declaration by name.
    #[must_use]
    pub fn find_agent(&self, name: &str) -> Option<&AgentDecl> {
        self.program.agents.iter().find(|a| a.name.name == name)
    }

    /// Find a function declaration by name.
    #[must_use]
    pub fn find_function(&self, name: &str) -> Option<&FnDecl> {
        self.program.functions.iter().find(|f| f.name.name == name)
    }
}

/// Evaluate an expression.
///
/// Returns a boxed future to allow recursive async calls without
/// infinitely-sized future types.
#[allow(clippy::too_many_lines)]
pub fn eval_expr<'a>(
    expr: &'a Expr,
    env: &'a mut Environment,
    ctx: &'a EvalContext,
) -> BoxFuture<'a, RuntimeResult<Value>> {
    async move {
        match expr {
            Expr::Literal { value, .. } => Ok(eval_literal(value)),

            Expr::Var { name, .. } => env
                .get(&name.name)
                .cloned()
                .ok_or_else(|| RuntimeError::undefined_variable(&name.name, &name.span)),

            Expr::List { elements, .. } => {
                let mut values = Vec::with_capacity(elements.len());
                for elem in elements {
                    values.push(eval_expr(elem, env, ctx).await?);
                }
                Ok(Value::List(values))
            }

            Expr::Binary {
                op,
                left,
                right,
                span,
            } => {
                let left_val = eval_expr(left, env, ctx).await?;
                let right_val = eval_expr(right, env, ctx).await?;
                eval_binary_op(*op, left_val, right_val, span)
            }

            Expr::Unary { op, operand, span } => {
                let val = eval_expr(operand, env, ctx).await?;
                eval_unary_op(*op, val, span)
            }

            Expr::Paren { inner, .. } => eval_expr(inner, env, ctx).await,

            Expr::Call { name, args, span } => {
                let arg_values = eval_args(args, env, ctx).await?;

                // Check builtins first
                if is_builtin(&name.name) {
                    return call_builtin(&name.name, arg_values, span).await;
                }

                // Check user-defined functions
                if let Some(func) = ctx.find_function(&name.name).cloned() {
                    return eval_function_call(&func, arg_values, ctx).await;
                }

                Err(RuntimeError::function_not_found(&name.name, span))
            }

            Expr::SelfField { field, span } => {
                if !ctx.in_agent {
                    return Err(RuntimeError::internal("self used outside agent", span));
                }
                env.get_belief(&field.name)
                    .cloned()
                    .ok_or_else(|| RuntimeError::undefined_belief(&field.name, span))
            }

            Expr::SelfMethodCall { method, span, .. } => {
                Err(RuntimeError::function_not_found(&method.name, span))
            }

            Expr::Infer {
                template, span, ..
            } => {
                // Interpolate the template
                let prompt = interpolate_template(template, env)?;
                // Call the LLM
                let response = ctx.llm.infer(&prompt, span).await?;
                Ok(Value::String(response))
            }

            Expr::Spawn {
                agent,
                fields,
                span,
            } => {
                let agent_decl = ctx
                    .find_agent(&agent.name)
                    .ok_or_else(|| RuntimeError::agent_not_found(&agent.name, span))?
                    .clone();

                // Evaluate field initializers
                let mut beliefs = HashMap::new();
                for field in fields {
                    let value = eval_expr(&field.value, env, ctx).await?;
                    beliefs.insert(field.name.name.clone(), value);
                }

                // Spawn the agent
                Ok(spawn_agent(agent_decl, beliefs, ctx))
            }

            Expr::Await { handle, span } => {
                let handle_val = eval_expr(handle, env, ctx).await?;
                let agent_handle = handle_val
                    .as_agent()
                    .ok_or_else(|| RuntimeError::type_error("Agent", handle_val.type_name(), span))?;

                match agent_handle.await_result().await {
                    Ok(value) => Ok(value),
                    Err(AwaitError::AlreadyAwaited) => Err(RuntimeError::already_awaited(span)),
                    Err(AwaitError::AgentPanicked) => Err(RuntimeError::agent_panicked(span)),
                }
            }

            Expr::Send {
                handle,
                message,
                span,
            } => {
                let handle_val = eval_expr(handle, env, ctx).await?;
                let msg_val = eval_expr(message, env, ctx).await?;

                let agent_handle = handle_val
                    .as_agent()
                    .ok_or_else(|| RuntimeError::type_error("Agent", handle_val.type_name(), span))?;

                match agent_handle.send(msg_val).await {
                    Ok(()) => Ok(Value::Unit),
                    Err(SendError::AgentStopped) => Err(RuntimeError::send_failed(span)),
                }
            }

            Expr::Emit { value, span } => {
                if !ctx.in_agent {
                    return Err(RuntimeError::emit_outside_agent(span));
                }
                let val = eval_expr(value, env, ctx).await?;
                // Return a special marker - actual emit is handled in statement eval
                Ok(val)
            }

            Expr::StringInterp { template, .. } => {
                // Interpolate the template - same as infer but returns directly
                let result = interpolate_template(template, env)?;
                Ok(Value::String(result))
            }
        }
    }
    .boxed()
}

/// Evaluate a literal value.
fn eval_literal(lit: &Literal) -> Value {
    match lit {
        Literal::Int(n) => Value::Int(*n),
        Literal::Float(f) => Value::Float(*f),
        Literal::Bool(b) => Value::Bool(*b),
        Literal::String(s) => Value::String(s.clone()),
    }
}

/// Evaluate a binary operation.
#[allow(clippy::too_many_lines, clippy::needless_pass_by_value)]
fn eval_binary_op(op: BinOp, left: Value, right: Value, span: &Span) -> RuntimeResult<Value> {
    match op {
        BinOp::Add => match (&left, &right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            _ => Err(RuntimeError::type_error(
                "numeric",
                format!("{} and {}", left.type_name(), right.type_name()),
                span,
            )),
        },

        BinOp::Sub => match (&left, &right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            _ => Err(RuntimeError::type_error(
                "numeric",
                format!("{} and {}", left.type_name(), right.type_name()),
                span,
            )),
        },

        BinOp::Mul => match (&left, &right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            _ => Err(RuntimeError::type_error(
                "numeric",
                format!("{} and {}", left.type_name(), right.type_name()),
                span,
            )),
        },

        BinOp::Div => match (&left, &right) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(RuntimeError::division_by_zero(span))
                } else {
                    Ok(Value::Int(a / b))
                }
            }
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
            _ => Err(RuntimeError::type_error(
                "numeric",
                format!("{} and {}", left.type_name(), right.type_name()),
                span,
            )),
        },

        BinOp::Concat => match (&left, &right) {
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{a}{b}"))),
            _ => Err(RuntimeError::type_error(
                "String",
                format!("{} and {}", left.type_name(), right.type_name()),
                span,
            )),
        },

        BinOp::Eq => Ok(Value::Bool(left == right)),
        BinOp::Ne => Ok(Value::Bool(left != right)),

        BinOp::Lt => match (&left, &right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
            _ => Err(RuntimeError::type_error(
                "numeric",
                format!("{} and {}", left.type_name(), right.type_name()),
                span,
            )),
        },

        BinOp::Le => match (&left, &right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
            _ => Err(RuntimeError::type_error(
                "numeric",
                format!("{} and {}", left.type_name(), right.type_name()),
                span,
            )),
        },

        BinOp::Gt => match (&left, &right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
            _ => Err(RuntimeError::type_error(
                "numeric",
                format!("{} and {}", left.type_name(), right.type_name()),
                span,
            )),
        },

        BinOp::Ge => match (&left, &right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
            _ => Err(RuntimeError::type_error(
                "numeric",
                format!("{} and {}", left.type_name(), right.type_name()),
                span,
            )),
        },

        BinOp::And => match (&left, &right) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a && *b)),
            _ => Err(RuntimeError::type_error(
                "Bool",
                format!("{} and {}", left.type_name(), right.type_name()),
                span,
            )),
        },

        BinOp::Or => match (&left, &right) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a || *b)),
            _ => Err(RuntimeError::type_error(
                "Bool",
                format!("{} and {}", left.type_name(), right.type_name()),
                span,
            )),
        },
    }
}

/// Evaluate a unary operation.
#[allow(clippy::needless_pass_by_value)]
fn eval_unary_op(op: UnaryOp, val: Value, span: &Span) -> RuntimeResult<Value> {
    match op {
        UnaryOp::Neg => match val {
            Value::Int(n) => Ok(Value::Int(-n)),
            Value::Float(f) => Ok(Value::Float(-f)),
            _ => Err(RuntimeError::type_error("numeric", val.type_name(), span)),
        },
        UnaryOp::Not => match val {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            _ => Err(RuntimeError::type_error("Bool", val.type_name(), span)),
        },
    }
}

/// Evaluate function arguments.
fn eval_args<'a>(
    args: &'a [Expr],
    env: &'a mut Environment,
    ctx: &'a EvalContext,
) -> BoxFuture<'a, RuntimeResult<Vec<Value>>> {
    async move {
        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(eval_expr(arg, env, ctx).await?);
        }
        Ok(values)
    }
    .boxed()
}

/// Evaluate a function call.
fn eval_function_call<'a>(
    func: &'a FnDecl,
    args: Vec<Value>,
    ctx: &'a EvalContext,
) -> BoxFuture<'a, RuntimeResult<Value>> {
    async move {
        // Create new environment for function scope
        let mut func_env = Environment::new();

        // Bind parameters
        for (param, value) in func.params.iter().zip(args.into_iter()) {
            func_env.define(&param.name.name, value);
        }

        // Execute function body
        match eval_block(&func.body, &mut func_env, ctx).await? {
            ControlFlow::Return(val) => Ok(val),
            // Emit should not happen in functions, but treat as unit
            ControlFlow::Continue | ControlFlow::Emit(_) => Ok(Value::Unit),
        }
    }
    .boxed()
}

/// Interpolate a string template with values from the environment.
fn interpolate_template(
    template: &sage_parser::StringTemplate,
    env: &Environment,
) -> RuntimeResult<String> {
    let mut result = String::new();

    for part in &template.parts {
        match part {
            StringPart::Literal(s) => result.push_str(s),
            StringPart::Interpolation(ident) => {
                // Handle self.field syntax for belief access
                let value = if let Some(field) = ident.name.strip_prefix("self.") {
                    env.get_belief(field)
                        .ok_or_else(|| RuntimeError::undefined_belief(field, &ident.span))?
                } else {
                    // Try regular variable first, then belief
                    env.get(&ident.name)
                        .or_else(|| env.get_belief(&ident.name))
                        .ok_or_else(|| RuntimeError::undefined_variable(&ident.name, &ident.span))?
                };
                result.push_str(&value.to_string());
            }
        }
    }

    Ok(result)
}

/// Evaluate a block of statements.
pub fn eval_block<'a>(
    block: &'a Block,
    env: &'a mut Environment,
    ctx: &'a EvalContext,
) -> BoxFuture<'a, RuntimeResult<ControlFlow>> {
    async move {
        env.push_scope();
        let result = eval_statements(&block.stmts, env, ctx).await;
        env.pop_scope();
        result
    }
    .boxed()
}

/// Evaluate a sequence of statements.
fn eval_statements<'a>(
    stmts: &'a [Stmt],
    env: &'a mut Environment,
    ctx: &'a EvalContext,
) -> BoxFuture<'a, RuntimeResult<ControlFlow>> {
    async move {
        for stmt in stmts {
            match eval_stmt(stmt, env, ctx).await? {
                ControlFlow::Continue => {}
                cf => return Ok(cf),
            }
        }
        Ok(ControlFlow::Continue)
    }
    .boxed()
}

/// Evaluate a single statement.
pub fn eval_stmt<'a>(
    stmt: &'a Stmt,
    env: &'a mut Environment,
    ctx: &'a EvalContext,
) -> BoxFuture<'a, RuntimeResult<ControlFlow>> {
    async move {
        match stmt {
            Stmt::Let { name, value, .. } => {
                let val = eval_expr(value, env, ctx).await?;
                env.define(&name.name, val);
                Ok(ControlFlow::Continue)
            }

            Stmt::Assign { name, value, .. } => {
                let val = eval_expr(value, env, ctx).await?;
                env.set(&name.name, val);
                Ok(ControlFlow::Continue)
            }

            Stmt::Return { value, .. } => {
                let val = match value {
                    Some(expr) => eval_expr(expr, env, ctx).await?,
                    None => Value::Unit,
                };
                Ok(ControlFlow::Return(val))
            }

            Stmt::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                let cond = eval_expr(condition, env, ctx).await?;
                if cond.is_truthy() {
                    eval_block(then_block, env, ctx).await
                } else if let Some(else_branch) = else_block {
                    match else_branch {
                        sage_parser::ElseBranch::Block(block) => eval_block(block, env, ctx).await,
                        sage_parser::ElseBranch::ElseIf(stmt) => eval_stmt(stmt, env, ctx).await,
                    }
                } else {
                    Ok(ControlFlow::Continue)
                }
            }

            Stmt::For {
                var, iter, body, span,
            } => {
                let iter_val = eval_expr(iter, env, ctx).await?;
                let items = iter_val
                    .as_list()
                    .ok_or_else(|| RuntimeError::type_error("List", iter_val.type_name(), span))?
                    .to_vec();

                for item in items {
                    env.push_scope();
                    env.define(&var.name, item);
                    match eval_block(body, env, ctx).await? {
                        ControlFlow::Continue => {}
                        cf => {
                            env.pop_scope();
                            return Ok(cf);
                        }
                    }
                    env.pop_scope();
                }
                Ok(ControlFlow::Continue)
            }

            Stmt::Expr { expr, span } => {
                // Check for emit expression specially
                if let Expr::Emit { value, .. } = expr {
                    let val = eval_expr(value, env, ctx).await?;
                    if ctx.in_agent {
                        return Ok(ControlFlow::Emit(val));
                    }
                    return Err(RuntimeError::emit_outside_agent(span));
                }

                eval_expr(expr, env, ctx).await?;
                Ok(ControlFlow::Continue)
            }
        }
    }
    .boxed()
}

/// Spawn a new agent and return a handle to it.
fn spawn_agent(
    agent: AgentDecl,
    beliefs: HashMap<String, Value>,
    ctx: &EvalContext,
) -> Value {
    let (message_tx, message_rx) = mpsc::channel::<Value>(32);
    let (result_tx, result_rx) = oneshot::channel::<Value>();

    let handle = AgentHandle::new(agent.name.name.clone(), message_tx, result_rx);

    // Clone what we need for the spawned task
    let program = Arc::clone(&ctx.program);
    let llm = Arc::clone(&ctx.llm);

    // Spawn the agent task
    tokio::spawn(async move {
        let agent_ctx = EvalContext::for_agent(program, llm, result_tx, message_rx);
        let mut env = Environment::with_beliefs(beliefs);

        // Find and run the start handler
        if let Some(start_handler) = agent.handlers.iter().find(|h| {
            matches!(h.event, sage_parser::EventKind::Start)
        }) {
            match eval_block(&start_handler.body, &mut env, &agent_ctx).await {
                Ok(ControlFlow::Emit(val)) => {
                    if let Some(tx) = agent_ctx.emit_tx {
                        let _ = tx.send(val);
                    }
                }
                Ok(_) => {
                    // No emit - send Unit
                    if let Some(tx) = agent_ctx.emit_tx {
                        let _ = tx.send(Value::Unit);
                    }
                }
                Err(e) => {
                    eprintln!("Agent {} error: {e}", agent.name.name);
                }
            }
        }

        // TODO: Handle message loop for on message handlers
    });

    Value::Agent(handle)
}
