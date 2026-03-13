//! Dependency specification parsing.

use crate::error::PackageError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A dependency specification from sage.toml.
///
/// Requires a git URL and exactly one of: tag, branch, or rev.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DependencySpec {
    /// Git repository URL.
    pub git: String,
    /// Git tag (e.g., "v1.0.0").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// Git branch (e.g., "main").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// Git revision (full or short SHA).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
}

impl DependencySpec {
    /// Create a new dependency spec with a tag.
    pub fn with_tag(git: impl Into<String>, tag: impl Into<String>) -> Self {
        Self {
            git: git.into(),
            tag: Some(tag.into()),
            branch: None,
            rev: None,
        }
    }

    /// Create a new dependency spec with a branch.
    pub fn with_branch(git: impl Into<String>, branch: impl Into<String>) -> Self {
        Self {
            git: git.into(),
            tag: None,
            branch: Some(branch.into()),
            rev: None,
        }
    }

    /// Create a new dependency spec with a revision.
    pub fn with_rev(git: impl Into<String>, rev: impl Into<String>) -> Self {
        Self {
            git: git.into(),
            tag: None,
            branch: None,
            rev: Some(rev.into()),
        }
    }

    /// Validate that exactly one of tag/branch/rev is specified.
    pub fn validate(&self, package_name: &str) -> Result<(), PackageError> {
        let count = [&self.tag, &self.branch, &self.rev]
            .iter()
            .filter(|x| x.is_some())
            .count();

        if count != 1 {
            return Err(PackageError::InvalidDependencySpec {
                package: package_name.to_string(),
            });
        }
        Ok(())
    }

    /// Get the ref string (tag, branch, or rev).
    pub fn ref_string(&self) -> &str {
        self.tag
            .as_deref()
            .or(self.branch.as_deref())
            .or(self.rev.as_deref())
            .unwrap_or("HEAD")
    }

    /// Get the ref type for display.
    pub fn ref_type(&self) -> &'static str {
        if self.tag.is_some() {
            "tag"
        } else if self.branch.is_some() {
            "branch"
        } else if self.rev.is_some() {
            "rev"
        } else {
            "HEAD"
        }
    }
}

/// Parse dependencies from a TOML table.
pub fn parse_dependencies(
    table: &toml::Table,
) -> Result<HashMap<String, DependencySpec>, PackageError> {
    let mut deps = HashMap::new();

    for (name, value) in table {
        let spec = parse_dependency_value(name, value)?;
        deps.insert(name.clone(), spec);
    }

    Ok(deps)
}

fn parse_dependency_value(name: &str, value: &toml::Value) -> Result<DependencySpec, PackageError> {
    match value {
        toml::Value::Table(t) => {
            let git = t
                .get("git")
                .and_then(|v| v.as_str())
                .ok_or_else(|| PackageError::MissingGitUrl {
                    package: name.to_string(),
                })?
                .to_string();

            let tag = t.get("tag").and_then(|v| v.as_str()).map(String::from);
            let branch = t.get("branch").and_then(|v| v.as_str()).map(String::from);
            let rev = t.get("rev").and_then(|v| v.as_str()).map(String::from);

            let spec = DependencySpec {
                git,
                tag,
                branch,
                rev,
            };
            spec.validate(name)?;
            Ok(spec)
        }
        _ => Err(PackageError::InvalidDependencySpec {
            package: name.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tag_dependency() {
        let toml_str = r#"
git = "https://github.com/example/package"
tag = "v1.0.0"
"#;
        let value: toml::Value = toml::from_str(toml_str).unwrap();
        let spec = parse_dependency_value("test", &value).unwrap();

        assert_eq!(spec.git, "https://github.com/example/package");
        assert_eq!(spec.tag, Some("v1.0.0".to_string()));
        assert_eq!(spec.branch, None);
        assert_eq!(spec.rev, None);
    }

    #[test]
    fn parse_branch_dependency() {
        let toml_str = r#"
git = "https://github.com/example/package"
branch = "develop"
"#;
        let value: toml::Value = toml::from_str(toml_str).unwrap();
        let spec = parse_dependency_value("test", &value).unwrap();

        assert_eq!(spec.branch, Some("develop".to_string()));
    }

    #[test]
    fn parse_rev_dependency() {
        let toml_str = r#"
git = "https://github.com/example/package"
rev = "abc123"
"#;
        let value: toml::Value = toml::from_str(toml_str).unwrap();
        let spec = parse_dependency_value("test", &value).unwrap();

        assert_eq!(spec.rev, Some("abc123".to_string()));
    }

    #[test]
    fn reject_missing_git() {
        let toml_str = r#"
tag = "v1.0.0"
"#;
        let value: toml::Value = toml::from_str(toml_str).unwrap();
        let result = parse_dependency_value("test", &value);

        assert!(matches!(result, Err(PackageError::MissingGitUrl { .. })));
    }

    #[test]
    fn reject_multiple_refs() {
        let toml_str = r#"
git = "https://github.com/example/package"
tag = "v1.0.0"
branch = "main"
"#;
        let value: toml::Value = toml::from_str(toml_str).unwrap();
        let result = parse_dependency_value("test", &value);

        assert!(matches!(
            result,
            Err(PackageError::InvalidDependencySpec { .. })
        ));
    }

    #[test]
    fn reject_no_ref() {
        let toml_str = r#"
git = "https://github.com/example/package"
"#;
        let value: toml::Value = toml::from_str(toml_str).unwrap();
        let result = parse_dependency_value("test", &value);

        assert!(matches!(
            result,
            Err(PackageError::InvalidDependencySpec { .. })
        ));
    }

    #[test]
    fn parse_multiple_dependencies() {
        let table: toml::Table = toml::from_str(
            r#"
[foo]
git = "https://github.com/example/foo"
tag = "v1.0.0"

[bar]
git = "https://github.com/example/bar"
branch = "main"
"#,
        )
        .unwrap();

        let deps = parse_dependencies(&table).unwrap();
        assert_eq!(deps.len(), 2);
        assert!(deps.contains_key("foo"));
        assert!(deps.contains_key("bar"));
    }
}
