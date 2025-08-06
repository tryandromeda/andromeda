// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::lint::LintError;
use miette as oxc_miette;
use oxc_miette::SourceSpan;
use tower_lsp::lsp_types::*;

/// Convert Andromeda lint errors to LSP diagnostics
pub fn lint_error_to_diagnostic(lint_error: &LintError, source_code: &str) -> Diagnostic {
    let (message, severity, code, source) = match lint_error {
        LintError::NoEmpty { .. } => (
            "Empty statement found".to_string(),
            DiagnosticSeverity::WARNING,
            Some(NumberOrString::String(
                "andromeda::lint::no-empty".to_string(),
            )),
            Some("andromeda".to_string()),
        ),
        LintError::NoVar { variable_name, .. } => (
            format!("Usage of 'var' for variable '{variable_name}'"),
            DiagnosticSeverity::WARNING,
            Some(NumberOrString::String(
                "andromeda::lint::no-var".to_string(),
            )),
            Some("andromeda".to_string()),
        ),
        LintError::NoUnusedVars { variable_name, .. } => (
            format!("Unused variable '{variable_name}'"),
            DiagnosticSeverity::WARNING,
            Some(NumberOrString::String(
                "andromeda::lint::no-unused-vars".to_string(),
            )),
            Some("andromeda".to_string()),
        ),
        LintError::PreferConst { variable_name, .. } => (
            format!("Variable '{variable_name}' could be const"),
            DiagnosticSeverity::INFORMATION,
            Some(NumberOrString::String(
                "andromeda::lint::prefer-const".to_string(),
            )),
            Some("andromeda".to_string()),
        ),
        LintError::NoConsole { method_name, .. } => (
            format!("Console.{method_name}() usage found"),
            DiagnosticSeverity::WARNING,
            Some(NumberOrString::String(
                "andromeda::lint::no-console".to_string(),
            )),
            Some("andromeda".to_string()),
        ),
        LintError::NoDebugger { .. } => (
            "Debugger statement found".to_string(),
            DiagnosticSeverity::ERROR,
            Some(NumberOrString::String(
                "andromeda::lint::no-debugger".to_string(),
            )),
            Some("andromeda".to_string()),
        ),
        LintError::NoExplicitAny { .. } => (
            "Explicit 'any' type used".to_string(),
            DiagnosticSeverity::WARNING,
            Some(NumberOrString::String(
                "andromeda::lint::no-explicit-any".to_string(),
            )),
            Some("andromeda".to_string()),
        ),
        LintError::RequireAwait { function_name, .. } => (
            format!("Async function '{function_name}' lacks await"),
            DiagnosticSeverity::WARNING,
            Some(NumberOrString::String(
                "andromeda::lint::require-await".to_string(),
            )),
            Some("andromeda".to_string()),
        ),
        LintError::NoEval { .. } => (
            "eval() usage found".to_string(),
            DiagnosticSeverity::ERROR,
            Some(NumberOrString::String(
                "andromeda::lint::no-eval".to_string(),
            )),
            Some("andromeda".to_string()),
        ),
        LintError::Eqeqeq { operator, .. } => (
            format!("Use strict equality instead of '{operator}'"),
            DiagnosticSeverity::WARNING,
            Some(NumberOrString::String(
                "andromeda::lint::eqeqeq".to_string(),
            )),
            Some("andromeda".to_string()),
        ),
        LintError::Camelcase { name, .. } => (
            format!("Identifier '{name}' is not in camelCase"),
            DiagnosticSeverity::INFORMATION,
            Some(NumberOrString::String(
                "andromeda::lint::camelcase".to_string(),
            )),
            Some("andromeda".to_string()),
        ),
        LintError::NoBooleanLiteralForArguments { value, .. } => (
            format!("Boolean literal '{value}' passed as argument"),
            DiagnosticSeverity::INFORMATION,
            Some(NumberOrString::String(
                "andromeda::lint::no-boolean-literal-for-arguments".to_string(),
            )),
            Some("andromeda".to_string()),
        ),
    };

    let span = get_lint_error_span(lint_error);
    let range = span_to_range(span, source_code);

    Diagnostic {
        range,
        severity: Some(severity),
        code,
        code_description: None,
        source,
        message,
        related_information: None,
        tags: None,
        data: None,
    }
}

/// Extract the source span from a lint error
fn get_lint_error_span(lint_error: &LintError) -> SourceSpan {
    match lint_error {
        LintError::NoEmpty { span, .. } => *span,
        LintError::NoVar { span, .. } => *span,
        LintError::NoUnusedVars { span, .. } => *span,
        LintError::PreferConst { span, .. } => *span,
        LintError::NoConsole { span, .. } => *span,
        LintError::NoDebugger { span, .. } => *span,
        LintError::NoExplicitAny { span, .. } => *span,
        LintError::RequireAwait { span, .. } => *span,
        LintError::NoEval { span, .. } => *span,
        LintError::Eqeqeq { span, .. } => *span,
        LintError::Camelcase { span, .. } => *span,
        LintError::NoBooleanLiteralForArguments { span, .. } => *span,
    }
}

/// Convert a source span to an LSP range
fn span_to_range(span: SourceSpan, source_code: &str) -> Range {
    let start_offset = span.offset();
    let end_offset = start_offset + span.len();

    let start_position = offset_to_position(start_offset, source_code);
    let end_position = offset_to_position(end_offset, source_code);

    Range {
        start: start_position,
        end: end_position,
    }
}

/// Convert a byte offset to an LSP position
fn offset_to_position(offset: usize, source_code: &str) -> Position {
    let mut line = 0;
    let mut character = 0;
    let mut current_offset = 0;

    for ch in source_code.chars() {
        if current_offset >= offset {
            break;
        }

        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            character += 1;
        }

        current_offset += ch.len_utf8();
    }

    Position {
        line: line as u32,
        character: character as u32,
    }
}

/// Convert an LSP position to a byte offset
#[allow(dead_code)]
pub fn position_to_offset(position: Position, source_code: &str) -> usize {
    let mut current_line = 0;
    let mut current_character = 0;
    let mut offset = 0;

    for ch in source_code.chars() {
        if current_line == position.line && current_character == position.character {
            break;
        }

        if ch == '\n' {
            current_line += 1;
            current_character = 0;
        } else {
            current_character += 1;
        }

        offset += ch.len_utf8();
    }

    offset
}
