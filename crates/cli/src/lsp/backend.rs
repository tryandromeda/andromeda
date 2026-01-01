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
use super::completions::{
    AndromedaCompletionProvider, create_completion_item, get_console_completions,
    get_env_completions,
};
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
    } else if text_before_cursor.contains("localStorage.")
        || text_before_cursor.contains("sessionStorage.")
    {
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
        documents.insert(
            uri,
            DocumentInfo {
                content,
                version,
                language_id,
            },
        );
    }

    /// Remove document from the tracker
    async fn remove_document(&self, uri: &Url) {
        let mut documents = self.document_tracker.write().await;
        documents.remove(uri);
    }

    /// Apply text changes to a document
    async fn apply_text_changes(
        &self,
        uri: &Url,
        changes: &[TextDocumentContentChangeEvent],
    ) -> Option<String> {
        let mut documents = self.document_tracker.write().await;
        let document = documents.get_mut(uri)?;

        for change in changes {
            if let Some(range) = change.range {
                // Handle range-based changes
                if let Some(start_offset) =
                    self.position_to_offset_sync(&document.content, range.start)
                    && let Some(end_offset) =
                        self.position_to_offset_sync(&document.content, range.end)
                {
                    let mut content_chars: Vec<char> = document.content.chars().collect();
                    content_chars.splice(start_offset..end_offset, change.text.chars());
                    document.content = content_chars.into_iter().collect();
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
        if text_before_cursor.contains("getContext(\"2d\")") || text_before_cursor.ends_with("ctx.")
        {
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
        if text_before_cursor.ends_with("localStorage.")
            || text_before_cursor.ends_with("sessionStorage.")
        {
            return Some(get_storage_completions());
        }

        // Check for database completions (SQLite)
        if text_before_cursor.contains("new Database(") || text_before_cursor.ends_with("db.") {
            return Some(get_database_completions());
        }

        None
    }

    /// Get word at position for hover information
    fn get_word_at_position(&self, content: &str, offset: usize) -> Option<String> {
        let chars: Vec<char> = content.chars().collect();
        if offset >= chars.len() {
            return None;
        }

        // Find word boundaries
        let mut start = offset;
        let mut end = offset;

        // Move start backwards to find word start
        while start > 0
            && (chars[start - 1].is_alphanumeric()
                || chars[start - 1] == '_'
                || chars[start - 1] == '.')
        {
            start -= 1;
        }

        // Move end forwards to find word end
        while end < chars.len()
            && (chars[end].is_alphanumeric() || chars[end] == '_' || chars[end] == '.')
        {
            end += 1;
        }

        if start < end {
            Some(chars[start..end].iter().collect())
        } else {
            None
        }
    }

    /// Get hover information for Andromeda APIs
    async fn get_andromeda_api_hover(&self, word: &str) -> Option<Hover> {
        let hover_info = match word {
            "Andromeda" => Some((
                "**Andromeda Runtime**",
                "The Andromeda runtime provides built-in APIs for file system operations, environment variables, process management, and more.",
            )),
            "Andromeda.readTextFileSync" => Some((
                "**Andromeda.readTextFileSync(path: string): string**",
                "Reads a text file from the file system synchronously.\n\n**Parameters:**\n- `path`: The file path to read\n\n**Returns:** The file content as a string",
            )),
            "Andromeda.writeTextFileSync" => Some((
                "**Andromeda.writeTextFileSync(path: string, data: string): void**",
                "Writes a text file to the file system synchronously.\n\n**Parameters:**\n- `path`: The file path to write\n- `data`: The content to write",
            )),
            "Andromeda.sleep" => Some((
                "**Andromeda.sleep(duration: number): Promise<void>**",
                "Returns a Promise that resolves after the specified duration in milliseconds.\n\n**Parameters:**\n- `duration`: Sleep duration in milliseconds",
            )),
            "Andromeda.env" => Some((
                "**Andromeda.env**",
                "Environment variable operations.\n\n**Methods:**\n- `get(key: string): string` - Get environment variable\n- `set(key: string, value: string): void` - Set environment variable\n- `keys(): string[]` - Get all environment variable keys",
            )),
            "Andromeda.args" => Some((
                "**Andromeda.args: string[]**",
                "Command-line arguments passed to the program.\n\n**Type:** Array of strings containing the command-line arguments",
            )),
            "Andromeda.exit" => Some((
                "**Andromeda.exit(code?: number): void**",
                "Exits the program with an optional exit code.\n\n**Parameters:**\n- `code`: Optional exit code (default: 0)",
            )),
            _ => None,
        };

        if let Some((title, description)) = hover_info {
            Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("{title}\n\n{description}"),
                }),
                range: None,
            })
        } else {
            None
        }
    }

    /// Get hover information for Web APIs
    async fn get_web_api_hover(&self, word: &str) -> Option<Hover> {
        let hover_info = match word {
            "console" => Some((
                "**console: Console**",
                "The Console API provides access to the debugging console.\n\n**Methods:**\n- `log()` - Output messages\n- `error()` - Output error messages\n- `warn()` - Output warning messages\n- `info()` - Output info messages",
            )),
            "fetch" => Some((
                "**fetch(input: RequestInfo, init?: RequestInit): Promise<Response>**",
                "Fetch API for making HTTP requests.\n\n**Parameters:**\n- `input`: URL or Request object\n- `init`: Optional request configuration\n\n**Returns:** Promise that resolves to Response object",
            )),
            "performance" => Some((
                "**performance: AndromedaPerformance**",
                "High-resolution time measurements and performance monitoring.\n\n**Methods:**\n- `now()` - Get current high-resolution timestamp\n- `mark()` - Create performance mark\n- `measure()` - Measure between marks",
            )),
            "localStorage" => Some((
                "**localStorage: Storage**",
                "Local storage for persistent data.\n\n**Methods:**\n- `getItem(key)` - Get stored value\n- `setItem(key, value)` - Store value\n- `removeItem(key)` - Remove value\n- `clear()` - Clear all data",
            )),
            "sessionStorage" => Some((
                "**sessionStorage: Storage**",
                "Session storage for temporary data that persists only for the session.\n\n**Methods:**\n- `getItem(key)` - Get stored value\n- `setItem(key, value)` - Store value\n- `removeItem(key)` - Remove value\n- `clear()` - Clear all data",
            )),
            "crypto" => Some((
                "**crypto: Crypto**",
                "Web Crypto API for cryptographic operations.\n\n**Properties:**\n- `subtle` - SubtleCrypto interface for advanced crypto\n\n**Methods:**\n- `randomUUID()` - Generate random UUID\n- `getRandomValues()` - Fill array with random values",
            )),
            _ => None,
        };

        if let Some((title, description)) = hover_info {
            Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("{title}\n\n{description}"),
                }),
                range: None,
            })
        } else {
            None
        }
    }

    /// Get context-specific hover information
    async fn get_context_specific_hover(&self, word: &str, text_before: &str) -> Option<Hover> {
        // Canvas context methods
        if text_before.contains("getContext(\"2d\")") || text_before.contains("ctx.") {
            match word {
                "fillRect" => Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: "**fillRect(x: number, y: number, width: number, height: number): void**\n\nDraws a filled rectangle.\n\n**Parameters:**\n- `x`: X coordinate\n- `y`: Y coordinate\n- `width`: Rectangle width\n- `height`: Rectangle height".to_string(),
                    }),
                    range: None,
                }),
                "strokeRect" => Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: "**strokeRect(x: number, y: number, width: number, height: number): void**\n\nDraws a rectangle outline.\n\n**Parameters:**\n- `x`: X coordinate\n- `y`: Y coordinate\n- `width`: Rectangle width\n- `height`: Rectangle height".to_string(),
                    }),
                    range: None,
                }),
                "fillStyle" => Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: "**fillStyle: string | CanvasGradient**\n\nSets or returns the color, gradient, or pattern used to fill the drawing.\n\n**Examples:**\n- `\"red\"`\n- `\"#FF0000\"`\n- `\"rgb(255, 0, 0)\"`".to_string(),
                    }),
                    range: None,
                }),
                _ => None,
            }
        }
        // Database methods
        else if text_before.contains("new Database(") || text_before.contains("db.") {
            match word {
                "prepare" => Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: "**prepare(sql: string): StatementSync**\n\nPrepares a SQL statement for execution.\n\n**Parameters:**\n- `sql`: SQL statement to prepare\n\n**Returns:** Prepared statement object".to_string(),
                    }),
                    range: None,
                }),
                "exec" => Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: "**exec(sql: string): void**\n\nExecutes SQL statements without returning results.\n\n**Parameters:**\n- `sql`: SQL statement to execute".to_string(),
                    }),
                    range: None,
                }),
                _ => None,
            }
        }
        // Performance methods
        else if text_before.contains("performance.") {
            match word {
                "now" => Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: "**now(): number**\n\nReturns a high-resolution timestamp in milliseconds.\n\n**Returns:** Current time in milliseconds with sub-millisecond precision".to_string(),
                    }),
                    range: None,
                }),
                "mark" => Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: "**mark(name: string, options?: PerformanceMarkOptions): PerformanceMark**\n\nCreates a named timestamp in the performance timeline.\n\n**Parameters:**\n- `name`: Name for the performance mark".to_string(),
                    }),
                    range: None,
                }),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Get signature help information for function calls
    async fn get_signature_help_info(&self, text_before: &str) -> Option<SignatureHelp> {
        // Look for Andromeda API function calls
        if let Some(captures) = regex::Regex::new(r"Andromeda\.(\w+)\s*\($")
            .ok()?
            .captures(text_before)
        {
            let function_name = captures.get(1)?.as_str();
            return self.get_andromeda_signature_help(function_name).await;
        }

        // Look for Web API function calls
        if let Some(captures) = regex::Regex::new(r"(\w+)\s*\($")
            .ok()?
            .captures(text_before)
        {
            let function_name = captures.get(1)?.as_str();
            return self.get_web_api_signature_help(function_name).await;
        }

        None
    }

    /// Get signature help for Andromeda APIs
    async fn get_andromeda_signature_help(&self, function_name: &str) -> Option<SignatureHelp> {
        let signature_info = match function_name {
            "readTextFileSync" => Some((
                "readTextFileSync(path: string): string",
                "Reads a text file from the file system synchronously.",
                vec![ParameterInformation {
                    label: ParameterLabel::Simple("path: string".to_string()),
                    documentation: Some(Documentation::String("The file path to read".to_string())),
                }],
            )),
            "writeTextFileSync" => Some((
                "writeTextFileSync(path: string, data: string): void",
                "Writes a text file to the file system synchronously.",
                vec![
                    ParameterInformation {
                        label: ParameterLabel::Simple("path: string".to_string()),
                        documentation: Some(Documentation::String(
                            "The file path to write".to_string(),
                        )),
                    },
                    ParameterInformation {
                        label: ParameterLabel::Simple("data: string".to_string()),
                        documentation: Some(Documentation::String(
                            "The content to write".to_string(),
                        )),
                    },
                ],
            )),
            "sleep" => Some((
                "sleep(duration: number): Promise<void>",
                "Returns a Promise that resolves after the specified duration in milliseconds.",
                vec![ParameterInformation {
                    label: ParameterLabel::Simple("duration: number".to_string()),
                    documentation: Some(Documentation::String(
                        "Sleep duration in milliseconds".to_string(),
                    )),
                }],
            )),
            _ => None,
        };

        if let Some((label, documentation, parameters)) = signature_info {
            Some(SignatureHelp {
                signatures: vec![SignatureInformation {
                    label: label.to_string(),
                    documentation: Some(Documentation::String(documentation.to_string())),
                    parameters: Some(parameters),
                    active_parameter: None,
                }],
                active_signature: Some(0),
                active_parameter: Some(0),
            })
        } else {
            None
        }
    }

    /// Get signature help for Web APIs
    async fn get_web_api_signature_help(&self, function_name: &str) -> Option<SignatureHelp> {
        let signature_info = match function_name {
            "fetch" => Some((
                "fetch(input: RequestInfo, init?: RequestInit): Promise<Response>",
                "Fetch API for making HTTP requests.",
                vec![
                    ParameterInformation {
                        label: ParameterLabel::Simple("input: RequestInfo".to_string()),
                        documentation: Some(Documentation::String(
                            "URL or Request object".to_string(),
                        )),
                    },
                    ParameterInformation {
                        label: ParameterLabel::Simple("init?: RequestInit".to_string()),
                        documentation: Some(Documentation::String(
                            "Optional request configuration".to_string(),
                        )),
                    },
                ],
            )),
            "setTimeout" => Some((
                "setTimeout(callback: () => void, delay?: number): number",
                "Executes a function after a delay.",
                vec![
                    ParameterInformation {
                        label: ParameterLabel::Simple("callback: () => void".to_string()),
                        documentation: Some(Documentation::String(
                            "Function to execute".to_string(),
                        )),
                    },
                    ParameterInformation {
                        label: ParameterLabel::Simple("delay?: number".to_string()),
                        documentation: Some(Documentation::String(
                            "Delay in milliseconds".to_string(),
                        )),
                    },
                ],
            )),
            _ => None,
        };

        if let Some((label, documentation, parameters)) = signature_info {
            Some(SignatureHelp {
                signatures: vec![SignatureInformation {
                    label: label.to_string(),
                    documentation: Some(Documentation::String(documentation.to_string())),
                    parameters: Some(parameters),
                    active_parameter: None,
                }],
                active_signature: Some(0),
                active_parameter: Some(0),
            })
        } else {
            None
        }
    }

    /// Create code actions for a specific diagnostic
    async fn create_code_actions_for_diagnostic(
        &self,
        uri: &Url,
        diagnostic: &Diagnostic,
        content: &str,
    ) -> Option<Vec<CodeActionOrCommand>> {
        let mut actions = Vec::new();

        if let Some(NumberOrString::String(code_str)) = &diagnostic.code {
            match code_str.as_str() {
                "andromeda::lint::no-var" => {
                    // Quick fix: Replace 'var' with 'let'
                    if let Some(edit) = self.create_var_to_let_fix(uri, diagnostic, content) {
                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: "Replace 'var' with 'let'".to_string(),
                            kind: Some(CodeActionKind::QUICKFIX),
                            diagnostics: Some(vec![diagnostic.clone()]),
                            edit: Some(edit),
                            ..Default::default()
                        }));
                    }
                }
                "andromeda::lint::prefer-const" => {
                    // Quick fix: Replace 'let' with 'const'
                    if let Some(edit) = self.create_let_to_const_fix(uri, diagnostic, content) {
                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: "Replace 'let' with 'const'".to_string(),
                            kind: Some(CodeActionKind::QUICKFIX),
                            diagnostics: Some(vec![diagnostic.clone()]),
                            edit: Some(edit),
                            ..Default::default()
                        }));
                    }
                }
                "andromeda::lint::no-console" => {
                    // Quick fix: Remove console statement
                    if let Some(edit) = self.create_remove_console_fix(uri, diagnostic, content) {
                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: "Remove console statement".to_string(),
                            kind: Some(CodeActionKind::QUICKFIX),
                            diagnostics: Some(vec![diagnostic.clone()]),
                            edit: Some(edit),
                            ..Default::default()
                        }));
                    }
                }
                "andromeda::lint::no-empty" => {
                    // Quick fix: Remove empty statement
                    if let Some(edit) =
                        self.create_remove_empty_statement_fix(uri, diagnostic, content)
                    {
                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: "Remove empty statement".to_string(),
                            kind: Some(CodeActionKind::QUICKFIX),
                            diagnostics: Some(vec![diagnostic.clone()]),
                            edit: Some(edit),
                            ..Default::default()
                        }));
                    }
                }
                _ => {}
            }
        }

        if actions.is_empty() {
            None
        } else {
            Some(actions)
        }
    }

    /// Get general code actions not related to specific diagnostics
    async fn get_general_code_actions(
        &self,
        uri: &Url,
        _range: &Range,
    ) -> Vec<CodeActionOrCommand> {
        vec![CodeActionOrCommand::CodeAction(CodeAction {
            title: "Fix all auto-fixable problems".to_string(),
            kind: Some(CodeActionKind::SOURCE_FIX_ALL),
            diagnostics: None,
            command: Some(Command {
                title: "Fix all".to_string(),
                command: "andromeda.fixAll".to_string(),
                arguments: Some(vec![serde_json::to_value(uri).unwrap()]),
            }),
            ..Default::default()
        })]
    }

    /// Create a workspace edit to replace 'var' with 'let'
    fn create_var_to_let_fix(
        &self,
        uri: &Url,
        diagnostic: &Diagnostic,
        content: &str,
    ) -> Option<WorkspaceEdit> {
        let range = diagnostic.range;
        let line_text = self.get_line_text(content, range.start.line as usize)?;

        if let Some(var_pos) = line_text.find("var ") {
            let edit_range = Range {
                start: Position {
                    line: range.start.line,
                    character: var_pos as u32,
                },
                end: Position {
                    line: range.start.line,
                    character: (var_pos + 3) as u32,
                },
            };

            let text_edit = TextEdit {
                range: edit_range,
                new_text: "let".to_string(),
            };

            let mut changes = std::collections::HashMap::new();
            changes.insert(uri.clone(), vec![text_edit]);

            return Some(WorkspaceEdit {
                changes: Some(changes),
                ..Default::default()
            });
        }

        None
    }

    /// Create a workspace edit to replace 'let' with 'const'
    fn create_let_to_const_fix(
        &self,
        uri: &Url,
        diagnostic: &Diagnostic,
        content: &str,
    ) -> Option<WorkspaceEdit> {
        let range = diagnostic.range;
        let line_text = self.get_line_text(content, range.start.line as usize)?;

        if let Some(let_pos) = line_text.find("let ") {
            let edit_range = Range {
                start: Position {
                    line: range.start.line,
                    character: let_pos as u32,
                },
                end: Position {
                    line: range.start.line,
                    character: (let_pos + 3) as u32,
                },
            };

            let text_edit = TextEdit {
                range: edit_range,
                new_text: "const".to_string(),
            };

            let mut changes = std::collections::HashMap::new();
            changes.insert(uri.clone(), vec![text_edit]);

            return Some(WorkspaceEdit {
                changes: Some(changes),
                ..Default::default()
            });
        }

        None
    }

    /// Create a workspace edit to remove console statement
    fn create_remove_console_fix(
        &self,
        uri: &Url,
        diagnostic: &Diagnostic,
        _content: &str,
    ) -> Option<WorkspaceEdit> {
        let range = diagnostic.range;

        // Find the entire console statement line
        let edit_range = Range {
            start: Position {
                line: range.start.line,
                character: 0,
            },
            end: Position {
                line: range.start.line + 1,
                character: 0,
            },
        };

        let text_edit = TextEdit {
            range: edit_range,
            new_text: String::new(),
        };

        let mut changes = std::collections::HashMap::new();
        changes.insert(uri.clone(), vec![text_edit]);

        Some(WorkspaceEdit {
            changes: Some(changes),
            ..Default::default()
        })
    }

    /// Create a workspace edit to remove empty statement
    fn create_remove_empty_statement_fix(
        &self,
        uri: &Url,
        diagnostic: &Diagnostic,
        _content: &str,
    ) -> Option<WorkspaceEdit> {
        let range = diagnostic.range;

        let text_edit = TextEdit {
            range,
            new_text: String::new(),
        };

        let mut changes = std::collections::HashMap::new();
        changes.insert(uri.clone(), vec![text_edit]);

        Some(WorkspaceEdit {
            changes: Some(changes),
            ..Default::default()
        })
    }

    /// Get the text of a specific line
    fn get_line_text(&self, content: &str, line_number: usize) -> Option<String> {
        content.lines().nth(line_number).map(|s| s.to_string())
    }

    /// Format document using dprint
    async fn format_document(
        &self,
        content: &str,
        _options: &FormattingOptions,
    ) -> anyhow::Result<String> {
        use dprint_core::configuration::{ConfigKeyValue, GlobalConfiguration, NewLineKind};
        use dprint_plugin_typescript::{
            FormatTextOptions as TsFormatTextOptions, configuration as ts_config,
            format_text as ts_format,
        };
        use indexmap::IndexMap;

        // Create basic dprint configuration
        let mut config = IndexMap::new();
        config.insert("indentWidth".to_string(), ConfigKeyValue::from_i32(2));
        config.insert("lineWidth".to_string(), ConfigKeyValue::from_i32(100));
        config.insert("useTabs".to_string(), ConfigKeyValue::from_bool(false));
        config.insert("semiColons".to_string(), ConfigKeyValue::from_str("prefer"));
        config.insert(
            "quoteStyle".to_string(),
            ConfigKeyValue::from_str("alwaysDouble"),
        );
        config.insert(
            "trailingCommas".to_string(),
            ConfigKeyValue::from_str("onlyMultiLine"),
        );
        config.insert("newLineKind".to_string(), ConfigKeyValue::from_str("lf"));

        let global_config = GlobalConfiguration {
            line_width: Some(100),
            use_tabs: Some(false),
            indent_width: Some(2),
            new_line_kind: Some(NewLineKind::LineFeed),
        };

        let resolved_config = ts_config::resolve_config(config, &global_config);

        if !resolved_config.diagnostics.is_empty() {
            let error_msg = resolved_config
                .diagnostics
                .iter()
                .map(|d| format!("{d}"))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(anyhow::anyhow!(
                "Failed to resolve dprint TS configuration: {error_msg}"
            ));
        }

        let config = resolved_config.config;
        let format_options = TsFormatTextOptions {
            path: std::path::Path::new("file.ts"),
            extension: Some("ts"),
            text: content.to_string(),
            external_formatter: None,
            config: &config,
        };

        match ts_format(format_options) {
            Ok(Some(formatted)) => Ok(formatted),
            Ok(None) => Ok(content.to_string()), // No changes needed
            Err(e) => Err(anyhow::anyhow!("Formatting error: {}", e)),
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
        )
        .await;

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
        if let Some(updated_content) = self
            .apply_text_changes(&params.text_document.uri, &params.content_changes)
            .await
            && self.should_run_diagnostics(DiagnosticTrigger::Change).await
        {
            let diagnostics = self
                .run_diagnostics(params.text_document.uri.clone(), &updated_content)
                .await;
            self.publish_diagnostics(params.text_document.uri, diagnostics)
                .await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        debug!("Document saved: {}", params.text_document.uri);

        if self.should_run_diagnostics(DiagnosticTrigger::Save).await {
            // Use tracked content or provided text
            let content = if let Some(text) = params.text {
                // Update tracker with saved content if provided
                if let Some(doc_info) = self
                    .document_tracker
                    .read()
                    .await
                    .get(&params.text_document.uri)
                {
                    self.update_document(
                        params.text_document.uri.clone(),
                        text.clone(),
                        doc_info.version + 1,
                        doc_info.language_id.clone(),
                    )
                    .await;
                }
                text
            } else {
                // Use tracked content
                self.get_document_content(&params.text_document.uri)
                    .await
                    .unwrap_or_default()
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
        let content = self
            .get_document_content(&params.text_document_position.text_document.uri)
            .await
            .unwrap_or_default();

        let offset = position_to_offset(params.text_document_position.position, &content);

        // Get text before cursor for context detection
        let text_before_cursor = content.chars().take(offset).collect::<String>();

        debug!(
            "Text before cursor: '{}'",
            text_before_cursor
                .chars()
                .rev()
                .take(50)
                .collect::<String>()
                .chars()
                .rev()
                .collect::<String>()
        );

        // Get context-specific completions
        if let Some(context_completions) = self
            .get_context_specific_completions(&text_before_cursor, &params.context)
            .await
        {
            completions.extend(context_completions);
        }

        // Add general Andromeda API completions
        let context_ref = params.context.as_ref();
        let general_completions =
            self.completion_provider
                .get_completions(context_ref, &content, offset);
        completions.extend(general_completions);

        debug!("Returning {} completions", completions.len());

        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        debug!(
            "Signature help request at: {:?}",
            params.text_document_position_params
        );

        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Get document content
        let content = match self.get_document_content(uri).await {
            Some(content) => content,
            None => return Ok(None),
        };

        let offset = position_to_offset(position, &content);

        // Get text before cursor to find function call context
        let text_before = content.chars().take(offset).collect::<String>();

        // Look for function call patterns
        if let Some(signature_info) = self.get_signature_help_info(&text_before).await {
            return Ok(Some(signature_info));
        }

        Ok(None)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        debug!(
            "Hover request at: {:?}",
            params.text_document_position_params
        );

        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Get document content
        let content = match self.get_document_content(uri).await {
            Some(content) => content,
            None => return Ok(None),
        };

        let offset = position_to_offset(position, &content);

        // Get word under cursor
        let word = self.get_word_at_position(&content, offset);

        if let Some(word) = word {
            debug!("Hover word: '{word}'");

            // Check if it's an Andromeda API
            if let Some(hover_info) = self.get_andromeda_api_hover(&word).await {
                return Ok(Some(hover_info));
            }

            // Check if it's a Web API
            if let Some(hover_info) = self.get_web_api_hover(&word).await {
                return Ok(Some(hover_info));
            }

            // Check context-specific hovers
            let text_before = content.chars().take(offset).collect::<String>();
            if let Some(hover_info) = self.get_context_specific_hover(&word, &text_before).await {
                return Ok(Some(hover_info));
            }
        }

        Ok(None)
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        debug!("Code action request for: {}", params.text_document.uri);

        let mut actions = Vec::new();

        // Get document content
        let content = match self.get_document_content(&params.text_document.uri).await {
            Some(content) => content,
            None => return Ok(None),
        };

        // Check if there are any diagnostics in the range
        for diagnostic in &params.context.diagnostics {
            if let Some(code_actions) = self
                .create_code_actions_for_diagnostic(&params.text_document.uri, diagnostic, &content)
                .await
            {
                actions.extend(code_actions);
            }
        }

        // Add general code actions
        actions.extend(
            self.get_general_code_actions(&params.text_document.uri, &params.range)
                .await,
        );

        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        debug!("Formatting request for: {}", params.text_document.uri);

        // Get document content
        let content = match self.get_document_content(&params.text_document.uri).await {
            Some(content) => content,
            None => return Ok(None),
        };

        // Use dprint for formatting
        match self.format_document(&content, &params.options).await {
            Ok(formatted_content) => {
                if formatted_content != content {
                    // Calculate the range for the entire document
                    let lines: Vec<&str> = content.lines().collect();
                    let last_line = lines.len().saturating_sub(1) as u32;
                    let last_char = lines.last().map(|line| line.len()).unwrap_or(0) as u32;

                    let edit = TextEdit {
                        range: Range {
                            start: Position {
                                line: 0,
                                character: 0,
                            },
                            end: Position {
                                line: last_line,
                                character: last_char,
                            },
                        },
                        new_text: formatted_content,
                    };

                    Ok(Some(vec![edit]))
                } else {
                    Ok(None)
                }
            }
            Err(e) => {
                warn!("Formatting failed: {e}");
                Ok(None)
            }
        }
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
                // Apply a specific auto-fix
                if !params.arguments.is_empty()
                    && let Some(uri_value) = params.arguments.first()
                    && let Ok(uri) = serde_json::from_value::<Url>(uri_value.clone())
                {
                    self.apply_auto_fix(&uri).await;
                }
                Ok(None)
            }
            "andromeda.fixAll" => {
                // Apply all auto-fixes to a document
                if !params.arguments.is_empty()
                    && let Some(uri_value) = params.arguments.first()
                    && let Ok(uri) = serde_json::from_value::<Url>(uri_value.clone())
                {
                    self.fix_all_problems(&uri).await;
                }
                Ok(None)
            }
            _ => {
                warn!("Unknown command: {}", params.command);
                Ok(None)
            }
        }
    }
}

