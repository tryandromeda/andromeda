// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::lint::lint_file_content;
use futures::future::join_all;
use log::{debug, info, warn};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tower_lsp::{Client, LanguageServer, jsonrpc::Result, lsp_types::*};

use super::capabilities::create_server_capabilities;
use super::diagnostic_converter::lint_error_to_diagnostic;
use super::options::{Options, RunMode, WorkspaceOptions};

/// The main Andromeda Language Server backend
pub struct AndromedaBackend {
    client: Client,
    workspace_folders: Arc<RwLock<Vec<WorkspaceFolder>>>,
    document_map: Arc<RwLock<HashMap<Url, String>>>,
    options: Arc<Mutex<Options>>,
}

impl AndromedaBackend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            workspace_folders: Arc::new(RwLock::new(Vec::new())),
            document_map: Arc::new(RwLock::new(HashMap::new())),
            options: Arc::new(Mutex::new(Options::default())),
        }
    }

    /// Check if a URI is a JavaScript or TypeScript file
    pub fn is_supported_file(uri: &Url) -> bool {
        if let Ok(path) = uri.to_file_path() {
            if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                matches!(extension, "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs")
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Run lint diagnostics on a document
    async fn lint_document(&self, uri: &Url, content: &str) -> Vec<Diagnostic> {
        if !Self::is_supported_file(uri) {
            return Vec::new();
        }

        // Convert URI to PathBuf for linting
        let path = match uri.to_file_path() {
            Ok(path) => path,
            Err(_) => {
                warn!("Failed to convert URI to file path: {uri}");
                return Vec::new();
            }
        };

        // Run the linter on the content
        match lint_file_content(&path, content) {
            Ok(lint_errors) => lint_errors
                .iter()
                .map(|error| lint_error_to_diagnostic(error, content))
                .collect(),
            Err(e) => {
                warn!("Linting failed for {uri}: {e}");
                Vec::new()
            }
        }
    }

    /// Publish diagnostics for a document
    async fn publish_diagnostics(&self, uri: Url, diagnostics: Vec<Diagnostic>) {
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    /// Check if we should run diagnostics based on the current run mode and trigger
    async fn should_run_diagnostics(&self, trigger: DiagnosticTrigger) -> bool {
        let options = self.options.lock().await;
        match options.run {
            RunMode::OnType => true,
            RunMode::OnSave => matches!(trigger, DiagnosticTrigger::Save),
            RunMode::Never => false,
        }
    }
}

/// When diagnostics should be triggered
#[derive(Debug, Clone, Copy)]
enum DiagnosticTrigger {
    Open,
    Change,
    Save,
}

#[tower_lsp::async_trait]
impl LanguageServer for AndromedaBackend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        info!("Initializing Andromeda Language Server");

        // Store workspace folders
        if let Some(workspace_folders) = params.workspace_folders {
            *self.workspace_folders.write().await = workspace_folders;
        } else if let Some(root_uri) = params.root_uri {
            // Legacy single root workspace
            let name = root_uri
                .to_file_path()
                .ok()
                .and_then(|path| {
                    path.file_name()
                        .map(|n| n.to_str().unwrap_or("root").to_string())
                })
                .unwrap_or_else(|| "root".to_string());

            let workspace_folder = WorkspaceFolder {
                uri: root_uri.clone(),
                name,
            };
            self.workspace_folders.write().await.push(workspace_folder);
        }

        // Handle initialization options
        if let Some(init_options) = params.initialization_options {
            if let Ok(workspace_options) =
                serde_json::from_value::<Vec<WorkspaceOptions>>(init_options.clone())
            {
                // Use first workspace options as default for now
                if let Some(first_workspace) = workspace_options.first() {
                    *self.options.lock().await = first_workspace.options.clone();
                }
            } else if let Ok(options) = serde_json::from_value::<Options>(init_options) {
                *self.options.lock().await = options;
            }
        }

        info!("Andromeda Language Server initialized");

        Ok(InitializeResult {
            capabilities: create_server_capabilities(),
            server_info: Some(ServerInfo {
                name: "andromeda-language-server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        info!("Andromeda Language Server initialized and ready to serve");
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Shutting down Andromeda Language Server");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        debug!("Document opened: {}", params.text_document.uri);

        let uri = params.text_document.uri;
        let content = params.text_document.text.clone();

        // Store document content
        self.document_map
            .write()
            .await
            .insert(uri.clone(), content.clone());

        // Run diagnostics if configured to do so
        if self.should_run_diagnostics(DiagnosticTrigger::Open).await {
            let diagnostics = self.lint_document(&uri, &content).await;
            self.publish_diagnostics(uri, diagnostics).await;
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        debug!("Document changed: {}", params.text_document.uri);

        let uri = params.text_document.uri;

        // Update document content (we're using FULL sync, so there's only one change)
        if let Some(change) = params.content_changes.into_iter().next() {
            let content = change.text;

            self.document_map
                .write()
                .await
                .insert(uri.clone(), content.clone());

            // Run diagnostics if configured to do so
            if self.should_run_diagnostics(DiagnosticTrigger::Change).await {
                let diagnostics = self.lint_document(&uri, &content).await;
                self.publish_diagnostics(uri, diagnostics).await;
            }
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        debug!("Document saved: {}", params.text_document.uri);

        let uri = params.text_document.uri;

        // Update content if provided
        if let Some(text) = params.text {
            self.document_map
                .write()
                .await
                .insert(uri.clone(), text.clone());
        }

        // Run diagnostics if configured to do so
        if self.should_run_diagnostics(DiagnosticTrigger::Save).await {
            let document_map = self.document_map.read().await;
            if let Some(content) = document_map.get(&uri) {
                let diagnostics = self.lint_document(&uri, content).await;
                self.publish_diagnostics(uri, diagnostics).await;
            }
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        debug!("Document closed: {}", params.text_document.uri);

        // Remove document from map
        self.document_map
            .write()
            .await
            .remove(&params.text_document.uri);

        // Clear diagnostics
        self.publish_diagnostics(params.text_document.uri, Vec::new())
            .await;
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        debug!("Configuration changed");

        // Try to parse new configuration
        if let Ok(options) = serde_json::from_value::<Options>(params.settings) {
            *self.options.lock().await = options;

            // Re-run diagnostics on all open documents
            let document_map = self.document_map.read().await;
            let futures = document_map.iter().map(|(uri, content)| async {
                let diagnostics = self.lint_document(uri, content).await;
                self.publish_diagnostics(uri.clone(), diagnostics).await;
            });

            join_all(futures).await;
        }
    }

    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        debug!("Workspace folders changed");

        let mut workspace_folders = self.workspace_folders.write().await;

        // Remove folders
        for removed in params.event.removed {
            workspace_folders.retain(|folder| folder.uri != removed.uri);
        }

        // Add new folders
        workspace_folders.extend(params.event.added);
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        debug!(
            "Hover request for: {}",
            params.text_document_position_params.text_document.uri
        );

        // TODO: Implement hover information
        // This could show documentation for built-in APIs, variable types, etc.
        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        debug!(
            "Completion request for: {}",
            params.text_document_position.text_document.uri
        );

        // TODO: Implement completion
        // This could provide completions for:
        // - Built-in JavaScript/TypeScript APIs
        // - Andromeda-specific APIs
        // - Imported modules
        Ok(None)
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        debug!("Code action request for: {}", params.text_document.uri);

        // TODO: Implement code actions
        // This could provide:
        // - Quick fixes for lint errors
        // - Refactoring suggestions
        // - Auto-imports
        Ok(None)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        debug!("Formatting request for: {}", params.text_document.uri);

        // TODO: Implement formatting using dprint or similar
        Ok(None)
    }

    async fn range_formatting(
        &self,
        params: DocumentRangeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        debug!("Range formatting request for: {}", params.text_document.uri);

        // TODO: Implement range formatting
        Ok(None)
    }

    async fn on_type_formatting(
        &self,
        params: DocumentOnTypeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        debug!(
            "On-type formatting request for: {}",
            params.text_document_position.text_document.uri
        );

        // TODO: Implement on-type formatting
        Ok(None)
    }

    async fn execute_command(
        &self,
        params: ExecuteCommandParams,
    ) -> Result<Option<serde_json::Value>> {
        debug!("Execute command: {}", params.command);

        match params.command.as_str() {
            "andromeda.applyAutoFix" => {
                // TODO: Implement auto-fix command
                Ok(None)
            }
            "andromeda.fixAll" => {
                // TODO: Implement fix-all command
                Ok(None)
            }
            _ => {
                warn!("Unknown command: {}", params.command);
                Ok(None)
            }
        }
    }
}
