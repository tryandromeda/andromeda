// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::config::AndromedaConfig;
use crate::error::{AndromedaError, Result};
use crate::helper::find_formattable_files;
use console::Style;
use miette as oxc_miette;
use miette::{Diagnostic, NamedSource, SourceSpan};
use owo_colors::OwoColorize;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use std::fs;
use std::path::{Path, PathBuf};

/// Type checking error types with rich diagnostic information
#[allow(dead_code)]
#[derive(Diagnostic, Debug, Clone)]
pub enum TypeCheckError {
    /// Type mismatch error
    #[diagnostic(
        code(andromeda::check::type_mismatch),
        help(
            "🔍 Ensure the value matches the expected type.\n💡 Check variable assignments and function return types.\n📖 Use type assertions (as Type) or type guards if needed."
        ),
        url("https://www.typescriptlang.org/docs/handbook/2/everyday-types.html#type-annotations")
    )]
    TypeMismatch {
        expected: String,
        actual: String,
        #[label("Type '{actual}' is not assignable to type '{expected}'")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
    },
    /// Unknown identifier
    #[diagnostic(
        code(andromeda::check::unknown_identifier),
        help(
            "🔍 Check for typos in the identifier name.\n💡 Ensure the variable is declared before use.\n📖 Import the identifier if it's from another module."
        ),
        url("https://www.typescriptlang.org/docs/handbook/variable-declarations.html")
    )]
    UnknownIdentifier {
        name: String,
        #[label("Cannot find name '{name}'")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
    },
    /// Property does not exist on type
    #[diagnostic(
        code(andromeda::check::property_not_found),
        help(
            "🔍 Check the property name for typos.\n💡 Ensure the object has the expected shape.\n📖 Use optional chaining (?.) if the property might not exist."
        ),
        url("https://www.typescriptlang.org/docs/handbook/2/objects.html#property-access")
    )]
    PropertyNotFound {
        property: String,
        object_type: String,
        #[label("Property '{property}' does not exist on type '{object_type}'")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
    },
    /// Function call with incorrect arguments
    #[allow(dead_code)]
    #[diagnostic(
        code(andromeda::check::argument_mismatch),
        help(
            "🔍 Check the function signature and provide the correct number of arguments.\n💡 Use optional parameters (?) or default parameters if applicable.\n📖 Consider function overloads if the function supports multiple call signatures."
        ),
        url("https://www.typescriptlang.org/docs/handbook/2/functions.html")
    )]
    ArgumentMismatch {
        function_name: String,
        expected_args: usize,
        actual_args: usize,
        #[label(
            "Expected {expected_args} arguments for function '{function_name}', but got {actual_args}"
        )]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
    },
    /// Return type mismatch
    #[allow(dead_code)]
    #[diagnostic(
        code(andromeda::check::return_type_mismatch),
        help(
            "🔍 Ensure the return value matches the function's return type.\n💡 Check all code paths return the expected type.\n📖 Use union types if multiple return types are valid."
        ),
        url(
            "https://www.typescriptlang.org/docs/handbook/2/functions.html#return-type-annotations"
        )
    )]
    ReturnTypeMismatch {
        expected: String,
        actual: String,
        #[label("Return type '{actual}' is not assignable to expected return type '{expected}'")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
    },
    /// Cannot assign to readonly property
    #[diagnostic(
        code(andromeda::check::readonly_assignment),
        help(
            "🔍 Remove the assignment to the readonly property.\n💡 Use a different approach like creating a new object.\n📖 Consider if the property should be mutable instead."
        ),
        url("https://www.typescriptlang.org/docs/handbook/2/objects.html#readonly-properties")
    )]
    ReadonlyAssignment {
        property: String,
        #[label("Cannot assign to '{property}' because it is a read-only property")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
    },
    /// Missing type annotation
    #[diagnostic(
        code(andromeda::check::missing_type_annotation),
        help(
            "🔍 Add a type annotation to the identifier.\n💡 Use type inference when possible, explicit types when needed.\n📖 Consider using const assertions for literal types."
        ),
        url("https://www.typescriptlang.org/docs/handbook/2/everyday-types.html#type-annotations")
    )]
    MissingTypeAnnotation {
        identifier: String,
        #[label("Missing type annotation for '{identifier}'")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
    },
    /// Unreachable code
    #[diagnostic(
        code(andromeda::check::unreachable_code),
        help(
            "🔍 Remove the unreachable code.\n💡 Check for early returns or conditions that prevent execution.\n📖 Consider restructuring the code flow."
        ),
        url("https://www.typescriptlang.org/docs/handbook/2/narrowing.html")
    )]
    UnreachableCode {
        #[label("This code is unreachable")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
    },
    /// Parse error
    #[diagnostic(
        code(andromeda::check::parse_error),
        help(
            "🔍 Fix the syntax error in the code.\n💡 Check for missing brackets, semicolons, or incorrect syntax.\n📖 Ensure proper TypeScript/JavaScript syntax."
        ),
        url("https://www.typescriptlang.org/docs/handbook/intro.html")
    )]
    ParseError {
        message: String,
        #[label("Syntax error: {message}")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
    },
    /// Semantic error
    #[diagnostic(
        code(andromeda::check::semantic_error),
        help(
            "🔍 Fix the semantic error in the code.\n💡 Check variable declarations, scope, and language semantics.\n📖 Ensure proper language usage and structure."
        ),
        url("https://www.typescriptlang.org/docs/handbook/intro.html")
    )]
    SemanticError {
        message: String,
        #[label("Semantic error: {message}")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
    },
    /// Unused variable
    #[diagnostic(
        code(andromeda::check::unused_variable),
        help(
            "🔍 Remove the unused variable or prefix with underscore (_) to indicate intentional.\n💡 Consider if the variable is needed for future use.\n📖 Use ESLint's no-unused-vars rule to catch these automatically."
        ),
        url("https://www.typescriptlang.org/docs/handbook/variable-declarations.html")
    )]
    UnusedVariable {
        name: String,
        #[label("Variable '{name}' is declared but never used")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
    },
}

