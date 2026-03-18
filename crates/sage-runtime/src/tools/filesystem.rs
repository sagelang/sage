//! RFC-0011: FileSystem tool for Sage agents.
//!
//! Provides the `Fs` tool with file operations.

use crate::error::{SageError, SageResult};
use crate::mock::{try_get_mock, MockResponse};
use std::path::PathBuf;

/// FileSystem client for Sage agents.
///
/// Created via `FsClient::new()` or `FsClient::with_root()`.
#[derive(Debug, Clone)]
pub struct FsClient {
    root: PathBuf,
}

impl FsClient {
    /// Create a new filesystem client with current directory as root.
    pub fn new() -> Self {
        Self {
            root: PathBuf::from("."),
        }
    }

    /// Create a new filesystem client from environment variables.
    ///
    /// Reads:
    /// - `SAGE_FS_ROOT`: Root directory for filesystem operations (default: ".")
    pub fn from_env() -> Self {
        let root = std::env::var("SAGE_FS_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));

        Self { root }
    }

    /// Create a new filesystem client with the given root directory.
    pub fn with_root(root: PathBuf) -> Self {
        Self { root }
    }

    /// Resolve a path relative to the root directory.
    fn resolve_path(&self, path: &str) -> PathBuf {
        self.root.join(path)
    }

    /// Read a file's contents as a string.
    ///
    /// # Arguments
    /// * `path` - Path to the file (relative to root)
    ///
    /// # Returns
    /// The file contents as a string.
    pub async fn read(&self, path: String) -> SageResult<String> {
        // Check for mock response first
        if let Some(mock_response) = try_get_mock("Fs", "read") {
            return Self::apply_mock_string(mock_response);
        }

        let full_path = self.resolve_path(&path);
        let content = tokio::fs::read_to_string(&full_path).await?;
        Ok(content)
    }

    /// Write content to a file.
    ///
    /// # Arguments
    /// * `path` - Path to the file (relative to root)
    /// * `content` - Content to write
    ///
    /// # Returns
    /// Unit on success.
    pub async fn write(&self, path: String, content: String) -> SageResult<()> {
        // Check for mock response first
        if let Some(mock_response) = try_get_mock("Fs", "write") {
            return Self::apply_mock_unit(mock_response);
        }

        let full_path = self.resolve_path(&path);
        // Create parent directories if they don't exist
        if let Some(parent) = full_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&full_path, content).await?;
        Ok(())
    }

    /// Check if a path exists.
    ///
    /// # Arguments
    /// * `path` - Path to check (relative to root)
    ///
    /// # Returns
    /// `true` if the path exists, `false` otherwise.
    pub async fn exists(&self, path: String) -> SageResult<bool> {
        // Check for mock response first
        if let Some(mock_response) = try_get_mock("Fs", "exists") {
            return Self::apply_mock_bool(mock_response);
        }

        let full_path = self.resolve_path(&path);
        Ok(full_path.exists())
    }

    /// List files and directories in a path.
    ///
    /// # Arguments
    /// * `path` - Directory path (relative to root)
    ///
    /// # Returns
    /// List of file/directory names.
    pub async fn list(&self, path: String) -> SageResult<Vec<String>> {
        // Check for mock response first
        if let Some(mock_response) = try_get_mock("Fs", "list") {
            return Self::apply_mock_vec_string(mock_response);
        }

        let full_path = self.resolve_path(&path);
        let mut entries = tokio::fs::read_dir(&full_path).await?;
        let mut names = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            if let Some(name) = entry.file_name().to_str() {
                names.push(name.to_string());
            }
        }
        Ok(names)
    }

    /// Delete a file.
    ///
    /// # Arguments
    /// * `path` - Path to the file (relative to root)
    ///
    /// # Returns
    /// Unit on success.
    pub async fn delete(&self, path: String) -> SageResult<()> {
        // Check for mock response first
        if let Some(mock_response) = try_get_mock("Fs", "delete") {
            return Self::apply_mock_unit(mock_response);
        }

        let full_path = self.resolve_path(&path);
        tokio::fs::remove_file(&full_path).await?;
        Ok(())
    }

    /// Apply a mock response for String.
    fn apply_mock_string(mock_response: MockResponse) -> SageResult<String> {
        match mock_response {
            MockResponse::Value(v) => serde_json::from_value(v)
                .map_err(|e| SageError::Tool(format!("mock deserialize: {e}"))),
            MockResponse::Fail(msg) => Err(SageError::Tool(msg)),
        }
    }

    /// Apply a mock response for ().
    fn apply_mock_unit(mock_response: MockResponse) -> SageResult<()> {
        match mock_response {
            MockResponse::Value(_) => Ok(()),
            MockResponse::Fail(msg) => Err(SageError::Tool(msg)),
        }
    }

    /// Apply a mock response for bool.
    fn apply_mock_bool(mock_response: MockResponse) -> SageResult<bool> {
        match mock_response {
            MockResponse::Value(v) => serde_json::from_value(v)
                .map_err(|e| SageError::Tool(format!("mock deserialize: {e}"))),
            MockResponse::Fail(msg) => Err(SageError::Tool(msg)),
        }
    }

    /// Apply a mock response for Vec<String>.
    fn apply_mock_vec_string(mock_response: MockResponse) -> SageResult<Vec<String>> {
        match mock_response {
            MockResponse::Value(v) => serde_json::from_value(v)
                .map_err(|e| SageError::Tool(format!("mock deserialize: {e}"))),
            MockResponse::Fail(msg) => Err(SageError::Tool(msg)),
        }
    }
}

impl Default for FsClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filesystem_client_creates() {
        let client = FsClient::new();
        assert_eq!(client.root, PathBuf::from("."));
    }

    #[test]
    fn filesystem_client_with_root() {
        let client = FsClient::with_root(PathBuf::from("/tmp"));
        assert_eq!(client.root, PathBuf::from("/tmp"));
    }

    #[tokio::test]
    async fn filesystem_read_write() {
        let temp_dir = tempfile::tempdir().unwrap();
        let client = FsClient::with_root(temp_dir.path().to_path_buf());

        // Write a file
        client
            .write("test.txt".to_string(), "Hello, World!".to_string())
            .await
            .unwrap();

        // Read it back
        let content = client.read("test.txt".to_string()).await.unwrap();
        assert_eq!(content, "Hello, World!");

        // Check it exists
        assert!(client.exists("test.txt".to_string()).await.unwrap());

        // Delete it
        client.delete("test.txt".to_string()).await.unwrap();

        // Check it's gone
        assert!(!client.exists("test.txt".to_string()).await.unwrap());
    }

    #[tokio::test]
    async fn filesystem_list() {
        let temp_dir = tempfile::tempdir().unwrap();
        let client = FsClient::with_root(temp_dir.path().to_path_buf());

        // Create some files
        client
            .write("a.txt".to_string(), "a".to_string())
            .await
            .unwrap();
        client
            .write("b.txt".to_string(), "b".to_string())
            .await
            .unwrap();

        // List the directory
        let mut files = client.list(".".to_string()).await.unwrap();
        files.sort();
        assert_eq!(files, vec!["a.txt", "b.txt"]);
    }

    #[tokio::test]
    async fn filesystem_write_creates_parents() {
        let temp_dir = tempfile::tempdir().unwrap();
        let client = FsClient::with_root(temp_dir.path().to_path_buf());

        // Write to a nested path
        client
            .write("nested/dir/file.txt".to_string(), "content".to_string())
            .await
            .unwrap();

        // Verify it was created
        assert!(client
            .exists("nested/dir/file.txt".to_string())
            .await
            .unwrap());
    }
}
