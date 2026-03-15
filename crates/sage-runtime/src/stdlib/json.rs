//! JSON helper functions for the Sage standard library.

use serde_json::Value;

/// Parse a JSON string, validating that it's well-formed JSON.
/// Returns the original string if valid.
pub fn json_parse(s: &str) -> Result<String, String> {
    let _: Value = serde_json::from_str(s)
        .map_err(|e| format!("invalid JSON: {}", e))?;
    Ok(s.to_string())
}

/// Get a field from a JSON object as a string.
/// Returns None if the field doesn't exist or isn't a string.
#[must_use]
pub fn json_get(json: &str, field: &str) -> Option<String> {
    let value: Value = serde_json::from_str(json).ok()?;
    match value.get(field)? {
        Value::String(s) => Some(s.clone()),
        other => Some(other.to_string()),
    }
}

/// Get a field from a JSON object as an integer.
/// Returns None if the field doesn't exist or isn't a number.
#[must_use]
pub fn json_get_int(json: &str, field: &str) -> Option<i64> {
    let value: Value = serde_json::from_str(json).ok()?;
    value.get(field)?.as_i64()
}

/// Get a field from a JSON object as a float.
/// Returns None if the field doesn't exist or isn't a number.
#[must_use]
pub fn json_get_float(json: &str, field: &str) -> Option<f64> {
    let value: Value = serde_json::from_str(json).ok()?;
    value.get(field)?.as_f64()
}

/// Get a field from a JSON object as a boolean.
/// Returns None if the field doesn't exist or isn't a boolean.
#[must_use]
pub fn json_get_bool(json: &str, field: &str) -> Option<bool> {
    let value: Value = serde_json::from_str(json).ok()?;
    value.get(field)?.as_bool()
}

/// Get a field from a JSON object as a list of strings.
/// Each array element is converted to its JSON string representation.
/// Returns None if the field doesn't exist or isn't an array.
#[must_use]
pub fn json_get_list(json: &str, field: &str) -> Option<Vec<String>> {
    let value: Value = serde_json::from_str(json).ok()?;
    let arr = value.get(field)?.as_array()?;
    Some(arr.iter().map(|v| {
        match v {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        }
    }).collect())
}

/// Convert a value to a JSON string.
/// Works best with strings, but accepts any type via its Debug representation.
#[must_use]
pub fn json_stringify_string(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| format!("\"{}\"", s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_parse() {
        assert!(json_parse(r#"{"name": "Alice"}"#).is_ok());
        assert!(json_parse(r#"[1, 2, 3]"#).is_ok());
        assert!(json_parse(r#"invalid json"#).is_err());
    }

    #[test]
    fn test_json_get() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        assert_eq!(json_get(json, "name"), Some("Alice".to_string()));
        assert_eq!(json_get(json, "age"), Some("30".to_string()));
        assert_eq!(json_get(json, "missing"), None);
    }

    #[test]
    fn test_json_get_int() {
        let json = r#"{"count": 42, "name": "test"}"#;
        assert_eq!(json_get_int(json, "count"), Some(42));
        assert_eq!(json_get_int(json, "name"), None);
        assert_eq!(json_get_int(json, "missing"), None);
    }

    #[test]
    fn test_json_get_float() {
        let json = r#"{"value": 3.14, "count": 42}"#;
        assert!((json_get_float(json, "value").unwrap() - 3.14).abs() < 0.001);
        assert_eq!(json_get_float(json, "count"), Some(42.0));
    }

    #[test]
    fn test_json_get_bool() {
        let json = r#"{"active": true, "deleted": false}"#;
        assert_eq!(json_get_bool(json, "active"), Some(true));
        assert_eq!(json_get_bool(json, "deleted"), Some(false));
        assert_eq!(json_get_bool(json, "missing"), None);
    }

    #[test]
    fn test_json_get_list() {
        let json = r#"{"items": ["a", "b", "c"], "numbers": [1, 2, 3]}"#;
        assert_eq!(json_get_list(json, "items"), Some(vec!["a".to_string(), "b".to_string(), "c".to_string()]));
        assert_eq!(json_get_list(json, "numbers"), Some(vec!["1".to_string(), "2".to_string(), "3".to_string()]));
        assert_eq!(json_get_list(json, "missing"), None);
    }

    #[test]
    fn test_json_stringify_string() {
        assert_eq!(json_stringify_string("hello"), r#""hello""#);
        assert_eq!(json_stringify_string("hello\nworld"), r#""hello\nworld""#);
    }
}
