//! Dependency resolution.

use crate::cache::{PackageCache, ResolvedPackage, ResolvedPackagesMap};
use crate::dependency::{parse_dependencies, DependencySpec};
use crate::error::PackageError;
use crate::lock::{LockFile, LockedPackage};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Result of dependency resolution.
#[derive(Debug)]
pub struct ResolvedPackages {
    /// All resolved packages (including transitive).
    pub packages: ResolvedPackagesMap,
    /// The generated lock file.
    pub lock_file: LockFile,
}

/// Minimal manifest for reading package info.
#[derive(Debug, Deserialize)]
struct PackageManifest {
    project: ProjectInfo,
    #[serde(default)]
    dependencies: toml::Table,
}

#[derive(Debug, Deserialize)]
struct ProjectInfo {
    name: String,
    #[serde(default = "default_version")]
    version: String,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

/// Resolve all dependencies from a project's sage.toml.
pub fn resolve_dependencies(
    project_root: &Path,
    deps: &HashMap<String, DependencySpec>,
    existing_lock: Option<&LockFile>,
) -> Result<ResolvedPackages, PackageError> {
    let cache = PackageCache::new()?;
    let mut resolver = Resolver::new(cache, existing_lock);

    // Resolve all direct dependencies
    for (name, spec) in deps {
        resolver.resolve(name, spec, "root")?;
    }

    // Build the result
    let packages = resolver.resolved;
    let lock_file = LockFile {
        version: 1,
        packages: packages
            .values()
            .map(|p| LockedPackage {
                name: p.name.clone(),
                version: p.version.clone(),
                git: p.git.clone(),
                rev: p.rev.clone(),
                dependencies: p.dependencies.clone(),
            })
            .collect(),
    };

    // Save lock file
    let lock_path = project_root.join("sage.lock");
    lock_file.save(&lock_path)?;

    Ok(ResolvedPackages {
        packages,
        lock_file,
    })
}

/// Check if dependencies are up-to-date with lock file.
pub fn check_lock_freshness(deps: &HashMap<String, DependencySpec>, lock: &LockFile) -> bool {
    lock.matches_dependencies(deps)
}

/// Install dependencies from an existing lock file.
pub fn install_from_lock(lock: &LockFile) -> Result<ResolvedPackagesMap, PackageError> {
    let cache = PackageCache::new()?;
    let mut packages = ResolvedPackagesMap::new();

    for locked in lock.in_dependency_order() {
        // Create a spec from the locked info
        let spec = DependencySpec::with_rev(&locked.git, &locked.rev);

        // Fetch (will use cache if available)
        let (path, _) = cache.fetch(&locked.name, &spec)?;

        packages.insert(
            locked.name.clone(),
            ResolvedPackage {
                name: locked.name.clone(),
                version: locked.version.clone(),
                path,
                rev: locked.rev.clone(),
                git: locked.git.clone(),
                dependencies: locked.dependencies.clone(),
            },
        );
    }

    Ok(packages)
}

struct Resolver<'a> {
    cache: PackageCache,
    resolved: ResolvedPackagesMap,
    in_progress: HashSet<String>,
    existing_lock: Option<&'a LockFile>,
}

impl<'a> Resolver<'a> {
    fn new(cache: PackageCache, existing_lock: Option<&'a LockFile>) -> Self {
        Self {
            cache,
            resolved: ResolvedPackagesMap::new(),
            in_progress: HashSet::new(),
            existing_lock,
        }
    }

