//! I/O helper functions for the Sage standard library.

use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Read as StdRead, Write as StdWrite};
use std::path::Path;

/// Read the entire contents of a file as a string.
pub fn read_file(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("failed to read file '{}': {}", path, e))
}

/// Write a string to a file, creating it if it doesn't exist or truncating it.
pub fn write_file(path: &str, contents: &str) -> Result<(), String> {
    fs::write(path, contents).map_err(|e| format!("failed to write file '{}': {}", path, e))
}

/// Append a string to a file, creating it if it doesn't exist.
pub fn append_file(path: &str, contents: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("failed to open file '{}' for append: {}", path, e))?;
    file.write_all(contents.as_bytes())
        .map_err(|e| format!("failed to append to file '{}': {}", path, e))
}

/// Check if a file or directory exists.
#[must_use]
pub fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// Delete a file.
pub fn delete_file(path: &str) -> Result<(), String> {
    fs::remove_file(path).map_err(|e| format!("failed to delete file '{}': {}", path, e))
}

/// List the contents of a directory.
pub fn list_dir(path: &str) -> Result<Vec<String>, String> {
    let entries = fs::read_dir(path)
        .map_err(|e| format!("failed to read directory '{}': {}", path, e))?;
    let mut result = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("failed to read directory entry: {}", e))?;
        result.push(entry.file_name().to_string_lossy().into_owned());
    }
    Ok(result)
}

/// Create a directory and all parent directories.
pub fn make_dir(path: &str) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|e| format!("failed to create directory '{}': {}", path, e))
}

/// Read a single line from stdin.
pub fn read_line() -> Result<String, String> {
    let stdin = io::stdin();
    let mut line = String::new();
    stdin
        .lock()
        .read_line(&mut line)
        .map_err(|e| format!("failed to read line from stdin: {}", e))?;
    // Remove trailing newline
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
    Ok(line)
}

/// Read all content from stdin until EOF.
pub fn read_all() -> Result<String, String> {
    let mut contents = String::new();
    io::stdin()
        .read_to_string(&mut contents)
        .map_err(|e| format!("failed to read from stdin: {}", e))?;
    Ok(contents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_read_write_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");
        let path_str = path.to_str().unwrap();

        write_file(path_str, "hello world").unwrap();
        assert_eq!(read_file(path_str).unwrap(), "hello world");
    }

    #[test]
    fn test_append_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");
        let path_str = path.to_str().unwrap();

        write_file(path_str, "hello").unwrap();
        append_file(path_str, " world").unwrap();
        assert_eq!(read_file(path_str).unwrap(), "hello world");
    }

    #[test]
    fn test_file_exists() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");
        let path_str = path.to_str().unwrap();

        assert!(!file_exists(path_str));
        fs::write(&path, "test").unwrap();
        assert!(file_exists(path_str));
    }

    #[test]
    fn test_delete_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");
        let path_str = path.to_str().unwrap();

        fs::write(&path, "test").unwrap();
        assert!(file_exists(path_str));
        delete_file(path_str).unwrap();
        assert!(!file_exists(path_str));
    }

    #[test]
    fn test_list_dir() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("a.txt"), "").unwrap();
        fs::write(dir.path().join("b.txt"), "").unwrap();

        let mut entries = list_dir(dir.path().to_str().unwrap()).unwrap();
        entries.sort();
        assert_eq!(entries, vec!["a.txt", "b.txt"]);
    }

    #[test]
    fn test_make_dir() {
        let dir = TempDir::new().unwrap();
        let nested = dir.path().join("a").join("b").join("c");
        let nested_str = nested.to_str().unwrap();

        assert!(!file_exists(nested_str));
        make_dir(nested_str).unwrap();
        assert!(file_exists(nested_str));
    }
}
