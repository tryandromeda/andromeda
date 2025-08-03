// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::config::ConfigManager;
use crate::lint::lint_file_content_with_config;
use log::{debug, info, warn};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tower_lsp::{Client, LanguageServer, jsonrpc::Result, lsp_types::*};

use super::capabilities::create_server_capabilities;
use super::completions::{AndromedaCompletionProvider, get_console_completions, get_env_completions, create_completion_item};
use super::diagnostic_converter::{lint_error_to_diagnostic, position_to_offset};
use super::options::{Options, RunMode, WorkspaceOptions};

/// Get Canvas 2D context completions
fn get_canvas_context_completions() -> Vec<CompletionItem> {
    vec![
        create_completion_item(
            "fillStyle",
            CompletionItemKind::PROPERTY,
            "fillStyle: string | CanvasGradient",
            "🎨 Sets or returns the color, gradient, or pattern used to fill the drawing.",
            "fillStyle = ${1:color}",
            Some("🎨"),
        ),
        create_completion_item(
            "strokeStyle", 
            CompletionItemKind::PROPERTY,
            "strokeStyle: string",
            "🖌️ Sets or returns the color, gradient, or pattern used for strokes.",
            "strokeStyle = ${1:color}",
            Some("🖌️"),
        ),
        create_completion_item(
            "fillRect",
            CompletionItemKind::FUNCTION,
            "fillRect(x: number, y: number, width: number, height: number): void",
            "🔳 Draws a filled rectangle.",
            "fillRect(${1:x}, ${2:y}, ${3:width}, ${4:height})",
            Some("🔳"),
        ),
        create_completion_item(
            "strokeRect",
            CompletionItemKind::FUNCTION,
            "strokeRect(x: number, y: number, width: number, height: number): void", 
            "▭ Draws a rectangle outline.",
            "strokeRect(${1:x}, ${2:y}, ${3:width}, ${4:height})",
            Some("▭"),
        ),
        create_completion_item(
            "beginPath",
            CompletionItemKind::FUNCTION,
            "beginPath(): void",
            "🎯 Begins a new path.",
            "beginPath()",
            Some("🎯"),
        ),
        create_completion_item(
            "moveTo",
            CompletionItemKind::FUNCTION,
            "moveTo(x: number, y: number): void",
            "📍 Moves the path to the specified point.",
            "moveTo(${1:x}, ${2:y})",
            Some("📍"),
        ),
        create_completion_item(
            "lineTo",
            CompletionItemKind::FUNCTION,
            "lineTo(x: number, y: number): void",
            "📏 Adds a line to the path.",
            "lineTo(${1:x}, ${2:y})",
            Some("📏"),
        ),
        create_completion_item(
            "arc",
            CompletionItemKind::FUNCTION,
            "arc(x: number, y: number, radius: number, startAngle: number, endAngle: number): void",
            "⭕ Adds an arc to the path.",
            "arc(${1:x}, ${2:y}, ${3:radius}, ${4:startAngle}, ${5:endAngle})",
            Some("⭕"),
        ),
        create_completion_item(
            "fill",
            CompletionItemKind::FUNCTION,
            "fill(): void",
            "🎨 Fills the current path.",
            "fill()",
            Some("🎨"),
        ),
        create_completion_item(
            "stroke",
            CompletionItemKind::FUNCTION,
            "stroke(): void",
            "🖌️ Strokes the current path.",
            "stroke()",
            Some("🖌️"),
        ),
    ]
}

