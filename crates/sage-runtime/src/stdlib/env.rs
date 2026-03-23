//! Environment variable functions for the Sage standard library.

/// Get an environment variable by name.
/// Returns None if the variable is not set.
#[must_use]
pub fn env_var(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

/// Get an environment variable by name, returning a default if not set.
#[must_use]
pub fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_var_missing() {
        assert_eq!(env_var("SAGE_TEST_NONEXISTENT_VAR_12345"), None);
    }

    #[test]
    fn test_env_var_exists() {
        std::env::set_var("SAGE_TEST_ENV_VAR", "hello");
        assert_eq!(env_var("SAGE_TEST_ENV_VAR"), Some("hello".to_string()));
        std::env::remove_var("SAGE_TEST_ENV_VAR");
    }

    #[test]
    fn test_env_or_missing() {
        assert_eq!(
            env_or("SAGE_TEST_NONEXISTENT_VAR_12345", "fallback"),
            "fallback"
        );
    }

    #[test]
    fn test_env_or_exists() {
        std::env::set_var("SAGE_TEST_ENV_OR_VAR", "present");
        assert_eq!(env_or("SAGE_TEST_ENV_OR_VAR", "fallback"), "present");
        std::env::remove_var("SAGE_TEST_ENV_OR_VAR");
    }
}
