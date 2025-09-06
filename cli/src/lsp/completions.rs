// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::HashMap;
use tower_lsp::lsp_types::*;

/// Andromeda-specific completion data
#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AndromedaCompletionData {
    /// The module URI where this completion is relevant
    pub module_uri: Url,
    /// The position in the document
    pub position: u32,
    /// Additional completion metadata
    pub metadata: Option<serde_json::Value>,
}

// /// Completion item kinds specific to Andromeda
// pub struct AndromedaCompletionKind;

// impl AndromedaCompletionKind {
//     pub const ANDROMEDA_API: CompletionItemKind = CompletionItemKind::MODULE;
//     pub const WEB_API: CompletionItemKind = CompletionItemKind::INTERFACE;
//     pub const RUNTIME_EXTENSION: CompletionItemKind = CompletionItemKind::CLASS;
// }

/// Core completion provider for Andromeda APIs
pub struct AndromedaCompletionProvider {
    /// Built-in API completions
    pub api_completions: HashMap<String, Vec<CompletionItem>>,
}

impl AndromedaCompletionProvider {
    pub fn new() -> Self {
        let mut provider = Self {
            api_completions: HashMap::new(),
        };
        provider.initialize_builtin_completions();
        provider
    }

    /// Initialize all built-in API completions
    fn initialize_builtin_completions(&mut self) {
        self.add_andromeda_namespace_completions();
        self.add_web_api_completions();
        self.add_canvas_api_completions();
        self.add_crypto_api_completions();
        self.add_sqlite_api_completions();
        self.add_storage_api_completions();
        self.add_performance_api_completions();
    }

    /// Add completions for the main Andromeda namespace
    fn add_andromeda_namespace_completions(&mut self) {
        let items = vec![
            // File system operations
            create_completion_item(
                "readTextFileSync",
                CompletionItemKind::FUNCTION,
                "readTextFileSync(path: string): string",
                "Reads a text file from the file system synchronously.",
                "readTextFileSync(${1:path})",
                Some("🗎"),
            ),
            create_completion_item(
                "writeTextFileSync",
                CompletionItemKind::FUNCTION,
                "writeTextFileSync(path: string, data: string): void",
                "Writes a text file to the file system synchronously.",
                "writeTextFileSync(${1:path}, ${2:data})",
                Some("💾"),
            ),
            create_completion_item(
                "readFileSync",
                CompletionItemKind::FUNCTION,
                "readFileSync(path: string): Uint8Array",
                "Reads a binary file from the file system synchronously.",
                "readFileSync(${1:path})",
                Some("🗎"),
            ),
            create_completion_item(
                "writeFileSync",
                CompletionItemKind::FUNCTION,
                "writeFileSync(path: string, data: Uint8Array): void",
                "Writes binary data to a file synchronously.",
                "writeFileSync(${1:path}, ${2:data})",
                Some("💾"),
            ),
            create_completion_item(
                "existsSync",
                CompletionItemKind::FUNCTION,
                "existsSync(path: string): boolean",
                "Checks if a file or directory exists synchronously.",
                "existsSync(${1:path})",
                Some("🔍"),
            ),
            create_completion_item(
                "removeSync",
                CompletionItemKind::FUNCTION,
                "removeSync(path: string): void",
                "Removes a file from the file system synchronously.",
                "removeSync(${1:path})",
                Some("🗑️"),
            ),
            create_completion_item(
                "mkdirSync",
                CompletionItemKind::FUNCTION,
                "mkdirSync(path: string): void",
                "Creates a directory synchronously.",
                "mkdirSync(${1:path})",
                Some("📁"),
            ),
            // Environment operations
            create_completion_item(
                "env",
                CompletionItemKind::MODULE,
                "env: { get(key: string): string; set(key: string, value: string): void; ... }",
                "Environment variable operations.",
                "env",
                Some("🌍"),
            ),
            // Process operations
            create_completion_item(
                "args",
                CompletionItemKind::PROPERTY,
                "args: string[]",
                "Command-line arguments passed to the program.",
                "args",
                Some("📋"),
            ),
            create_completion_item(
                "exit",
                CompletionItemKind::FUNCTION,
                "exit(code?: number): void",
                "Exits the program with an optional exit code.",
                "exit(${1:0})",
                Some("🚪"),
            ),
            create_completion_item(
                "sleep",
                CompletionItemKind::FUNCTION,
                "sleep(duration: number): Promise<void>",
                "Returns a Promise that resolves after the specified duration in milliseconds.",
                "sleep(${1:1000})",
                Some("⏱️"),
            ),
            // I/O operations
            create_completion_item(
                "stdin",
                CompletionItemKind::MODULE,
                "stdin: { readLine(): string }",
                "Standard input operations.",
                "stdin",
                Some("⌨️"),
            ),
            create_completion_item(
                "stdout",
                CompletionItemKind::MODULE,
                "stdout: { write(message: string): void }",
                "Standard output operations.",
                "stdout",
                Some("🖥️"),
            ),
        ];

        self.api_completions.insert("Andromeda".to_string(), items);
    }