/// Get crypto.subtle completions
fn get_crypto_subtle_completions() -> Vec<CompletionItem> {
    vec![
        create_completion_item(
            "digest",
            CompletionItemKind::FUNCTION,
            "digest(algorithm: AlgorithmIdentifier, data: BufferSource): Promise<ArrayBuffer>",
            "🔐 Computes a digest of the given data.",
            "digest(${1:'SHA-256'}, ${2:data})",
            Some("🔐"),
        ),
        create_completion_item(
            "encrypt",
            CompletionItemKind::FUNCTION,
            "encrypt(algorithm: AlgorithmIdentifier, key: CryptoKey, data: BufferSource): Promise<ArrayBuffer>",
            "🔒 Encrypts data using the specified algorithm and key.",
            "encrypt(${1:algorithm}, ${2:key}, ${3:data})",
            Some("🔒"),
        ),
        create_completion_item(
            "decrypt",
            CompletionItemKind::FUNCTION,
            "decrypt(algorithm: AlgorithmIdentifier, key: CryptoKey, data: BufferSource): Promise<ArrayBuffer>",
            "🔓 Decrypts data using the specified algorithm and key.",
            "decrypt(${1:algorithm}, ${2:key}, ${3:data})",
            Some("🔓"),
        ),
        create_completion_item(
            "generateKey",
            CompletionItemKind::FUNCTION,
            "generateKey(algorithm: AlgorithmIdentifier, extractable: boolean, keyUsages: KeyUsage[]): Promise<CryptoKey | CryptoKeyPair>",
            "🔑 Generates a key or key pair.",
            "generateKey(${1:algorithm}, ${2:extractable}, ${3:keyUsages})",
            Some("🔑"),
        ),
        create_completion_item(
            "sign",
            CompletionItemKind::FUNCTION,
            "sign(algorithm: AlgorithmIdentifier, key: CryptoKey, data: BufferSource): Promise<ArrayBuffer>",
            "✍️ Signs data using the specified algorithm and key.",
            "sign(${1:algorithm}, ${2:key}, ${3:data})",
            Some("✍️"),
        ),
        create_completion_item(
            "verify",
            CompletionItemKind::FUNCTION,
            "verify(algorithm: AlgorithmIdentifier, key: CryptoKey, signature: BufferSource, data: BufferSource): Promise<boolean>",
            "✅ Verifies a signature.",
            "verify(${1:algorithm}, ${2:key}, ${3:signature}, ${4:data})",
            Some("✅"),
        ),
    ]
}

/// Get performance API completions
fn get_performance_completions() -> Vec<CompletionItem> {
    vec![
        create_completion_item(
            "now",
            CompletionItemKind::FUNCTION,
            "now(): number",
            "⏱️ Returns a high-resolution timestamp in milliseconds.",
            "now()",
            Some("⏱️"),
        ),
        create_completion_item(
            "mark",
            CompletionItemKind::FUNCTION,
            "mark(name: string, options?: PerformanceMarkOptions): PerformanceMark",
            "📍 Creates a named timestamp in the performance timeline.",
            "mark(${1:name})",
            Some("📍"),
        ),
        create_completion_item(
            "measure",
            CompletionItemKind::FUNCTION,
            "measure(name: string, startOrOptions?: string | PerformanceMeasureOptions, endMark?: string): PerformanceMeasure",
            "📏 Creates a named measurement between two marks.",
            "measure(${1:name}, ${2:start}, ${3:end})",
            Some("📏"),
        ),
        create_completion_item(
            "clearMarks",
            CompletionItemKind::FUNCTION,
            "clearMarks(name?: string): void",
            "🗑️ Removes performance marks from the timeline.",
            "clearMarks(${1:name})",
            Some("🗑️"),
        ),
        create_completion_item(
            "clearMeasures",
            CompletionItemKind::FUNCTION,
            "clearMeasures(name?: string): void",
            "🗑️ Removes performance measures from the timeline.",
            "clearMeasures(${1:name})",
            Some("🗑️"),
        ),
    ]
}