    fn resolve(
        &mut self,
        name: &str,
        spec: &DependencySpec,
        requirer: &str,
    ) -> Result<(), PackageError> {
        // Check for circular dependency
        if self.in_progress.contains(name) {
            // Circular deps in packages are allowed as long as we've already
            // resolved this package - just return
            return Ok(());
        }

        // Already resolved?
        if let Some(existing) = self.resolved.get(name) {
            // Check for version conflict
            if existing.git != spec.git {
                return Err(PackageError::IncompatibleVersions {
                    package: name.to_string(),
                    version_a: existing.rev.clone(),
                    requirer_a: "previously resolved".to_string(),
                    version_b: spec.ref_string().to_string(),
                    requirer_b: requirer.to_string(),
                });
            }
            return Ok(());
        }

        self.in_progress.insert(name.to_string());

        // Check if we can use the lock file
        let (path, rev) = if let Some(lock) = self.existing_lock {
            if let Some(locked) = lock.find(name) {
                if locked.git == spec.git {
                    // Use locked version
                    let locked_spec = DependencySpec::with_rev(&locked.git, &locked.rev);
                    self.cache.fetch(name, &locked_spec)?
                } else {
                    // Git URL changed - resolve fresh
                    self.cache.fetch(name, spec)?
                }
            } else {
                // Not in lock file - resolve fresh
                self.cache.fetch(name, spec)?
            }
        } else {
            // No lock file - resolve fresh
            self.cache.fetch(name, spec)?
        };

        // Read the package's manifest
        let manifest = self.read_manifest(&path, name)?;

        // Verify package name matches
        if manifest.project.name != name {
            return Err(PackageError::PackageNameMismatch {
                expected: name.to_string(),
                found: manifest.project.name,
            });
        }

        // Parse transitive dependencies
        let trans_deps = parse_dependencies(&manifest.dependencies)?;
        let dep_names: Vec<String> = trans_deps.keys().cloned().collect();

        // Store the resolved package
        self.resolved.insert(
            name.to_string(),
            ResolvedPackage {
                name: name.to_string(),
                version: manifest.project.version,
                path: path.clone(),
                rev: rev.clone(),
                git: spec.git.clone(),
                dependencies: dep_names.clone(),
            },
        );

        self.in_progress.remove(name);

        // Resolve transitive dependencies
        for (dep_name, dep_spec) in trans_deps {
            self.resolve(&dep_name, &dep_spec, name)?;
        }

        Ok(())
    }

    fn read_manifest(&self, path: &Path, name: &str) -> Result<PackageManifest, PackageError> {
        let manifest_path = path.join("sage.toml");
        let contents =
            std::fs::read_to_string(&manifest_path).map_err(|e| PackageError::IoError {
                message: format!("failed to read manifest for '{name}'"),
                source: e,
            })?;

        toml::from_str(&contents).map_err(|e| PackageError::InvalidManifest {
            package: name.to_string(),
            source: e,
        })
    }
}

/// Check if a package has a `run` statement (making it an executable, not a library).
pub fn check_is_library(path: &Path) -> Result<bool, PackageError> {
    // Read the entry file and check for `run` statement
    let manifest_path = path.join("sage.toml");
    let manifest_contents = std::fs::read_to_string(&manifest_path)?;
    let _manifest: PackageManifest =
        toml::from_str(&manifest_contents).map_err(|e| PackageError::InvalidManifest {
            package: path.display().to_string(),
            source: e,
        })?;

    // Get entry path
    #[derive(Deserialize)]
    struct FullManifest {
        project: FullProjectInfo,
    }
    #[derive(Deserialize)]
    struct FullProjectInfo {
        #[serde(default = "default_entry")]
        entry: String,
    }
    fn default_entry() -> String {
        "src/main.sg".to_string()
    }

    let full: FullManifest =
        toml::from_str(&manifest_contents).map_err(|e| PackageError::InvalidManifest {
            package: path.display().to_string(),
            source: e,
        })?;

    let entry_path = path.join(&full.project.entry);
    if !entry_path.exists() {
        // No entry file means it's a library
        return Ok(true);
    }

    let entry_contents = std::fs::read_to_string(&entry_path)?;

    // Simple check: look for `run` followed by identifier and semicolon
    // This is a rough check - a proper check would use the parser
    let has_run = entry_contents
        .lines()
        .any(|line| line.trim().starts_with("run ") && line.trim().ends_with(';'));

    Ok(!has_run)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_lock_freshness_matches() {
        let mut deps = HashMap::new();
        deps.insert(
            "foo".to_string(),
            DependencySpec::with_tag("https://github.com/example/foo", "v1.0.0"),
        );

        let lock = LockFile {
            version: 1,
            packages: vec![LockedPackage {
                name: "foo".to_string(),
                version: "1.0.0".to_string(),
                git: "https://github.com/example/foo".to_string(),
                rev: "abc123".to_string(),
                dependencies: vec![],
            }],
        };

        assert!(check_lock_freshness(&deps, &lock));
    }

    #[test]
    fn check_lock_freshness_missing_dep() {
        let mut deps = HashMap::new();
        deps.insert(
            "foo".to_string(),
            DependencySpec::with_tag("https://github.com/example/foo", "v1.0.0"),
        );
        deps.insert(
            "bar".to_string(),
            DependencySpec::with_tag("https://github.com/example/bar", "v2.0.0"),
        );

        let lock = LockFile {
            version: 1,
            packages: vec![LockedPackage {
                name: "foo".to_string(),
                version: "1.0.0".to_string(),
                git: "https://github.com/example/foo".to_string(),
                rev: "abc123".to_string(),
                dependencies: vec![],
            }],
        };

        assert!(!check_lock_freshness(&deps, &lock));
    }
}
