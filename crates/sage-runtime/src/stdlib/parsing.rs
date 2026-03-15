//! Parsing helper functions for the Sage standard library.

/// Parse a boolean from a string.
/// Accepts "true", "false", "1", "0" (case-insensitive for true/false).
pub fn parse_bool(s: &str) -> Result<bool, String> {
    match s.trim().to_lowercase().as_str() {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        _ => Err(format!("invalid boolean: '{s}'")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bool() {
        assert_eq!(parse_bool("true"), Ok(true));
        assert_eq!(parse_bool("false"), Ok(false));
        assert_eq!(parse_bool("TRUE"), Ok(true));
        assert_eq!(parse_bool("FALSE"), Ok(false));
        assert_eq!(parse_bool("True"), Ok(true));
        assert_eq!(parse_bool("1"), Ok(true));
        assert_eq!(parse_bool("0"), Ok(false));
        assert_eq!(parse_bool("  true  "), Ok(true));
        assert!(parse_bool("yes").is_err());
        assert!(parse_bool("no").is_err());
        assert!(parse_bool("").is_err());
    }
}