/// Get Web Storage API completions
fn get_storage_completions() -> Vec<CompletionItem> {
    vec![
        create_completion_item(
            "getItem",
            CompletionItemKind::FUNCTION,
            "getItem(key: string): string | null",
            "🔍 Returns the value for the specified key.",
            "getItem(${1:key})",
            Some("🔍"),
        ),
        create_completion_item(
            "setItem",
            CompletionItemKind::FUNCTION,
            "setItem(key: string, value: string): void",
            "💾 Sets the value for the specified key.",
            "setItem(${1:key}, ${2:value})",
            Some("💾"),
        ),
        create_completion_item(
            "removeItem",
            CompletionItemKind::FUNCTION,
            "removeItem(key: string): void",
            "🗑️ Removes the item with the specified key.",
            "removeItem(${1:key})",
            Some("🗑️"),
        ),
        create_completion_item(
            "clear",
            CompletionItemKind::FUNCTION,
            "clear(): void",
            "🗑️ Removes all items from storage.",
            "clear()",
            Some("🗑️"),
        ),
        create_completion_item(
            "key",
            CompletionItemKind::FUNCTION,
            "key(index: number): string | null",
            "🔑 Returns the key at the specified index.",
            "key(${1:index})",
            Some("🔑"),
        ),
        create_completion_item(
            "length",
            CompletionItemKind::PROPERTY,
            "length: number",
            "📊 The number of items in storage.",
            "length",
            Some("📊"),
        ),
    ]
}

/// Get SQLite database completions
fn get_database_completions() -> Vec<CompletionItem> {
    vec![
        create_completion_item(
            "prepare",
            CompletionItemKind::FUNCTION,
            "prepare(sql: string): StatementSync",
            "🗄️ Prepares a SQL statement for execution.",
            "prepare(${1:sql})",
            Some("🗄️"),
        ),
        create_completion_item(
            "exec",
            CompletionItemKind::FUNCTION,
            "exec(sql: string): void",
            "⚡ Executes SQL statements without returning results.",
            "exec(${1:sql})",
            Some("⚡"),
        ),
        create_completion_item(
            "close",
            CompletionItemKind::FUNCTION,
            "close(): void",
            "🔒 Closes the database connection.",
            "close()",
            Some("🔒"),
        ),
        create_completion_item(
            "all",
            CompletionItemKind::FUNCTION,
            "all(...params: any[]): unknown[]",
            "📋 Executes the statement and returns all results.",
            "all(${1:params})",
            Some("📋"),
        ),
        create_completion_item(
            "get",
            CompletionItemKind::FUNCTION,
            "get(...params: any[]): unknown",
            "🎯 Executes the statement and returns the first result.",
            "get(${1:params})",
            Some("🎯"),
        ),
        create_completion_item(
            "run",
            CompletionItemKind::FUNCTION,
            "run(...params: any[]): StatementResultingChanges",
            "🏃 Executes the statement and returns change information.",
            "run(${1:params})",
            Some("🏃"),
        ),
    ]
}

/// Get context-specific completions based on what the user is typing
#[allow(dead_code)]
fn get_context_specific_completions(text_before_cursor: &str) -> Option<Vec<CompletionItem>> {
    if text_before_cursor.contains("ctx.") || text_before_cursor.contains("context.") {
        Some(get_canvas_context_completions())
    } else if text_before_cursor.contains("crypto.subtle.") {
        Some(get_crypto_subtle_completions())
    } else if text_before_cursor.contains("performance.") {
        Some(get_performance_completions())
    } else if text_before_cursor.contains("localStorage.") || text_before_cursor.contains("sessionStorage.") {
        Some(get_storage_completions())
    } else if text_before_cursor.contains("db.") {
        Some(get_database_completions())
    } else {
        None
    }
}

/// Document tracker for maintaining document state
#[derive(Debug, Clone)]
pub struct DocumentInfo {
    pub content: String,
    pub version: i32,
    pub language_id: String,
}

/// Document tracker that maintains document content in memory
pub type DocumentTracker = Arc<RwLock<HashMap<Url, DocumentInfo>>>;

/// Andromeda Language Server Backend
pub struct AndromedaBackend {
    client: Client,
    options: Arc<Mutex<Options>>,
    workspace_folders: Arc<RwLock<Vec<WorkspaceFolder>>>,
    completion_provider: AndromedaCompletionProvider,
    document_tracker: DocumentTracker,
}