    /// Add completions for Web APIs
    fn add_web_api_completions(&mut self) {
        let items = vec![
            // Console API
            create_completion_item(
                "console",
                CompletionItemKind::MODULE,
                "console: Console",
                "Console API for logging and debugging.",
                "console",
                Some("🖥️"),
            ),
            // Fetch API
            create_completion_item(
                "fetch",
                CompletionItemKind::FUNCTION,
                "fetch(input: RequestInfo, init?: RequestInit): Promise<Response>",
                "Fetch API for making HTTP requests.",
                "fetch(${1:url})",
                Some("🌐"),
            ),
            // Text encoding/decoding
            create_completion_item(
                "TextEncoder",
                CompletionItemKind::CLASS,
                "TextEncoder: new() => TextEncoder",
                "Encodes strings to UTF-8 bytes.",
                "new TextEncoder()",
                Some("🔤"),
            ),
            create_completion_item(
                "TextDecoder",
                CompletionItemKind::CLASS,
                "TextDecoder: new(label?: string, options?: TextDecoderOptions) => TextDecoder",
                "Decodes bytes to strings.",
                "new TextDecoder(${1:'utf-8'})",
                Some("🔤"),
            ),
            // URL API
            create_completion_item(
                "URL",
                CompletionItemKind::CLASS,
                "URL: new(url: string, base?: string) => URL",
                "URL parsing and manipulation.",
                "new URL(${1:url})",
                Some("🔗"),
            ),
            create_completion_item(
                "URLSearchParams",
                CompletionItemKind::CLASS,
                "URLSearchParams: new(init?: string | string[][] | Record<string, string>) => URLSearchParams",
                "URL search parameters manipulation.",
                "new URLSearchParams(${1:params})",
                Some("🔍"),
            ),
            // Structured clone
            create_completion_item(
                "structuredClone",
                CompletionItemKind::FUNCTION,
                "structuredClone<T>(value: T, options?: StructuredSerializeOptions): T",
                "Creates a deep clone using the structured clone algorithm.",
                "structuredClone(${1:value})",
                Some("📋"),
            ),
            // Navigator
            create_completion_item(
                "navigator",
                CompletionItemKind::MODULE,
                "navigator: Navigator",
                "Navigator API with user agent and platform information.",
                "navigator",
                Some("🧭"),
            ),
            // Timers
            create_completion_item(
                "setTimeout",
                CompletionItemKind::FUNCTION,
                "setTimeout(callback: () => void, delay?: number): number",
                "Executes a function after a delay.",
                "setTimeout(${1:callback}, ${2:delay})",
                Some("⏰"),
            ),
            create_completion_item(
                "setInterval",
                CompletionItemKind::FUNCTION,
                "setInterval(callback: () => void, delay?: number): number",
                "Repeatedly executes a function at intervals.",
                "setInterval(${1:callback}, ${2:delay})",
                Some("🔄"),
            ),
            create_completion_item(
                "clearTimeout",
                CompletionItemKind::FUNCTION,
                "clearTimeout(id: number): void",
                "Cancels a timeout.",
                "clearTimeout(${1:id})",
                Some("❌"),
            ),
            create_completion_item(
                "clearInterval",
                CompletionItemKind::FUNCTION,
                "clearInterval(id: number): void",
                "Cancels an interval.",
                "clearInterval(${1:id})",
                Some("❌"),
            ),
            create_completion_item(
                "queueMicrotask",
                CompletionItemKind::FUNCTION,
                "queueMicrotask(callback: () => void): void",
                "Queues a microtask for execution.",
                "queueMicrotask(${1:callback})",
                Some("⚡"),
            ),
        ];

        self.api_completions.insert("web".to_string(), items);
    }

