//! Integration tests for the package manager.

use sage_package::{parse_dependencies, DependencySpec, LockFile, LockedPackage, PackageCache};
use std::collections::HashMap;
use tempfile::TempDir;

#[test]
fn dependency_spec_serialization_roundtrip() {
    let spec = DependencySpec::with_tag("https://github.com/example/foo", "v1.0.0");
    let toml = toml::to_string(&spec).unwrap();
    let parsed: DependencySpec = toml::from_str(&toml).unwrap();
    assert_eq!(spec, parsed);
}

#[test]
fn lock_file_roundtrip() {
    let lock = LockFile {
        version: 1,
        packages: vec![
            LockedPackage {
                name: "foo".to_string(),
                version: "1.0.0".to_string(),
                git: "https://github.com/example/foo".to_string(),
                rev: "abc123def456789".to_string(),
                dependencies: vec![],
            },
            LockedPackage {
                name: "bar".to_string(),
                version: "2.0.0".to_string(),
                git: "https://github.com/example/bar".to_string(),
                rev: "def456abc789xyz".to_string(),
                dependencies: vec!["foo".to_string()],
            },
        ],
    };

    let dir = TempDir::new().unwrap();
    let lock_path = dir.path().join("sage.lock");

    lock.save(&lock_path).unwrap();
    let loaded = LockFile::load(&lock_path).unwrap();

    assert_eq!(loaded.version, lock.version);
    assert_eq!(loaded.packages.len(), lock.packages.len());
    assert_eq!(loaded.packages[0].name, "foo");
    assert_eq!(loaded.packages[1].dependencies, vec!["foo"]);
}

#[test]
fn parse_dependencies_table() {
    let table: toml::Table = toml::from_str(
        r#"
[http_client]
git = "https://github.com/sage-packages/http"
tag = "v1.0.0"

[json_parser]
git = "https://github.com/sage-packages/json"
branch = "main"

[utils]
git = "https://github.com/sage-packages/utils"
rev = "abc123"
"#,
    )
    .unwrap();

    let deps = parse_dependencies(&table).unwrap();
    assert_eq!(deps.len(), 3);

    assert!(deps.contains_key("http_client"));
    assert_eq!(deps["http_client"].tag, Some("v1.0.0".to_string()));

    assert!(deps.contains_key("json_parser"));
    assert_eq!(deps["json_parser"].branch, Some("main".to_string()));

    assert!(deps.contains_key("utils"));
    assert_eq!(deps["utils"].rev, Some("abc123".to_string()));
}

#[test]
fn lock_file_matches_deps() {
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

    assert!(lock.matches_dependencies(&deps));
}

#[test]
fn lock_file_does_not_match_missing_dep() {
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

    assert!(!lock.matches_dependencies(&deps));
}

#[test]
fn package_cache_can_be_created() {
    // Just verify the cache can be created without errors
    let cache = PackageCache::new().unwrap();
    assert!(cache.root().exists() || cache.root().parent().is_some());
}

#[test]
fn lock_file_dependency_ordering() {
    let lock = LockFile {
        version: 1,
        packages: vec![
            // Define in reverse order to test sorting
            LockedPackage {
                name: "app".to_string(),
                version: "1.0.0".to_string(),
                git: "https://example.com/app".to_string(),
                rev: "app123".to_string(),
                dependencies: vec!["core".to_string(), "utils".to_string()],
            },
            LockedPackage {
                name: "utils".to_string(),
                version: "1.0.0".to_string(),
                git: "https://example.com/utils".to_string(),
                rev: "utils123".to_string(),
                dependencies: vec!["core".to_string()],
            },
            LockedPackage {
                name: "core".to_string(),
                version: "1.0.0".to_string(),
                git: "https://example.com/core".to_string(),
                rev: "core123".to_string(),
                dependencies: vec![],
            },
        ],
    };

    let ordered = lock.in_dependency_order();
    let names: Vec<&str> = ordered.iter().map(|p| p.name.as_str()).collect();

    // core must come before utils and app
    // utils must come before app
    let core_pos = names.iter().position(|&n| n == "core").unwrap();
    let utils_pos = names.iter().position(|&n| n == "utils").unwrap();
    let app_pos = names.iter().position(|&n| n == "app").unwrap();

    assert!(core_pos < utils_pos, "core should come before utils");
    assert!(core_pos < app_pos, "core should come before app");
    assert!(utils_pos < app_pos, "utils should come before app");
}