impl AndromedaBackend {
    /// Create a new Andromeda Language Server backend
    pub fn new(client: Client) -> Self {
        Self {
            client,
            options: Arc::new(Mutex::new(Options::default())),
            workspace_folders: Arc::new(RwLock::new(Vec::new())),
            completion_provider: AndromedaCompletionProvider::new(),
            document_tracker: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get document content from the tracker
    async fn get_document_content(&self, uri: &Url) -> Option<String> {
        let documents = self.document_tracker.read().await;
        documents.get(uri).map(|doc| doc.content.clone())
    }

    /// Update document content in the tracker
    async fn update_document(&self, uri: Url, content: String, version: i32, language_id: String) {
        let mut documents = self.document_tracker.write().await;
        documents.insert(uri, DocumentInfo {
            content,
            version,
            language_id,
        });
    }

    /// Remove document from the tracker
    async fn remove_document(&self, uri: &Url) {
        let mut documents = self.document_tracker.write().await;
        documents.remove(uri);
    }

    /// Apply text changes to a document
    async fn apply_text_changes(&self, uri: &Url, changes: &[TextDocumentContentChangeEvent]) -> Option<String> {
        let mut documents = self.document_tracker.write().await;
        let document = documents.get_mut(uri)?;
        
        for change in changes {
            if let Some(range) = change.range {
                // Handle range-based changes
                if let Some(start_offset) = self.position_to_offset_sync(&document.content, range.start) {
                    if let Some(end_offset) = self.position_to_offset_sync(&document.content, range.end) {
                        let mut content_chars: Vec<char> = document.content.chars().collect();
                        content_chars.splice(start_offset..end_offset, change.text.chars());
                        document.content = content_chars.into_iter().collect();
                    }
                }
            } else {
                // Full document replacement
                document.content = change.text.clone();
            }
        }
        
        Some(document.content.clone())
    }

    /// Convert position to offset synchronously (helper for text changes)
    fn position_to_offset_sync(&self, content: &str, position: Position) -> Option<usize> {
        let mut current_line = 0;
        let mut current_char = 0;
        let mut offset = 0;

        for ch in content.chars() {
            if current_line == position.line && current_char == position.character {
                return Some(offset);
            }

            if ch == '\n' {
                current_line += 1;
                current_char = 0;
            } else {
                current_char += 1;
            }

            offset += ch.len_utf8();
        }

        if current_line == position.line && current_char == position.character {
            Some(offset)
        } else {
            None
        }
    }

    /// Run diagnostics on a document
    async fn run_diagnostics(&self, uri: Url, content: &str) -> Vec<Diagnostic> {
        debug!("Running diagnostics for: {uri}");

        // Convert URI to file path
        let path = match uri.to_file_path() {
            Ok(path) => path,
            Err(_) => {
                warn!("Failed to convert URI to file path: {uri}");
                return Vec::new();
            }
        };

        // Load configuration for linting
        let config = ConfigManager::load_or_default(None);

        // Run the linter on the content
        match lint_file_content_with_config(&path, content, Some(config)) {
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

    /// Check if a completion should be triggered based on the character before cursor
    #[allow(dead_code)]
    async fn should_trigger_completion(&self, character_before_cursor: Option<char>) -> bool {
        if let Some(ch) = character_before_cursor {
            matches!(ch, '.' | '(' | '<' | '"' | '\'' | ':' | '/')
        } else {
            false
        }
    }

    /// Get context-specific completions based on what the user is typing
    async fn get_context_specific_completions(
        &self,
        text_before_cursor: &str,
        _context: &Option<CompletionContext>,
    ) -> Option<Vec<CompletionItem>> {
        // Check for Andromeda.env completions
        if text_before_cursor.ends_with("Andromeda.env.") {
            return Some(get_env_completions());
        }
        
        // Check for console completions
        if text_before_cursor.ends_with("console.") {
            return Some(get_console_completions());
        }

        // Check for canvas context completions
        if text_before_cursor.contains("getContext(\"2d\")") || text_before_cursor.ends_with("ctx.") {
            return Some(get_canvas_context_completions());
        }

        // Check for crypto.subtle completions
        if text_before_cursor.ends_with("crypto.subtle.") {
            return Some(get_crypto_subtle_completions());
        }

        // Check for performance completions
        if text_before_cursor.ends_with("performance.") {
            return Some(get_performance_completions());
        }

        // Check for localStorage/sessionStorage completions
        if text_before_cursor.ends_with("localStorage.") || text_before_cursor.ends_with("sessionStorage.") {
            return Some(get_storage_completions());
        }

        // Check for database completions (SQLite)
        if text_before_cursor.contains("new Database(") || text_before_cursor.ends_with("db.") {
            return Some(get_database_completions());
        }

        None
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
        info!("Andromeda Language Server shutting down");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        debug!("Document opened: {}", params.text_document.uri);

        // Track the document
        self.update_document(
            params.text_document.uri.clone(),
            params.text_document.text.clone(),
            params.text_document.version,
            params.text_document.language_id.clone(),
        ).await;

        if self.should_run_diagnostics(DiagnosticTrigger::Open).await {
            let diagnostics = self
                .run_diagnostics(params.text_document.uri.clone(), &params.text_document.text)
                .await;
            self.publish_diagnostics(params.text_document.uri, diagnostics)
                .await;
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        debug!("Document changed: {}", params.text_document.uri);

        // Apply text changes to the tracked document
        if let Some(updated_content) = self.apply_text_changes(&params.text_document.uri, &params.content_changes).await {
            if self.should_run_diagnostics(DiagnosticTrigger::Change).await {
                let diagnostics = self
                    .run_diagnostics(params.text_document.uri.clone(), &updated_content)
                    .await;
                self.publish_diagnostics(params.text_document.uri, diagnostics)
                    .await;
            }
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        debug!("Document saved: {}", params.text_document.uri);

        if self.should_run_diagnostics(DiagnosticTrigger::Save).await {
            // Use tracked content or provided text
            let content = if let Some(text) = params.text {
                // Update tracker with saved content if provided
                if let Some(doc_info) = self.document_tracker.read().await.get(&params.text_document.uri) {
                    self.update_document(
                        params.text_document.uri.clone(),
                        text.clone(),
                        doc_info.version + 1,
                        doc_info.language_id.clone(),
                    ).await;
                }
                text
            } else {
                // Use tracked content
                self.get_document_content(&params.text_document.uri).await.unwrap_or_default()
            };

            let diagnostics = self
                .run_diagnostics(params.text_document.uri.clone(), &content)
                .await;
            self.publish_diagnostics(params.text_document.uri, diagnostics)
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        debug!("Document closed: {}", params.text_document.uri);
        
        // Remove document from tracker
        self.remove_document(&params.text_document.uri).await;
        
        // Clear diagnostics for closed document
        self.publish_diagnostics(params.text_document.uri, Vec::new())
            .await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        debug!("Completion request at: {:?}", params.text_document_position);

        let mut completions = Vec::new();

        // Get document content from the tracker
        let content = self.get_document_content(&params.text_document_position.text_document.uri)
            .await
            .unwrap_or_default();
        
        let offset = position_to_offset(params.text_document_position.position, &content);

        // Get text before cursor for context detection
        let text_before_cursor = content.chars().take(offset).collect::<String>();

        debug!("Text before cursor: '{}'", text_before_cursor.chars().rev().take(50).collect::<String>().chars().rev().collect::<String>());

        // Get context-specific completions
        if let Some(context_completions) = self.get_context_specific_completions(
            &text_before_cursor,
            &params.context,
        ).await {
            completions.extend(context_completions);
        }
        
        // Add general Andromeda API completions
        let context_ref = params.context.as_ref();
        let general_completions = self.completion_provider.get_completions(
            context_ref,
            &content,
            offset,
        );
        completions.extend(general_completions);

        debug!("Returning {} completions", completions.len());

        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        debug!("Hover request at: {:?}", params.text_document_position_params);

        // TODO: Implement hover information
        // This could provide:
        // - Function signatures
        // - Documentation
        // - Type information
        // - Examples

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

        // TODO: Implement formatting
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
