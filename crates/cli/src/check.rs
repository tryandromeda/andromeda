// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
#![allow(unused_assignments)]

use crate::config::AndromedaConfig;
use crate::error::CliResult;
use crate::helper::find_formattable_files;
use console::Style;
use miette as oxc_miette;
use miette::{Diagnostic, NamedSource, SourceSpan};
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Type checking error types with rich diagnostic information.
#[derive(Diagnostic, Debug, Clone)]
#[non_exhaustive]
pub enum TypeCheckError {
    /// Unknown identifier (could not be resolved against any scope or ambient declaration)
    #[diagnostic(code(andromeda::check::unknown_identifier))]
    UnknownIdentifier {
        name: String,
        #[label("Cannot find name '{name}'")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
    },
    /// Parse error
    #[diagnostic(code(andromeda::check::parse_error))]
    ParseError {
        message: String,
        #[label("Syntax error: {message}")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
    },
    /// Semantic error (anything from oxc_semantic that isn't a name resolution issue)
    #[diagnostic(code(andromeda::check::semantic_error))]
    SemanticError {
        message: String,
        #[label("Semantic error: {message}")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
    },
    /// Unused variable
    #[diagnostic(code(andromeda::check::unused_variable))]
    UnusedVariable {
        name: String,
        #[label("Variable '{name}' is declared but never used")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
    },
}

impl TypeCheckError {
    /// The stable diagnostic code for serialised output.
    pub fn code(&self) -> &'static str {
        match self {
            TypeCheckError::UnknownIdentifier { .. } => "andromeda::check::unknown_identifier",
            TypeCheckError::ParseError { .. } => "andromeda::check::parse_error",
            TypeCheckError::SemanticError { .. } => "andromeda::check::semantic_error",
            TypeCheckError::UnusedVariable { .. } => "andromeda::check::unused_variable",
        }
    }

    pub fn span(&self) -> SourceSpan {
        match self {
            TypeCheckError::UnknownIdentifier { span, .. } => *span,
            TypeCheckError::ParseError { span, .. } => *span,
            TypeCheckError::SemanticError { span, .. } => *span,
            TypeCheckError::UnusedVariable { span, .. } => *span,
        }
    }
}

impl std::fmt::Display for TypeCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeCheckError::UnknownIdentifier { name, .. } => {
                write!(f, "Cannot find name '{name}'")
            }
            TypeCheckError::ParseError { message, .. } => {
                write!(f, "Parse error: {message}")
            }
            TypeCheckError::SemanticError { message, .. } => {
                write!(f, "Semantic error: {message}")
            }
            TypeCheckError::UnusedVariable { name, .. } => {
                write!(f, "Variable '{name}' is declared but never used")
            }
        }
    }
}

impl std::error::Error for TypeCheckError {}

/// JSON-friendly view of a single diagnostic, for `--json` output.
#[derive(Debug, Serialize)]
pub struct JsonDiagnostic {
    pub path: String,
    pub code: &'static str,
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub offset: usize,
    pub length: usize,
}

impl JsonDiagnostic {
    fn from_error(path: &Path, source: &str, err: &TypeCheckError) -> Self {
        let span = err.span();
        let offset = span.offset();
        let length = span.len();
        let (line, column) = offset_to_line_column(source, offset);
        Self {
            path: path.display().to_string(),
            code: err.code(),
            message: err.to_string(),
            line,
            column,
            offset,
            length,
        }
    }
}

fn offset_to_line_column(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
    let mut byte = 0usize;
    for ch in source.chars() {
        if byte >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
        byte += ch.len_utf8();
    }
    (line, col)
}

