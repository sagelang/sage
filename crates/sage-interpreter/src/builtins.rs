//! Built-in functions for the Sage prelude.

// Builtins need to own args for the consistent call_builtin API
#![allow(clippy::needless_pass_by_value)]

use crate::error::{RuntimeError, RuntimeResult};
use crate::value::Value;
use sage_types::Span;
use std::time::Duration;

/// Evaluate a built-in function call.
pub async fn call_builtin(
    name: &str,
    args: Vec<Value>,
    span: &Span,
) -> RuntimeResult<Value> {
    match name {
        "print" => builtin_print(args, span),
        "len" => builtin_len(args, span),
        "push" => builtin_push(args, span),
        "join" => builtin_join(args, span),
        "str" => builtin_str(args, span),
        "int_to_str" => builtin_int_to_str(args, span),
        "str_contains" => builtin_str_contains(args, span),
        "sleep_ms" => builtin_sleep_ms(args, span).await,
        _ => Err(RuntimeError::function_not_found(name, span)),
    }
}

/// Check if a name is a built-in function.
#[must_use]
pub fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        "print" | "len" | "push" | "join" | "str" | "int_to_str" | "str_contains" | "sleep_ms"
    )
}

/// `print(String) -> Unit`
fn builtin_print(args: Vec<Value>, span: &Span) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::internal(
            format!("print expects 1 argument, got {}", args.len()),
            span,
        ));
    }

    // Print any value using Display
    println!("{}", args[0]);
    Ok(Value::Unit)
}

/// `len(List<T>) -> Int`
fn builtin_len(args: Vec<Value>, span: &Span) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::internal(
            format!("len expects 1 argument, got {}", args.len()),
            span,
        ));
    }

    match &args[0] {
        #[allow(clippy::cast_possible_wrap)]
        Value::List(items) => Ok(Value::Int(items.len() as i64)),
        #[allow(clippy::cast_possible_wrap)]
        Value::String(s) => Ok(Value::Int(s.len() as i64)),
        other => Err(RuntimeError::type_error("List or String", other.type_name(), span)),
    }
}

/// `push(List<T>, T) -> List<T>`
fn builtin_push(args: Vec<Value>, span: &Span) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::internal(
            format!("push expects 2 arguments, got {}", args.len()),
            span,
        ));
    }

    match &args[0] {
        Value::List(items) => {
            let mut new_list = items.clone();
            new_list.push(args[1].clone());
            Ok(Value::List(new_list))
        }
        other => Err(RuntimeError::type_error("List", other.type_name(), span)),
    }
}

/// `join(List<String>, String) -> String`
fn builtin_join(args: Vec<Value>, span: &Span) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::internal(
            format!("join expects 2 arguments, got {}", args.len()),
            span,
        ));
    }

    let list = match &args[0] {
        Value::List(items) => items,
        other => return Err(RuntimeError::type_error("List<String>", other.type_name(), span)),
    };

    let separator = match &args[1] {
        Value::String(s) => s,
        other => return Err(RuntimeError::type_error("String", other.type_name(), span)),
    };

    let strings: Result<Vec<&str>, _> = list
        .iter()
        .map(|v| match v {
            Value::String(s) => Ok(s.as_str()),
            other => Err(RuntimeError::type_error("String", other.type_name(), span)),
        })
        .collect();

    Ok(Value::String(strings?.join(separator)))
}

/// `str(T) -> String` - Convert any value to a string.
fn builtin_str(args: Vec<Value>, span: &Span) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::internal(
            format!("str expects 1 argument, got {}", args.len()),
            span,
        ));
    }

    let result = match &args[0] {
        Value::Int(n) => n.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::String(s) => s.clone(),
        Value::Unit => "()".to_string(),
        Value::List(items) => {
            let inner: Vec<String> = items.iter().map(ToString::to_string).collect();
            format!("[{}]", inner.join(", "))
        }
        Value::Agent(_) => "<agent>".to_string(),
        Value::Option(opt) => match opt {
            Some(v) => format!("Some({})", v),
            None => "None".to_string(),
        },
    };

    Ok(Value::String(result))
}

/// `int_to_str(Int) -> String`
fn builtin_int_to_str(args: Vec<Value>, span: &Span) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::internal(
            format!("int_to_str expects 1 argument, got {}", args.len()),
            span,
        ));
    }

    match &args[0] {
        Value::Int(n) => Ok(Value::String(n.to_string())),
        other => Err(RuntimeError::type_error("Int", other.type_name(), span)),
    }
}

