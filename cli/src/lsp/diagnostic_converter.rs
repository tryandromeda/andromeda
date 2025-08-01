// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::lint::LintError;
use miette as oxc_miette;
use oxc_miette::SourceSpan;
use tower_lsp::lsp_types::*;

/// Convert Andromeda lint errors to LSP diagnostics
pub fn lint_error_to_diagnostic(lint_error: &LintError, source_code: &str) -> Diagnostic {
    let (message, severity, code, source) = match lint_error {
        LintError::EmptyStatement { .. } => (
            "Empty statement found".to_string(),
            DiagnosticSeverity::WARNING,
            Some(NumberOrString::String(
                "andromeda::lint::empty_statement".to_string(),
            )),
            Some("andromeda".to_string()),
        ),
        LintError::VarUsage { variable_name, .. } => (
            format!("Usage of 'var' for variable '{variable_name}'"),
            DiagnosticSeverity::WARNING,
            Some(NumberOrString::String(
                "andromeda::lint::var_usage".to_string(),
            )),
            Some("andromeda".to_string()),
        ),
        LintError::EmptyFunction { function_name, .. } => (
            format!("Function '{function_name}' has an empty body"),
            DiagnosticSeverity::WARNING,
            Some(NumberOrString::String(
                "andromeda::lint::empty_function".to_string(),
            )),
            Some("andromeda".to_string()),
        ),
        LintError::UnusedVariable { variable_name, .. } => (
            format!("Unused variable '{variable_name}'"),
            DiagnosticSeverity::WARNING,
            Some(NumberOrString::String(
                "andromeda::lint::unused_variable".to_string(),
            )),
            Some("andromeda".to_string()),
        ),
        LintError::PreferConst { variable_name, .. } => (
            format!("Variable '{variable_name}' could be const"),
            DiagnosticSeverity::INFORMATION,
            Some(NumberOrString::String(
                "andromeda::lint::prefer_const".to_string(),
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
        LintError::EmptyStatement { span, .. } => *span,
        LintError::VarUsage { span, .. } => *span,
        LintError::EmptyFunction { span, .. } => *span,
        LintError::UnusedVariable { span, .. } => *span,
        LintError::PreferConst { span, .. } => *span,
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