impl AndromedaBackend {
    /// Apply a single auto-fix to a document
    async fn apply_auto_fix(&self, uri: &Url) {
        debug!("Applying auto-fix to: {uri}");

        // Get document content
        if let Some(content) = self.get_document_content(uri).await {
            let diagnostics = self.run_diagnostics(uri.clone(), &content).await;

            // Find the first fixable diagnostic and apply the fix
            for diagnostic in &diagnostics {
                if let Some(actions) = self
                    .create_code_actions_for_diagnostic(uri, diagnostic, &content)
                    .await
                    && let Some(CodeActionOrCommand::CodeAction(action)) = actions.first()
                    && let Some(edit) = &action.edit
                {
                    self.apply_workspace_edit(edit).await;
                    break;
                }
            }
        }
    }

    /// Fix all auto-fixable problems in a document
    async fn fix_all_problems(&self, uri: &Url) {
        debug!("Fixing all problems in: {uri}");

        // Get document content
        if let Some(mut content) = self.get_document_content(uri).await {
            let mut applied_fixes = 0;
            const MAX_ITERATIONS: usize = 10; // Prevent infinite loops

            // Keep applying fixes until no more can be applied
            for _iteration in 0..MAX_ITERATIONS {
                let diagnostics = self.run_diagnostics(uri.clone(), &content).await;
                let mut fixed_any = false;

                for diagnostic in &diagnostics {
                    if let Some(actions) = self
                        .create_code_actions_for_diagnostic(uri, diagnostic, &content)
                        .await
                        && let Some(CodeActionOrCommand::CodeAction(action)) = actions.first()
                        && let Some(edit) = &action.edit
                        && let Some(changes) = &edit.changes
                        && let Some(text_edits) = changes.get(uri)
                    {
                        // Apply the edits to our local content
                        content = self.apply_text_edits_to_content(&content, text_edits);
                        applied_fixes += 1;
                        fixed_any = true;
                        break; // Process one fix at a time to avoid conflicts
                    }
                }

                if !fixed_any {
                    break; // No more fixes to apply
                }
            }

            if applied_fixes > 0 {
                // Update the document in the editor
                self.apply_full_document_edit(uri, &content).await;
                info!("Applied {applied_fixes} auto-fixes to {uri}");
            }
        }
    }