/// Names that are always considered globally available, so they should not trigger `UnknownIdentifier`
const BUILTIN_GLOBALS: &[&str] = &[
    "globalThis",
    "undefined",
    "NaN",
    "Infinity",
    "this",
    "arguments",
    "eval",
    "Object",
    "Function",
    "Array",
    "String",
    "Boolean",
    "Number",
    "BigInt",
    "Symbol",
    "Date",
    "RegExp",
    "Error",
    "TypeError",
    "RangeError",
    "SyntaxError",
    "ReferenceError",
    "EvalError",
    "URIError",
    "AggregateError",
    "Promise",
    "Proxy",
    "Reflect",
    "JSON",
    "Math",
    "Map",
    "Set",
    "WeakMap",
    "WeakSet",
    "WeakRef",
    "FinalizationRegistry",
    "Iterator",
    "AsyncIterator",
    "ArrayBuffer",
    "SharedArrayBuffer",
    "DataView",
    "Atomics",
    "Int8Array",
    "Uint8Array",
    "Uint8ClampedArray",
    "Int16Array",
    "Uint16Array",
    "Int32Array",
    "Uint32Array",
    "Float16Array",
    "Float32Array",
    "Float64Array",
    "BigInt64Array",
    "BigUint64Array",
    "parseInt",
    "parseFloat",
    "isNaN",
    "isFinite",
    "encodeURI",
    "encodeURIComponent",
    "decodeURI",
    "decodeURIComponent",
    "console",
    "self",
    "performance",
    "setTimeout",
    "setInterval",
    "clearTimeout",
    "clearInterval",
    "queueMicrotask",
    "structuredClone",
    "fetch",
    "Request",
    "Response",
    "Headers",
    "URL",
    "URLPattern",
    "URLSearchParams",
    "TextEncoder",
    "TextEncoderStream",
    "TextDecoder",
    "TextDecoderStream",
    "ReadableStream",
    "WritableStream",
    "TransformStream",
    "ByteLengthQueuingStrategy",
    "CountQueuingStrategy",
    "Blob",
    "File",
    "FormData",
    "AbortController",
    "AbortSignal",
    "Event",
    "EventTarget",
    "CustomEvent",
    "MessageEvent",
    "ErrorEvent",
    "CloseEvent",
    "BroadcastChannel",
    "MessageChannel",
    "MessagePort",
    "crypto",
    "Crypto",
    "SubtleCrypto",
    "CryptoKey",
    "atob",
    "btoa",
    "alert",
    "confirm",
    "prompt",
    "localStorage",
    "sessionStorage",
    "Storage",
    "WebSocket",
    "Worker",
    "DOMException",
    "Navigator",
    "navigator",
    "clientInformation",
    "Image",
    "ImageData",
    "ImageBitmap",
    "OffscreenCanvas",
    "Path2D",
    "CanvasGradient",
    "CanvasPattern",
    "CanvasRenderingContext2D",
    "DOMMatrix",
    "DOMMatrixReadOnly",
    "TextMetrics",
    "createImageBitmap",
    "Database",
    "DatabaseSync",
    "StatementSync",
    "sqlite",
    "Cron",
    "Window",
    "createWindow",
    "Andromeda",
    "__andromeda__",
    "assert",
    "assertEquals",
    "assertNotEquals",
    "assertThrows",
    "constants",
    "exports",
];

fn known_globals() -> &'static HashSet<&'static str> {
    use std::sync::OnceLock;
    static GLOBALS: OnceLock<HashSet<&'static str>> = OnceLock::new();
    GLOBALS.get_or_init(|| BUILTIN_GLOBALS.iter().copied().collect())
}

fn is_known_global(name: &str) -> bool {
    known_globals().contains(name)
}

/// Type check file content directly
#[allow(clippy::result_large_err)]
pub fn check_file_content_with_config(
    path: &PathBuf,
    content: &str,
    _config_override: Option<AndromedaConfig>,
) -> CliResult<Vec<TypeCheckError>> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap_or_default();

    if !source_type.is_typescript() {
        return Ok(Vec::new());
    }

    let is_declaration = source_type.is_typescript_definition();

    let ret = Parser::new(&allocator, content, source_type).parse();
    let program = &ret.program;
    let mut type_errors: Vec<TypeCheckError> = Vec::new();

    let source_name = path.display().to_string();
    let named_source = NamedSource::new(source_name, content.to_string());

    for error in &ret.errors {
        if let Some(labels) = &error.labels
            && let Some(label) = labels.first()
        {
            let span = SourceSpan::new(label.offset().into(), label.len());
            type_errors.push(TypeCheckError::ParseError {
                message: error.to_string(),
                span,
                source_code: named_source.clone(),
            });
        }
    }

    let semantic_ret = SemanticBuilder::new()
        .with_check_syntax_error(true)
        .with_cfg(true)
        .build(program);

    let semantic = &semantic_ret.semantic;
    let scoping = semantic.scoping();

    let mut unresolved_spans: HashSet<(u32, u32)> = HashSet::new();

    for reference_id_list in scoping.root_unresolved_references_ids() {
        for reference_id in reference_id_list {
            let reference = scoping.get_reference(reference_id);

            if reference.flags().is_type_only() {
                continue;
            }

            let name = semantic.reference_name(reference);

            if is_known_global(name) {
                continue;
            }

            let ref_span = semantic.reference_span(reference);
            let key = (ref_span.start, ref_span.end);
            if !unresolved_spans.insert(key) {
                continue;
            }

            let span = SourceSpan::new((ref_span.start as usize).into(), ref_span.size() as usize);

            type_errors.push(TypeCheckError::UnknownIdentifier {
                name: name.to_string(),
                span,
                source_code: named_source.clone(),
            });
        }
    }

    for error in &semantic_ret.errors {
        let Some(labels) = &error.labels else {
            continue;
        };
        let Some(label) = labels.first() else {
            continue;
        };

        let start = label.offset() as u32;
        let end = start + label.len() as u32;
        if unresolved_spans.contains(&(start, end)) {
            continue;
        }

        let span = SourceSpan::new(label.offset().into(), label.len());
        type_errors.push(TypeCheckError::SemanticError {
            message: error.to_string(),
            span,
            source_code: named_source.clone(),
        });
    }

    if !is_declaration {
        for symbol_id in scoping.symbol_ids() {
            if !scoping.symbol_is_unused(symbol_id) {
                continue;
            }
            let name = scoping.symbol_name(symbol_id);
            if name.starts_with('_') {
                continue;
            }
            let symbol_span = scoping.symbol_span(symbol_id);
            let span = SourceSpan::new(
                (symbol_span.start as usize).into(),
                symbol_span.size() as usize,
            );
            type_errors.push(TypeCheckError::UnusedVariable {
                name: name.to_string(),
                span,
                source_code: named_source.clone(),
            });
        }
    }

    Ok(type_errors)
}