    /// Add completions for Canvas API
    fn add_canvas_api_completions(&mut self) {
        let items = vec![
            create_completion_item(
                "OffscreenCanvas",
                CompletionItemKind::CLASS,
                "OffscreenCanvas: new(width: number, height: number) => OffscreenCanvas",
                "GPU-accelerated off-screen canvas for graphics rendering.",
                "new OffscreenCanvas(${1:width}, ${2:height})",
                Some("🎨"),
            ),
            create_completion_item(
                "CanvasRenderingContext2D",
                CompletionItemKind::INTERFACE,
                "CanvasRenderingContext2D",
                "2D rendering context for canvas operations.",
                "CanvasRenderingContext2D",
                Some("🖌️"),
            ),
            create_completion_item(
                "createImageBitmap",
                CompletionItemKind::FUNCTION,
                "createImageBitmap(path: string): Promise<ImageBitmap>",
                "Creates an ImageBitmap from a file path or URL.",
                "createImageBitmap(${1:path})",
                Some("🖼️"),
            ),
        ];

        self.api_completions.insert("canvas".to_string(), items);
    }

    /// Add completions for Crypto API
    fn add_crypto_api_completions(&mut self) {
        let items = vec![
            create_completion_item(
                "crypto",
                CompletionItemKind::MODULE,
                "crypto: Crypto",
                "Web Crypto API for cryptographic operations.",
                "crypto",
                Some("🔐"),
            ),
            // crypto.subtle methods would be added here
            create_completion_item(
                "randomUUID",
                CompletionItemKind::FUNCTION,
                "crypto.randomUUID(): string",
                "Generates a cryptographically secure random UUID.",
                "crypto.randomUUID()",
                Some("🎲"),
            ),
            create_completion_item(
                "getRandomValues",
                CompletionItemKind::FUNCTION,
                "crypto.getRandomValues<T extends TypedArray>(array: T): T",
                "Fills a typed array with cryptographically secure random values.",
                "crypto.getRandomValues(${1:array})",
                Some("🎲"),
            ),
        ];

        self.api_completions.insert("crypto".to_string(), items);
    }

    /// Add completions for SQLite API
    fn add_sqlite_api_completions(&mut self) {
        let items = vec![
            create_completion_item(
                "Database",
                CompletionItemKind::CLASS,
                "Database: new(filename: string, options?: DatabaseSyncOptions) => DatabaseSync",
                "SQLite database connection.",
                "new Database(${1:filename})",
                Some("🗄️"),
            ),
            create_completion_item(
                "DatabaseSync",
                CompletionItemKind::CLASS,
                "DatabaseSync: SQLite database class",
                "Synchronous SQLite database operations.",
                "DatabaseSync",
                Some("🗄️"),
            ),
        ];

        self.api_completions.insert("sqlite".to_string(), items);
    }

    /// Add completions for Web Storage API
    fn add_storage_api_completions(&mut self) {
        let items = vec![
            create_completion_item(
                "localStorage",
                CompletionItemKind::MODULE,
                "localStorage: Storage",
                "Local storage for persistent data.",
                "localStorage",
                Some("💾"),
            ),
            create_completion_item(
                "sessionStorage",
                CompletionItemKind::MODULE,
                "sessionStorage: Storage",
                "Session storage for temporary data.",
                "sessionStorage",
                Some("🗃️"),
            ),
        ];

        self.api_completions.insert("storage".to_string(), items);
    }

    /// Add completions for Performance API
    fn add_performance_api_completions(&mut self) {
        let items = vec![create_completion_item(
            "performance",
            CompletionItemKind::MODULE,
            "performance: AndromedaPerformance",
            "High-resolution time measurements and performance monitoring.",
            "performance",
            Some("⚡"),
        )];

        self.api_completions
            .insert("performance".to_string(), items);
    }

    /// Get completions for a specific context
    pub fn get_completions(
        &self,
        context: Option<&CompletionContext>,
        text: &str,
        position: usize,
    ) -> Vec<CompletionItem> {
        let mut completions = Vec::new();

        // Check if we're in an Andromeda namespace context
        if text.contains("Andromeda.")
            && let Some(andromeda_completions) = self.api_completions.get("Andromeda")
        {
            completions.extend(andromeda_completions.clone());
        }

        // Check for global APIs
        completions.extend(self.get_global_completions(context, text, position));

        // Sort by relevance
        completions.sort_by(|a, b| {
            a.sort_text
                .as_ref()
                .unwrap_or(&a.label)
                .cmp(b.sort_text.as_ref().unwrap_or(&b.label))
        });

        completions
    }

