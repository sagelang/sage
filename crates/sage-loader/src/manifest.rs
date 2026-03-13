//! Project manifest (sage.toml) parsing.

use crate::error::LoadError;
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// A Sage project manifest (sage.toml).
#[derive(Debug, Clone, Deserialize)]
pub struct ProjectManifest {
    pub project: ProjectConfig,
    #[serde(default)]
    pub dependencies: toml::Table,
}

/// The [project] section of sage.toml.
#[derive(Debug, Clone, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_entry")]
    pub entry: PathBuf,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_entry() -> PathBuf {
    PathBuf::from("src/main.sg")
}

impl ProjectManifest {
    /// Load a manifest from a sage.toml file.
    pub fn load(path: &Path) -> Result<Self, LoadError> {
        let contents = std::fs::read_to_string(path).map_err(|e| LoadError::IoError {
            path: path.to_path_buf(),
            source: e,
        })?;

        toml::from_str(&contents).map_err(|e| LoadError::InvalidManifest {
            path: path.to_path_buf(),
            source: e,
        })
    }

    /// Find a sage.toml file by searching upward from the given directory.
    pub fn find(start_dir: &Path) -> Option<PathBuf> {
        let mut current = start_dir.to_path_buf();
        loop {
            let manifest_path = current.join("sage.toml");
            if manifest_path.exists() {
                return Some(manifest_path);
            }
            if !current.pop() {
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_manifest() {
        let toml = r#"
[project]
name = "test"
"#;
        let manifest: ProjectManifest = toml::from_str(toml).unwrap();
        assert_eq!(manifest.project.name, "test");
        assert_eq!(manifest.project.version, "0.1.0");
        assert_eq!(manifest.project.entry, PathBuf::from("src/main.sg"));
    }

    #[test]
    fn parse_full_manifest() {
        let toml = r#"
[project]
name = "research"
version = "1.2.3"
entry = "src/app.sg"

[dependencies]
"#;
        let manifest: ProjectManifest = toml::from_str(toml).unwrap();
        assert_eq!(manifest.project.name, "research");
        assert_eq!(manifest.project.version, "1.2.3");
        assert_eq!(manifest.project.entry, PathBuf::from("src/app.sg"));
    }
}
