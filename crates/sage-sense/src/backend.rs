//! LSP backend implementation.

use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::store::DocumentStore;

/// The LSP backend that handles all requests and notifications.
pub struct Backend {
    /// The LSP client for sending notifications back to the editor.
    client: Client,
    /// Store of open documents.
    store: Arc<DocumentStore>,
}

impl Backend {
    /// Create a new backend with the given client.
    pub fn new(client: Client, store: Arc<DocumentStore>) -> Self {
        Self { client, store }
    }

    /// Handle a file change (open or modify).
    async fn on_file_change(&self, uri: Url, text: String, version: i32) {
        let diagnostics = crate::analysis::analyse(&text);
        self.store
            .update(uri.clone(), text, diagnostics.clone(), version);
        self.client
            .publish_diagnostics(uri, diagnostics, Some(version))
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: crate::capabilities::server_capabilities(),
            server_info: Some(ServerInfo {
                name: "sage-sense".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "sage-sense initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.on_file_change(
            params.text_document.uri,
            params.text_document.text,
            params.text_document.version,
        )
        .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // Full sync: take the last content update
        if let Some(change) = params.content_changes.into_iter().last() {
            self.on_file_change(
                params.text_document.uri,
                change.text,
                params.text_document.version,
            )
            .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.store.remove(&params.text_document.uri);
        // Clear diagnostics for the closed file
        self.client
            .publish_diagnostics(params.text_document.uri, vec![], None)
            .await;
    }

    async fn did_save(&self, _params: DidSaveTextDocumentParams) {
        // We already analyse on change, so nothing to do here
    }
}