    /// Apply text edits to content string
    fn apply_text_edits_to_content(&self, content: &str, edits: &[TextEdit]) -> String {
        let mut result = content.to_string();

        // Sort edits by position (reverse order to apply from end to start)
        let mut sorted_edits = edits.to_vec();
        sorted_edits.sort_by(|a, b| {
            b.range
                .start
                .line
                .cmp(&a.range.start.line)
                .then_with(|| b.range.start.character.cmp(&a.range.start.character))
        });

        for edit in sorted_edits {
            if let Some(start_offset) = self.position_to_offset_sync(&result, edit.range.start)
                && let Some(end_offset) = self.position_to_offset_sync(&result, edit.range.end)
            {
                let mut chars: Vec<char> = result.chars().collect();
                chars.splice(start_offset..end_offset, edit.new_text.chars());
                result = chars.into_iter().collect();
            }
        }

        result
    }

    /// Apply a workspace edit by sending it to the client
    async fn apply_workspace_edit(&self, edit: &WorkspaceEdit) {
        if let Err(e) = self.client.apply_edit(edit.clone()).await {
            warn!("Failed to apply workspace edit: {e:?}");
        }
    }

    /// Apply a full document edit
    async fn apply_full_document_edit(&self, uri: &Url, new_content: &str) {
        // Get current document to calculate the range
        if let Some(current_content) = self.get_document_content(uri).await {
            let lines: Vec<&str> = current_content.lines().collect();
            let last_line = lines.len().saturating_sub(1) as u32;
            let last_char = lines.last().map(|line| line.len()).unwrap_or(0) as u32;

            let edit = TextEdit {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: last_line,
                        character: last_char,
                    },
                },
                new_text: new_content.to_string(),
            };

            let mut changes = std::collections::HashMap::new();
            changes.insert(uri.clone(), vec![edit]);

            let workspace_edit = WorkspaceEdit {
                changes: Some(changes),
                ..Default::default()
            };

            self.apply_workspace_edit(&workspace_edit).await;
        }
    }
}