    /// Get global API completions
    fn get_global_completions(
        &self,
        _context: Option<&CompletionContext>,
        text: &str,
        _position: usize,
    ) -> Vec<CompletionItem> {
        let mut completions = Vec::new();

        // Add web API completions
        if let Some(web_completions) = self.api_completions.get("web") {
            completions.extend(web_completions.clone());
        }

        // Add context-specific completions based on detected usage
        if (text.contains("canvas") || text.contains("OffscreenCanvas"))
            && let Some(canvas_completions) = self.api_completions.get("canvas")
        {
            completions.extend(canvas_completions.clone());
        }

        if text.contains("crypto")
            && let Some(crypto_completions) = self.api_completions.get("crypto")
        {
            completions.extend(crypto_completions.clone());
        }

        if (text.contains("Database") || text.contains("sqlite"))
            && let Some(sqlite_completions) = self.api_completions.get("sqlite")
        {
            completions.extend(sqlite_completions.clone());
        }

        if (text.contains("localStorage") || text.contains("sessionStorage"))
            && let Some(storage_completions) = self.api_completions.get("storage")
        {
            completions.extend(storage_completions.clone());
        }

        if text.contains("performance")
            && let Some(perf_completions) = self.api_completions.get("performance")
        {
            completions.extend(perf_completions.clone());
        }

        completions
    }
}

impl Default for AndromedaCompletionProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create a completion item
pub fn create_completion_item(
    label: &str,
    kind: CompletionItemKind,
    detail: &str,
    documentation: &str,
    insert_text: &str,
    icon: Option<&str>,
) -> CompletionItem {
    let mut item = CompletionItem {
        label: label.to_string(),
        kind: Some(kind),
        detail: Some(detail.to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: if let Some(icon) = icon {
                format!("{icon} {documentation}")
            } else {
                documentation.to_string()
            },
        })),
        insert_text: Some(insert_text.to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    };

    // Add sort text for better ordering
    item.sort_text = Some(format!(
        "{:04}",
        match kind {
            CompletionItemKind::FUNCTION => 1,
            CompletionItemKind::CLASS => 2,
            CompletionItemKind::MODULE => 3,
            CompletionItemKind::PROPERTY => 4,
            CompletionItemKind::INTERFACE => 5,
            _ => 9,
        }
    ));

    item
}

/// Environment-specific completions for Andromeda.env namespace
pub fn get_env_completions() -> Vec<CompletionItem> {
    vec![
        create_completion_item(
            "get",
            CompletionItemKind::FUNCTION,
            "get(key: string): string",
            "Gets the value of an environment variable.",
            "get(${1:key})",
            Some("🔑"),
        ),
        create_completion_item(
            "set",
            CompletionItemKind::FUNCTION,
            "set(key: string, value: string): void",
            "Sets the value of an environment variable.",
            "set(${1:key}, ${2:value})",
            Some("✏️"),
        ),
        create_completion_item(
            "remove",
            CompletionItemKind::FUNCTION,
            "remove(key: string): void",
            "Removes an environment variable.",
            "remove(${1:key})",
            Some("🗑️"),
        ),
        create_completion_item(
            "keys",
            CompletionItemKind::FUNCTION,
            "keys(): string[]",
            "Returns all environment variable keys.",
            "keys()",
            Some("🗂️"),
        ),
    ]
}

/// Console-specific completions
pub fn get_console_completions() -> Vec<CompletionItem> {
    vec![
        create_completion_item(
            "log",
            CompletionItemKind::FUNCTION,
            "log(...data: any[]): void",
            "Logs messages to the console.",
            "log(${1:message})",
            Some("📝"),
        ),
        create_completion_item(
            "error",
            CompletionItemKind::FUNCTION,
            "error(...data: any[]): void",
            "Logs error messages to the console.",
            "error(${1:message})",
            Some("❌"),
        ),
        create_completion_item(
            "warn",
            CompletionItemKind::FUNCTION,
            "warn(...data: any[]): void",
            "Logs warning messages to the console.",
            "warn(${1:message})",
            Some("⚠️"),
        ),
        create_completion_item(
            "info",
            CompletionItemKind::FUNCTION,
            "info(...data: any[]): void",
            "Logs info messages to the console.",
            "info(${1:message})",
            Some("ℹ️"),
        ),
        create_completion_item(
            "debug",
            CompletionItemKind::FUNCTION,
            "debug(...data: any[]): void",
            "Logs debug messages to the console.",
            "debug(${1:message})",
            Some("🐛"),
        ),
        create_completion_item(
            "table",
            CompletionItemKind::FUNCTION,
            "table(data: any): void",
            "Displays data in a table format.",
            "table(${1:data})",
            Some("📋"),
        ),
        create_completion_item(
            "time",
            CompletionItemKind::FUNCTION,
            "time(label?: string): void",
            "Starts a timer for performance measurement.",
            "time(${1:label})",
            Some("⏱️"),
        ),
        create_completion_item(
            "timeEnd",
            CompletionItemKind::FUNCTION,
            "timeEnd(label?: string): void",
            "Ends a timer and logs the elapsed time.",
            "timeEnd(${1:label})",
            Some("⏹️"),
        ),
    ]
}