impl std::fmt::Display for TypeCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeCheckError::TypeMismatch {
                expected, actual, ..
            } => {
                write!(f, "Type '{actual}' is not assignable to type '{expected}'")
            }
            TypeCheckError::UnknownIdentifier { name, .. } => {
                write!(f, "Cannot find name '{name}'")
            }
            TypeCheckError::PropertyNotFound {
                property,
                object_type,
                ..
            } => {
                write!(
                    f,
                    "Property '{property}' does not exist on type '{object_type}'"
                )
            }
            TypeCheckError::ArgumentMismatch {
                function_name,
                expected_args,
                actual_args,
                ..
            } => {
                write!(
                    f,
                    "Expected {expected_args} arguments for function '{function_name}', but got {actual_args}"
                )
            }
            TypeCheckError::ReturnTypeMismatch {
                expected, actual, ..
            } => {
                write!(
                    f,
                    "Return type '{actual}' is not assignable to expected return type '{expected}'"
                )
            }
            TypeCheckError::ReadonlyAssignment { property, .. } => {
                write!(
                    f,
                    "Cannot assign to '{property}' because it is a read-only property"
                )
            }
            TypeCheckError::MissingTypeAnnotation { identifier, .. } => {
                write!(f, "Missing type annotation for '{identifier}'")
            }
            TypeCheckError::UnreachableCode { .. } => {
                write!(f, "Unreachable code detected")
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

/// Type check a single TypeScript file
#[allow(clippy::result_large_err)]
pub fn check_file_with_config(
    path: &PathBuf,
    config_override: Option<AndromedaConfig>,
) -> Result<Vec<TypeCheckError>> {
    let content =
        fs::read_to_string(path).map_err(|e| AndromedaError::file_read_error(path.clone(), e))?;

    check_file_content_with_config(path, &content, config_override)
}

/// Type check file content directly
#[allow(clippy::result_large_err)]
pub fn check_file_content_with_config(
    path: &PathBuf,
    content: &str,
    _config_override: Option<AndromedaConfig>,
) -> Result<Vec<TypeCheckError>> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap_or_default();

    if !source_type.is_typescript() {
        return Ok(Vec::new());
    }

    let ret = Parser::new(&allocator, content, source_type).parse();
    let program = &ret.program;
    let mut type_errors = Vec::new();

    if !ret.errors.is_empty() {
        for error in &ret.errors {
            let source_name = path.display().to_string();
            let named_source = NamedSource::new(source_name, content.to_string());

            if let Some(labels) = &error.labels
                && let Some(label) = labels.first()
            {
                let span = SourceSpan::new(label.offset().into(), label.len());

                type_errors.push(TypeCheckError::ParseError {
                    message: error.to_string(),
                    span,
                    source_code: named_source,
                });
            }
        }
    }

    let semantic_ret = SemanticBuilder::new()
        .with_check_syntax_error(true)
        .with_cfg(true)
        .build(program);

    let source_name = path.display().to_string();
    let named_source = NamedSource::new(source_name, content.to_string());

    for error in &semantic_ret.errors {
        if let Some(labels) = &error.labels
            && let Some(label) = labels.first()
        {
            let span = SourceSpan::new(label.offset().into(), label.len());

            let error_message = error.to_string();

            if error_message.contains("Cannot find name") {
                if let Some(name) = extract_identifier_from_error(&error_message) {
                    type_errors.push(TypeCheckError::UnknownIdentifier {
                        name,
                        span,
                        source_code: named_source.clone(),
                    });
                }
            } else if error_message.contains("Property") && error_message.contains("does not exist")
            {
                if let (Some(property), Some(object_type)) =
                    extract_property_error_info(&error_message)
                {
                    type_errors.push(TypeCheckError::PropertyNotFound {
                        property,
                        object_type,
                        span,
                        source_code: named_source.clone(),
                    });
                }
            } else if error_message.contains("not assignable to") {
                if let (Some(actual), Some(expected)) = extract_type_mismatch_info(&error_message) {
                    type_errors.push(TypeCheckError::TypeMismatch {
                        expected,
                        actual,
                        span,
                        source_code: named_source.clone(),
                    });
                }
            } else {
                type_errors.push(TypeCheckError::SemanticError {
                    message: error.to_string(),
                    span,
                    source_code: named_source.clone(),
                });
            }
        }
    }

    let _semantic = &semantic_ret.semantic;
    let scoping = _semantic.scoping();

    for reference_id_list in scoping.root_unresolved_references_ids() {
        for reference_id in reference_id_list {
            let reference = scoping.get_reference(reference_id);
            let name = _semantic.reference_name(reference);
            let ref_span = _semantic.reference_span(reference);

            let span = SourceSpan::new((ref_span.start as usize).into(), ref_span.size() as usize);

            type_errors.push(TypeCheckError::UnknownIdentifier {
                name: name.to_string(),
                span,
                source_code: named_source.clone(),
            });
        }
    }

    for symbol_id in scoping.symbol_ids() {
        if scoping.symbol_is_unused(symbol_id) {
            let name = scoping.symbol_name(symbol_id);
            let symbol_span = scoping.symbol_span(symbol_id);

            if !name.starts_with('_') {
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
    }

    Ok(type_errors)
}

/// Helper function to extract identifier name from error message
fn extract_identifier_from_error(error_message: &str) -> Option<String> {
    let patterns = [
        "Cannot find name '",
        "Identifier '",
        "Variable '",
        "Function '",
        "Property '",
        "Type '",
        "Interface '",
        "Class '",
        "Enum '",
        "Namespace '",
        "Module '",
    ];

    for pattern in &patterns {
        if let Some(start_pos) = error_message.find(pattern) {
            let content_start = start_pos + pattern.len();
            if let Some(end_pos) = error_message[content_start..].find("'") {
                let identifier = &error_message[content_start..content_start + end_pos];
                if !identifier.is_empty()
                    && identifier
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '_' || c == '$')
                {
                    return Some(identifier.to_string());
                }
            }
        }
    }

    let mut in_quotes = false;
    let mut current_identifier = String::new();

    for ch in error_message.chars() {
        if ch == '\'' {
            if in_quotes && !current_identifier.is_empty() {
                if current_identifier
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c == '$')
                {
                    return Some(current_identifier);
                }
                current_identifier.clear();
            }
            in_quotes = !in_quotes;
        } else if in_quotes {
            current_identifier.push(ch);
        }
    }

    None
}