/// `str_contains(String, String) -> Bool`
fn builtin_str_contains(args: Vec<Value>, span: &Span) -> RuntimeResult<Value> {
    if args.len() != 2 {
        return Err(RuntimeError::internal(
            format!("str_contains expects 2 arguments, got {}", args.len()),
            span,
        ));
    }

    let haystack = match &args[0] {
        Value::String(s) => s,
        other => return Err(RuntimeError::type_error("String", other.type_name(), span)),
    };

    let needle = match &args[1] {
        Value::String(s) => s,
        other => return Err(RuntimeError::type_error("String", other.type_name(), span)),
    };

    Ok(Value::Bool(haystack.contains(needle.as_str())))
}

/// `sleep_ms(Int) -> Unit`
async fn builtin_sleep_ms(args: Vec<Value>, span: &Span) -> RuntimeResult<Value> {
    if args.len() != 1 {
        return Err(RuntimeError::internal(
            format!("sleep_ms expects 1 argument, got {}", args.len()),
            span,
        ));
    }

    #[allow(clippy::cast_sign_loss)]
    let ms = match &args[0] {
        Value::Int(n) if *n >= 0 => *n as u64,
        Value::Int(_) => {
            return Err(RuntimeError::internal("sleep_ms requires non-negative integer", span))
        }
        other => return Err(RuntimeError::type_error("Int", other.type_name(), span)),
    };

    tokio::time::sleep(Duration::from_millis(ms)).await;
    Ok(Value::Unit)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_span() -> Span {
        Span::dummy()
    }

    #[test]
    fn test_is_builtin() {
        assert!(is_builtin("print"));
        assert!(is_builtin("len"));
        assert!(!is_builtin("foo"));
    }

    #[test]
    fn test_builtin_len() {
        let span = dummy_span();
        let result = builtin_len(vec![Value::List(vec![Value::Int(1), Value::Int(2)])], &span);
        assert_eq!(result.unwrap(), Value::Int(2));

        let result = builtin_len(vec![Value::String("hello".into())], &span);
        assert_eq!(result.unwrap(), Value::Int(5));
    }

    #[test]
    fn test_builtin_push() {
        let span = dummy_span();
        let result = builtin_push(
            vec![Value::List(vec![Value::Int(1)]), Value::Int(2)],
            &span,
        );
        assert_eq!(
            result.unwrap(),
            Value::List(vec![Value::Int(1), Value::Int(2)])
        );
    }

    #[test]
    fn test_builtin_int_to_str() {
        let span = dummy_span();
        let result = builtin_int_to_str(vec![Value::Int(42)], &span);
        assert_eq!(result.unwrap(), Value::String("42".into()));
    }

    #[test]
    fn test_builtin_str() {
        let span = dummy_span();

        // Int
        let result = builtin_str(vec![Value::Int(42)], &span);
        assert_eq!(result.unwrap(), Value::String("42".into()));

        // Float
        let result = builtin_str(vec![Value::Float(3.14)], &span);
        assert_eq!(result.unwrap(), Value::String("3.14".into()));

        // Bool
        let result = builtin_str(vec![Value::Bool(true)], &span);
        assert_eq!(result.unwrap(), Value::String("true".into()));

        // String (identity)
        let result = builtin_str(vec![Value::String("hello".into())], &span);
        assert_eq!(result.unwrap(), Value::String("hello".into()));
    }

    #[test]
    fn test_builtin_str_contains() {
        let span = dummy_span();
        let result = builtin_str_contains(
            vec![Value::String("hello world".into()), Value::String("world".into())],
            &span,
        );
        assert_eq!(result.unwrap(), Value::Bool(true));

        let result = builtin_str_contains(
            vec![Value::String("hello".into()), Value::String("world".into())],
            &span,
        );
        assert_eq!(result.unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_builtin_join() {
        let span = dummy_span();
        let result = builtin_join(
            vec![
                Value::List(vec![
                    Value::String("a".into()),
                    Value::String("b".into()),
                    Value::String("c".into()),
                ]),
                Value::String(", ".into()),
            ],
            &span,
        );
        assert_eq!(result.unwrap(), Value::String("a, b, c".into()));
    }
}
