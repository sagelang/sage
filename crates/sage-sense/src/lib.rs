//! Language Server Protocol implementation for the Sage language.
//!
//! This crate provides the `sage sense` LSP server, which enables editor
//! integration with syntax highlighting, diagnostics, and more.
//!
//! # Example
//!
//! The server is typically started via the CLI:
//!
//! ```bash
//! sage sense
//! ```
//!
//! The server communicates over stdin/stdout using the LSP JSON-RPC protocol.

#![forbid(unsafe_code)]

mod analysis;
mod backend;
mod capabilities;
mod convert;
mod store;

use std::sync::Arc;
use tower_lsp::{LspService, Server};

use backend::Backend;
use store::DocumentStore;

/// Start the LSP server, reading from stdin and writing to stdout.
/// Blocks until the client sends `shutdown` followed by `exit`.
pub async fn run() -> anyhow::Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let store = Arc::new(DocumentStore::new());
    let (service, socket) = LspService::new(|client| Backend::new(client, Arc::clone(&store)));

    Server::new(stdin, stdout, socket).serve(service).await;
    Ok(())
}
