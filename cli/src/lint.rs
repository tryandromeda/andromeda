// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::error::{AndromedaError, Result};
use console::Style;
use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_semantic::SymbolFlags;
use oxc_span::{SourceType, GetSpan};
use miette as oxc_miette;
use oxc_miette::{Diagnostic, NamedSource, SourceSpan};
use owo_colors::OwoColorize;
use std::fs;
use std::path::PathBuf;

/// Lint error types with enhanced diagnostics
#[derive(Diagnostic, Debug, Clone)]
pub enum LintError {
    /// Empty statement found
    #[diagnostic(
        code(andromeda::lint::empty_statement),
        help("🔍 Remove unnecessary semicolons that create empty statements.\n💡 Empty statements can make code harder to read and may indicate errors."),
        url("https://eslint.org/docs/latest/rules/no-empty-statement")
    )]
    EmptyStatement {
        #[label("Empty statement found here")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
    },

    /// Usage of 'var' keyword
    #[diagnostic(
        code(andromeda::lint::var_usage),
        help("🔍 Replace 'var' with 'let' or 'const' for better scoping.\n💡 'var' has function-level scoping which can lead to unexpected behavior.\n📖 Use 'let' for variables that will be reassigned, 'const' for constants."),
        url("https://eslint.org/docs/latest/rules/no-var")
    )]
    VarUsage {
        #[label("'var' keyword used here")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
        variable_name: String,
    },

    /// Function with empty body
    #[diagnostic(
        code(andromeda::lint::empty_function),
        help("🔍 Add implementation to the function or mark it as intentionally empty.\n💡 Empty functions may indicate incomplete implementation.\n📝 Consider adding a comment if the function is intentionally empty."),
        url("https://eslint.org/docs/latest/rules/no-empty-function")
    )]
    EmptyFunction {
        #[label("Function with empty body")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
        function_name: String,
    },

    /// Unused variable
    #[diagnostic(
        code(andromeda::lint::unused_variable),
        help("🔍 Remove the unused variable or prefix it with '_' if intentionally unused.\n💡 Unused variables can indicate dead code or typos in variable names.\n🧹 Removing unused variables helps keep code clean and maintainable."),
        url("https://eslint.org/docs/latest/rules/no-unused-vars")
    )]
    UnusedVariable {
        #[label("Unused variable '{variable_name}'")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
        variable_name: String,
    },
}

impl std::fmt::Display for LintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LintError::EmptyStatement { .. } => write!(f, "Empty statement found"),
            LintError::VarUsage { variable_name, .. } => write!(f, "Usage of 'var' for variable '{}'", variable_name),
            LintError::EmptyFunction { function_name, .. } => write!(f, "Function '{}' has empty body", function_name),
            LintError::UnusedVariable { variable_name, .. } => write!(f, "Unused variable '{}'", variable_name),
        }
    }
}

impl std::error::Error for LintError {}

/// Lint a single JS/TS file
#[allow(clippy::result_large_err)]
pub fn lint_file(path: &PathBuf) -> Result<()> {
    let content =
        fs::read_to_string(path).map_err(|e| AndromedaError::file_read_error(path.clone(), e))?;
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap_or_default();
    let ret = Parser::new(&allocator, &content, source_type).parse();
    let program = &ret.program;
    let mut lint_errors = Vec::new();

    // Create named source for error reporting
    let source_name = path.display().to_string();
    let named_source = NamedSource::new(source_name.clone(), content.clone());

    // Check for various lint issues
    for stmt in &program.body {
        match stmt {
            Statement::EmptyStatement(empty_stmt) => {
                let span = SourceSpan::new(
                    (empty_stmt.span().start as usize).into(),
                    empty_stmt.span().size() as usize,
                );
                lint_errors.push(LintError::EmptyStatement {
                    span,
                    source_code: named_source.clone(),
                });
            }
            Statement::VariableDeclaration(decl) => {
                if decl.kind.is_var() {
                    let span = SourceSpan::new(
                        (decl.span().start as usize).into(),
                        decl.span().size() as usize,
                    );
                    let variable_name = decl
                        .declarations
                        .first()
                        .and_then(|d| d.id.get_binding_identifier())
                        .map(|id| id.name.as_str())
                        .unwrap_or("<unknown>")
                        .to_string();
                    
                    lint_errors.push(LintError::VarUsage {
                        span,
                        source_code: named_source.clone(),
                        variable_name,
                    });
                }
            }
            Statement::FunctionDeclaration(func) => {
                if let Some(body) = &func.body {
                    if body.statements.is_empty() {
                        let span = SourceSpan::new(
                            (func.span().start as usize).into(),
                            func.span().size() as usize,
                        );
                        let function_name = func
                            .id
                            .as_ref()
                            .map(|id| id.name.as_str())
                            .unwrap_or("<anonymous>")
                            .to_string();
                        
                        lint_errors.push(LintError::EmptyFunction {
                            span,
                            source_code: named_source.clone(),
                            function_name,
                        });
                    }
                }
            }
            _ => {}
        }
    }

    // Check for unused variables using semantic analysis
    let semantic = SemanticBuilder::new().build(program);
    let scoping = semantic.semantic.scoping();
    for symbol_id in scoping.symbol_ids() {
        let flags = scoping.symbol_flags(symbol_id);
        if flags.intersects(
            SymbolFlags::BlockScopedVariable
                | SymbolFlags::ConstVariable
                | SymbolFlags::FunctionScopedVariable,
        ) && scoping.symbol_is_unused(symbol_id)
        {
            let name = scoping.symbol_name(symbol_id);
            if !name.starts_with('_') {
                let symbol_span = scoping.symbol_span(symbol_id);
                let span = SourceSpan::new(
                    (symbol_span.start as usize).into(),
                    symbol_span.size() as usize,
                );
                
                lint_errors.push(LintError::UnusedVariable {
                    span,
                    source_code: named_source.clone(),
                    variable_name: name.to_string(),
                });
            }
        }
    }

    // Report all lint errors using miette
    if !lint_errors.is_empty() {
        println!();
        println!(
            "{} {} ({} issue{} found)",
            "🔍".bright_yellow(),
            "Lint Issues".bright_yellow().bold(),
            lint_errors.len(),
            if lint_errors.len() == 1 { "" } else { "s" }
        );
        println!("{}", "─".repeat(60).yellow());
        
        for (i, error) in lint_errors.iter().enumerate() {
            if lint_errors.len() > 1 {
                println!();
                println!(
                    "{} Issue {} of {}:",
                    "📍".cyan(),
                    (i + 1).to_string().bright_cyan(),
                    lint_errors.len().to_string().bright_cyan()
                );
                println!("{}", "─".repeat(30).cyan());
            }
            println!("{:?}", oxc_miette::Report::new(error.clone()));
        }
        println!();
    } else {
        let ok = Style::new().green().bold().apply_to("✔");
        let file = Style::new().cyan().apply_to(path.display());
        let msg = Style::new().white().dim().apply_to("No lint issues found.");
        println!("{ok} {file}: {msg}");
    }
    
    Ok(())
}