/// Helper function to extract property error information
fn extract_property_error_info(error_message: &str) -> (Option<String>, Option<String>) {
    if let Some(prop_start) = error_message.find("Property '") {
        let prop_content_start = prop_start + "Property '".len();
        if let Some(prop_end) = error_message[prop_content_start..].find("'") {
            let property =
                error_message[prop_content_start..prop_content_start + prop_end].to_string();

            if let Some(type_start) = error_message.find("on type '") {
                let type_content_start = type_start + "on type '".len();
                if let Some(type_end) = error_message[type_content_start..].find("'") {
                    let object_type = error_message
                        [type_content_start..type_content_start + type_end]
                        .to_string();
                    return (Some(property), Some(object_type));
                }
            }

            return (Some(property), Some("unknown".to_string()));
        }
    }

    if error_message.contains("property does not exist on") {
        let parts: Vec<&str> = error_message.split('\'').collect();
        if parts.len() >= 4 {
            let property = parts[1].to_string();
            let object_type = if parts.len() >= 6 {
                parts[3].to_string()
            } else {
                "unknown".to_string()
            };
            return (Some(property), Some(object_type));
        }
    }

    if let Some(property) = extract_identifier_from_error(error_message) {
        return (Some(property), Some("unknown".to_string()));
    }

    (None, None)
}

