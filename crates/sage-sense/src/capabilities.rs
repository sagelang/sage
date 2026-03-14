//! Server capability declarations.

use tower_lsp::lsp_types::{ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind};

/// Return the server's capabilities.
pub fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        // Full document sync: client sends the entire file on every change.
        // Incremental sync is a future optimisation.
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::FULL,
        )),
        // All other capabilities disabled for Phase 1
        ..Default::default()
    }
}