/// Output format for `andromeda check`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckOutputFormat {
    Pretty,
    Json,
    Quiet,
}

/// Type check multiple files
#[allow(clippy::result_large_err, dead_code)]
#[hotpath::measure]
pub fn check_files_with_config(
    paths: &[PathBuf],
    config_override: Option<AndromedaConfig>,
) -> CliResult<()> {
    check_files_with_options(paths, config_override, CheckOutputFormat::Pretty)
}

#[allow(clippy::result_large_err)]
pub fn check_files_with_options(
    paths: &[PathBuf],
    config_override: Option<AndromedaConfig>,
    format: CheckOutputFormat,
) -> CliResult<()> {
    let files_to_check: Vec<PathBuf> = if paths.is_empty() {
        find_formattable_files(&[PathBuf::from(".")])
            .unwrap_or_default()
            .into_iter()
            .filter(|path| {
                let source_type = SourceType::from_path(path).unwrap_or_default();
                source_type.is_typescript()
            })
            .collect()
    } else {
        find_formattable_files(paths)
            .unwrap_or_default()
            .into_iter()
            .filter(|path| {
                let source_type = SourceType::from_path(path).unwrap_or_default();
                source_type.is_typescript() || source_type.is_javascript()
            })
            .collect()
    };

    if files_to_check.is_empty() {
        if format == CheckOutputFormat::Pretty {
            let warning = Style::new().yellow().apply_to("Warning");
            eprintln!("{warning} No matching files found.");
        }
        return Ok(());
    }

    let mut total_errors = 0usize;
    let mut files_with_read_errors = 0usize;

    for path in &files_to_check {
        if format == CheckOutputFormat::Pretty {
            let label = Style::new().green().apply_to("Check");
            eprintln!("{label} {}", path.display());
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                files_with_read_errors += 1;
                if format == CheckOutputFormat::Pretty {
                    let prefix = Style::new().red().bold().apply_to("error");
                    eprintln!("{prefix}: Failed to read {}: {e}", path.display());
                }
                continue;
            }
        };

        match check_file_content_with_config(path, &content, config_override.clone()) {
            Ok(type_errors) => {
                total_errors += type_errors.len();
                emit_results(path, &content, &type_errors, format);
            }
            Err(e) => {
                total_errors += 1;
                if format == CheckOutputFormat::Pretty {
                    let prefix = Style::new().red().bold().apply_to("error");
                    eprintln!("{prefix}: Failed to type-check {}: {e}", path.display());
                }
            }
        }
    }

    if format == CheckOutputFormat::Pretty && total_errors > 0 {
        eprintln!();
        let plural = if total_errors == 1 { "error" } else { "errors" };
        eprintln!("Found {total_errors} {plural}.");
    }

    if total_errors > 0 || files_with_read_errors > 0 {
        std::process::exit(1);
    }

    Ok(())
}

/// Maximum number of type errors to display per file before truncating.
const MAX_DISPLAY_ERRORS: usize = 20;

fn emit_results(
    path: &Path,
    content: &str,
    type_errors: &[TypeCheckError],
    format: CheckOutputFormat,
) {
    match format {
        CheckOutputFormat::Pretty => display_type_check_results(type_errors),
        CheckOutputFormat::Json => emit_json_results(path, content, type_errors),
        CheckOutputFormat::Quiet => {}
    }
}

fn emit_json_results(path: &Path, content: &str, type_errors: &[TypeCheckError]) {
    for err in type_errors {
        let diag = JsonDiagnostic::from_error(path, content, err);
        if let Ok(line) = serde_json::to_string(&diag) {
            println!("{line}");
        }
    }
}

/// Display type check diagnostics. Silent when there are none.
fn display_type_check_results(type_errors: &[TypeCheckError]) {
    if type_errors.is_empty() {
        return;
    }

    let errors_to_show = type_errors.len().min(MAX_DISPLAY_ERRORS);

    for error in type_errors.iter().take(errors_to_show) {
        let report = oxc_miette::Report::new(error.clone());
        eprintln!("{report:?}");
    }

    let remaining = type_errors.len() - errors_to_show;
    if remaining > 0 {
        let plural = if remaining == 1 { "" } else { "s" };
        eprintln!("... {remaining} more diagnostic{plural} not shown.");
    }
}