/// Helper function to extract type mismatch information
fn extract_type_mismatch_info(error_message: &str) -> (Option<String>, Option<String>) {
    if let Some(actual_start) = error_message.find("Type '") {
        let actual_content_start = actual_start + "Type '".len();
        if let Some(actual_end) = error_message[actual_content_start..].find("'") {
            let actual_type =
                error_message[actual_content_start..actual_content_start + actual_end].to_string();

            if let Some(expected_start) = error_message.find("assignable to type '") {
                let expected_content_start = expected_start + "assignable to type '".len();
                if let Some(expected_end) = error_message[expected_content_start..].find("'") {
                    let expected_type = error_message
                        [expected_content_start..expected_content_start + expected_end]
                        .to_string();
                    return (Some(actual_type), Some(expected_type));
                }
            }
        }
    }

    if error_message.contains("is not assignable to") {
        let parts: Vec<&str> = error_message.split('\'').collect();
        if parts.len() >= 4 {
            let actual_type = parts[1].to_string();
            let expected_type = if parts.len() >= 6 {
                parts[3].to_string()
            } else {
                "unknown".to_string()
            };
            return (Some(actual_type), Some(expected_type));
        }
    }

    if error_message.contains("Cannot assign") {
        let parts: Vec<&str> = error_message.split('\'').collect();
        if parts.len() >= 4 {
            let actual_type = parts[1].to_string();
            let expected_type = if parts.len() >= 6 {
                parts[3].to_string()
            } else {
                "unknown".to_string()
            };
            return (Some(actual_type), Some(expected_type));
        }
    }

    if error_message.contains("Return type") && error_message.contains("not assignable to") {
        let parts: Vec<&str> = error_message.split('\'').collect();
        if parts.len() >= 4 {
            let actual_type = parts[1].to_string();
            let expected_type = if parts.len() >= 6 {
                parts[3].to_string()
            } else {
                "unknown".to_string()
            };
            return (Some(actual_type), Some(expected_type));
        }
    }

    let quoted_types: Vec<&str> = error_message
        .split('\'')
        .enumerate()
        .filter_map(|(i, part)| if i % 2 == 1 { Some(part) } else { None })
        .collect();

    if quoted_types.len() >= 2 {
        return (
            Some(quoted_types[0].to_string()),
            Some(quoted_types[1].to_string()),
        );
    } else if quoted_types.len() == 1 {
        return (
            Some(quoted_types[0].to_string()),
            Some("unknown".to_string()),
        );
    }

    (Some("unknown".to_string()), Some("unknown".to_string()))
}

