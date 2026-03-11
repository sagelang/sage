//! Runtime environment for variable bindings.

use crate::value::Value;
use std::collections::HashMap;

/// A single scope containing variable bindings.
#[derive(Debug, Clone, Default)]
pub struct Scope {
    bindings: HashMap<String, Value>,
}

impl Scope {
    /// Create a new empty scope.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Define a variable in this scope.
    pub fn define(&mut self, name: impl Into<String>, value: Value) {
        self.bindings.insert(name.into(), value);
    }

    /// Look up a variable in this scope.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.bindings.get(name)
    }

    /// Update a variable in this scope. Returns false if not found.
    pub fn set(&mut self, name: &str, value: Value) -> bool {
        if self.bindings.contains_key(name) {
            self.bindings.insert(name.to_string(), value);
            true
        } else {
            false
        }
    }
}

/// The runtime environment with nested scopes.
#[derive(Debug, Clone)]
pub struct Environment {
    /// Stack of scopes (innermost last).
    scopes: Vec<Scope>,
    /// Agent beliefs (separate from regular scopes).
    beliefs: HashMap<String, Value>,
}

impl Environment {
    /// Create a new environment with a global scope.
    #[must_use]
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
            beliefs: HashMap::new(),
        }
    }

    /// Create an environment for an agent with initial beliefs.
    #[must_use]
    pub fn with_beliefs(beliefs: HashMap<String, Value>) -> Self {
        Self {
            scopes: vec![Scope::new()],
            beliefs,
        }
    }

    /// Push a new scope.
    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    /// Pop the current scope.
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Define a variable in the current scope.
    pub fn define(&mut self, name: impl Into<String>, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.define(name, value);
        }
    }

    /// Look up a variable, searching from innermost to outermost scope.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value);
            }
        }
        None
    }

    /// Update a variable in the nearest scope that contains it.
    /// Returns false if the variable is not found in any scope.
    #[allow(clippy::needless_pass_by_value)]
    pub fn set(&mut self, name: &str, value: Value) -> bool {
        for scope in self.scopes.iter_mut().rev() {
            if scope.set(name, value.clone()) {
                return true;
            }
        }
        false
    }

    /// Get a belief value (for `self.belief_name`).
    #[must_use]
    pub fn get_belief(&self, name: &str) -> Option<&Value> {
        self.beliefs.get(name)
    }

    /// Set a belief value.
    pub fn set_belief(&mut self, name: impl Into<String>, value: Value) {
        self.beliefs.insert(name.into(), value);
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_define_and_get() {
        let mut scope = Scope::new();
        scope.define("x", Value::Int(42));
        assert_eq!(scope.get("x"), Some(&Value::Int(42)));
        assert_eq!(scope.get("y"), None);
    }

    #[test]
    fn env_nested_scopes() {
        let mut env = Environment::new();
        env.define("x", Value::Int(1));

        env.push_scope();
        env.define("y", Value::Int(2));

        // Can see both x and y
        assert_eq!(env.get("x"), Some(&Value::Int(1)));
        assert_eq!(env.get("y"), Some(&Value::Int(2)));

        env.pop_scope();

        // Can see x, but not y
        assert_eq!(env.get("x"), Some(&Value::Int(1)));
        assert_eq!(env.get("y"), None);
    }

    #[test]
    fn env_shadowing() {
        let mut env = Environment::new();
        env.define("x", Value::Int(1));

        env.push_scope();
        env.define("x", Value::Int(2));

        // Inner x shadows outer
        assert_eq!(env.get("x"), Some(&Value::Int(2)));

        env.pop_scope();

        // Back to outer x
        assert_eq!(env.get("x"), Some(&Value::Int(1)));
    }

    #[test]
    fn env_beliefs() {
        let mut beliefs = HashMap::new();
        beliefs.insert("topic".to_string(), Value::String("Rust".into()));

        let env = Environment::with_beliefs(beliefs);
        assert_eq!(
            env.get_belief("topic"),
            Some(&Value::String("Rust".into()))
        );
    }

    #[test]
    fn env_set_variable() {
        let mut env = Environment::new();
        env.define("x", Value::Int(1));

        assert!(env.set("x", Value::Int(2)));
        assert_eq!(env.get("x"), Some(&Value::Int(2)));

        assert!(!env.set("y", Value::Int(3))); // Not defined
    }
}