/// Type check multiple files
#[allow(clippy::result_large_err)]
#[hotpath::measure]
pub fn check_files_with_config(
    paths: &[PathBuf],
    config_override: Option<AndromedaConfig>,
) -> Result<()> {
    let files_to_check: Vec<PathBuf> = if paths.is_empty() {
        // If no paths provided, check all TypeScript files in current directory
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
        let warning = Style::new().yellow().bold().apply_to("⚠️");
        let msg = Style::new()
            .yellow()
            .apply_to("No TypeScript files found to type-check.");
        println!("{warning} {msg}");
        return Ok(());
    }

    let count = Style::new().cyan().apply_to(files_to_check.len());
    println!("Found {count} file(s) to type-check");
    println!("{}", Style::new().dim().apply_to("─".repeat(40)));

    let mut total_errors = 0;
    let mut files_with_errors = 0;

    for path in &files_to_check {
        match check_file_with_config(path, config_override.clone()) {
            Ok(type_errors) => {
                if !type_errors.is_empty() {
                    files_with_errors += 1;
                    total_errors += type_errors.len();
                }
                display_type_check_results(path, &type_errors);
            }
            Err(e) => {
                let error_icon = Style::new().red().bold().apply_to("❌");
                let file = Style::new().cyan().apply_to(path.display());
                println!("{error_icon} {file}: Failed to type-check - {e}");
                files_with_errors += 1;
            }
        }
    }

    println!();
    if total_errors == 0 {
        let success = Style::new().green().bold().apply_to("✅");
        let complete_msg = Style::new()
            .green()
            .bold()
            .apply_to("Type checking complete");
        let files_count = Style::new().green().bold().apply_to(files_to_check.len());
        println!("{success} {complete_msg}: All {files_count} files passed type checking!");
    } else {
        let warning = Style::new().yellow().bold().apply_to("⚠️");
        let complete_msg = Style::new()
            .yellow()
            .bold()
            .apply_to("Type checking complete");
        let error_count = Style::new().red().bold().apply_to(total_errors);
        let file_count = Style::new().red().bold().apply_to(files_with_errors);
        println!("{warning} {complete_msg}: Found {error_count} error(s) in {file_count} file(s)");
        return Err(AndromedaError::runtime_error(
            "Type checking completed with errors".to_string(),
            None,
            None,
            None,
            None,
        ));
    }

    Ok(())
}

/// Display type check results to the console with rich diagnostics
fn display_type_check_results(path: &Path, type_errors: &[TypeCheckError]) {
    if !type_errors.is_empty() {
        let error_icon = Style::new().red().bold().apply_to("❌");
        let file = Style::new().cyan().apply_to(path.display());
        let error_count = Style::new().red().bold().apply_to(type_errors.len());
        let error_text = if type_errors.len() == 1 {
            "error"
        } else {
            "errors"
        };

        println!("{error_icon} {file}: {error_count} type {error_text}");

        for (i, error) in type_errors.iter().enumerate() {
            if type_errors.len() > 1 {
                println!();
                println!(
                    "     {} Issue {} of {}:",
                    "📍".cyan(),
                    (i + 1).to_string().bright_cyan(),
                    type_errors.len().to_string().bright_cyan()
                );
                println!("     {}", "─".repeat(25).cyan());
            }
            let report = oxc_miette::Report::new(error.clone());
            let report_str = format!("{report:?}");
            for line in report_str.lines() {
                println!("     {line}");
            }
        }

        if type_errors.len() < type_errors.len() {
            println!();
            println!(
                "     {} {} more issue{} not shown",
                "⚠️".bright_yellow(),
                (type_errors.len() - type_errors.len())
                    .to_string()
                    .bright_yellow(),
                if type_errors.len() - type_errors.len() == 1 {
                    ""
                } else {
                    "s"
                }
            );
        }

        println!();
    } else {
        let ok = Style::new().green().bold().apply_to("✅");
        let file = Style::new().cyan().apply_to(path.display());
        let msg = Style::new().white().dim().apply_to("No type errors found.");
        println!("{ok} {file}: {msg}");
    }
}
