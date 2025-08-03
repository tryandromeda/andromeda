// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::config::{AndromedaConfig, ConfigManager, LintConfig};
use crate::error::{AndromedaError, Result};
use console::Style;
use miette as oxc_miette;
use owo_colors::OwoColorize;
use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_miette::{Diagnostic, NamedSource, SourceSpan};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_semantic::SymbolFlags;
use oxc_span::{GetSpan, SourceType};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Lint error types with enhanced diagnostics
#[derive(Diagnostic, Debug, Clone)]
pub enum LintError {
    /// Empty statement found
    #[diagnostic(
        code(andromeda::lint::empty_statement),
        help(
            "🔍 Remove unnecessary semicolons that create empty statements.\n💡 Empty statements can make code harder to read and may indicate errors."
        ),
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
        help(
            "🔍 Replace 'var' with 'let' or 'const' for better scoping.\n💡 'var' has function-level scoping which can lead to unexpected behavior.\n📖 Use 'let' for variables that will be reassigned, 'const' for constants."
        ),
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
        help(
            "🔍 Add implementation to the function or mark it as intentionally empty.\n💡 Empty functions may indicate incomplete implementation.\n📝 Consider adding a comment if the function is intentionally empty."
        ),
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
        help(
            "🔍 Remove the unused variable or prefix it with '_' if intentionally unused.\n💡 Unused variables can indicate dead code or typos in variable names.\n🧹 Removing unused variables helps keep code clean and maintainable."
        ),
        url("https://eslint.org/docs/latest/rules/no-unused-vars")
    )]
    UnusedVariable {
        #[label("Unused variable '{variable_name}'")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
        variable_name: String,
    },

    /// Variable could be const
    #[diagnostic(
        code(andromeda::lint::prefer_const),
        help(
            "🔍 Use 'const' instead of 'let' for variables that are never reassigned.\n💡 'const' prevents accidental reassignment and makes intent clearer.\n📖 Save 'let' for variables that will be modified."
        ),
        url("https://eslint.org/docs/latest/rules/prefer-const")
    )]
    PreferConst {
        #[label("Variable '{variable_name}' is never reassigned, use 'const'")]
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
            LintError::VarUsage { variable_name, .. } => {
                write!(f, "Usage of 'var' for variable '{variable_name}'")
            }
            LintError::EmptyFunction { function_name, .. } => {
                write!(f, "Function '{function_name}' has empty body")
            }
            LintError::UnusedVariable { variable_name, .. } => {
                write!(f, "Unused variable '{variable_name}'")
            }
            LintError::PreferConst { variable_name, .. } => {
                write!(f, "Variable '{variable_name}' could be const")
            }
        }
    }
}

impl std::error::Error for LintError {}

/// Helper function to check if a lint rule should be applied
/// Rules are enabled if:
/// 1. They are explicitly in the `rules` list, OR
/// 2. They are not in the `disabled_rules` list (default enabled)
///    AND the rule is not in the `disabled_rules` list
fn is_rule_enabled(rule_name: &str, lint_config: &LintConfig) -> bool {
    // If disabled_rules contains the rule, it's disabled
    if lint_config.disabled_rules.contains(&rule_name.to_string()) {
        return false;
    }

    // If rules list is empty, all rules are enabled by default (unless disabled)
    // If rules list is not empty, only explicitly listed rules are enabled
    if lint_config.rules.is_empty() {
        true
    } else {
        lint_config.rules.contains(&rule_name.to_string())
    }
}

/// Helper function to recursively check expressions for lint issues
fn check_expression_for_issues(
    expr: &oxc_ast::ast::Expression,
    _source_code: &str,
    _named_source: &NamedSource<String>,
    _lint_errors: &mut Vec<LintError>,
    _lint_config: &LintConfig,
) {
    use oxc_ast::ast::Expression;

    if let Expression::CallExpression(call) = expr {
        for arg in &call.arguments {
            if let Some(expr) = arg.as_expression() {
                check_expression_for_issues(
                    expr,
                    _source_code,
                    _named_source,
                    _lint_errors,
                    _lint_config,
                );
            }
        }
    }
}

/// Helper function to check statements for expressions that need linting
fn check_statement_for_expressions(
    stmt: &Statement,
    source_code: &str,
    named_source: &NamedSource<String>,
    lint_errors: &mut Vec<LintError>,
    lint_config: &LintConfig,
) {
    use oxc_ast::ast::Statement;

    match stmt {
        Statement::ExpressionStatement(expr_stmt) => {
            check_expression_for_issues(
                &expr_stmt.expression,
                source_code,
                named_source,
                lint_errors,
                lint_config,
            );
        }
        Statement::VariableDeclaration(var_decl) => {
            for declarator in &var_decl.declarations {
                if let Some(init) = &declarator.init {
                    check_expression_for_issues(
                        init,
                        source_code,
                        named_source,
                        lint_errors,
                        lint_config,
                    );
                }
            }
        }
        Statement::IfStatement(if_stmt) => {
            check_expression_for_issues(
                &if_stmt.test,
                source_code,
                named_source,
                lint_errors,
                lint_config,
            );
            check_statement_for_expressions(
                &if_stmt.consequent,
                source_code,
                named_source,
                lint_errors,
                lint_config,
            );
            if let Some(alternate) = &if_stmt.alternate {
                check_statement_for_expressions(
                    alternate,
                    source_code,
                    named_source,
                    lint_errors,
                    lint_config,
                );
            }
        }
        Statement::BlockStatement(block) => {
            for stmt in &block.body {
                check_statement_for_expressions(
                    stmt,
                    source_code,
                    named_source,
                    lint_errors,
                    lint_config,
                );
            }
        }
        Statement::ReturnStatement(ret_stmt) => {
            if let Some(arg) = &ret_stmt.argument {
                check_expression_for_issues(
                    arg,
                    source_code,
                    named_source,
                    lint_errors,
                    lint_config,
                );
            }
        }
        // Add more statement types as needed
        _ => {}
    }
}

/// Helper function to check for variables that could be const
fn check_prefer_const(
    statements: &[Statement],
    source_code: &str,
    named_source: &NamedSource<String>,
    lint_errors: &mut Vec<LintError>,
    lint_config: &LintConfig,
) {
    let mut let_variables = std::collections::HashSet::new();
    for stmt in statements {
        collect_let_variables(stmt, &mut let_variables);
    }

    let mut reassigned_variables = HashSet::new();
    for stmt in statements {
        check_for_reassignments(stmt, &mut reassigned_variables);
    }

    for stmt in statements {
        report_prefer_const_violations(
            stmt,
            &let_variables,
            &reassigned_variables,
            source_code,
            named_source,
            lint_errors,
            lint_config,
        );
    }
}

/// Recursively collect all let variable declarations
fn collect_let_variables(stmt: &Statement, let_variables: &mut HashSet<String>) {
    use oxc_ast::ast::{Statement, VariableDeclarationKind};

    match stmt {
        Statement::VariableDeclaration(decl) => {
            if matches!(decl.kind, VariableDeclarationKind::Let) {
                for declarator in &decl.declarations {
                    if let Some(id) = declarator.id.get_binding_identifier() {
                        let_variables.insert(id.name.to_string());
                    }
                }
            }
        }
        Statement::BlockStatement(block) => {
            for stmt in &block.body {
                collect_let_variables(stmt, let_variables);
            }
        }
        Statement::IfStatement(if_stmt) => {
            collect_let_variables(&if_stmt.consequent, let_variables);
            if let Some(alternate) = &if_stmt.alternate {
                collect_let_variables(alternate, let_variables);
            }
        }
        Statement::SwitchStatement(switch_stmt) => {
            for case in &switch_stmt.cases {
                for stmt in &case.consequent {
                    collect_let_variables(stmt, let_variables);
                }
            }
        }
        Statement::ForStatement(for_stmt) => {
            if let Some(oxc_ast::ast::ForStatementInit::VariableDeclaration(decl)) = &for_stmt.init
            {
                if matches!(decl.kind, VariableDeclarationKind::Let) {
                    for declarator in &decl.declarations {
                        if let Some(id) = declarator.id.get_binding_identifier() {
                            let_variables.insert(id.name.to_string());
                        }
                    }
                }
            }
            collect_let_variables(&for_stmt.body, let_variables);
        }
        Statement::WhileStatement(while_stmt) => {
            collect_let_variables(&while_stmt.body, let_variables);
        }
        Statement::FunctionDeclaration(func) => {
            if let Some(body) = &func.body {
                for stmt in &body.statements {
                    collect_let_variables(stmt, let_variables);
                }
            }
        }
        _ => {}
    }
}

/// Recursively check for reassignments to variables
fn check_for_reassignments(stmt: &Statement, reassigned_variables: &mut HashSet<String>) {
    match stmt {
        Statement::ExpressionStatement(expr_stmt) => {
            check_expression_for_reassignments(&expr_stmt.expression, reassigned_variables);
        }
        Statement::BlockStatement(block) => {
            for stmt in &block.body {
                check_for_reassignments(stmt, reassigned_variables);
            }
        }
        Statement::IfStatement(if_stmt) => {
            check_for_reassignments(&if_stmt.consequent, reassigned_variables);
            if let Some(alternate) = &if_stmt.alternate {
                check_for_reassignments(alternate, reassigned_variables);
            }
        }
        Statement::SwitchStatement(switch_stmt) => {
            for case in &switch_stmt.cases {
                for stmt in &case.consequent {
                    check_for_reassignments(stmt, reassigned_variables);
                }
            }
        }
        Statement::ForStatement(for_stmt) => {
            check_for_reassignments(&for_stmt.body, reassigned_variables);
        }
        Statement::WhileStatement(while_stmt) => {
            check_for_reassignments(&while_stmt.body, reassigned_variables);
        }
        Statement::FunctionDeclaration(func) => {
            if let Some(body) = &func.body {
                for stmt in &body.statements {
                    check_for_reassignments(stmt, reassigned_variables);
                }
            }
        }
        _ => {}
    }
}

/// Check expressions for variable reassignments
fn check_expression_for_reassignments(
    expr: &oxc_ast::ast::Expression,
    reassigned_variables: &mut HashSet<String>,
) {
    use oxc_ast::ast::{AssignmentTarget, Expression};

    match expr {
        Expression::AssignmentExpression(assign) => {
            if let AssignmentTarget::AssignmentTargetIdentifier(id) = &assign.left {
                reassigned_variables.insert(id.name.to_string());
            }
        }
        Expression::UpdateExpression(update) => {
            if let oxc_ast::ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(id) =
                &update.argument
            {
                reassigned_variables.insert(id.name.to_string());
            }
        }
        Expression::CallExpression(call) => {
            for arg in &call.arguments {
                if let Some(expr) = arg.as_expression() {
                    check_expression_for_reassignments(expr, reassigned_variables);
                }
            }
        }
        _ => {}
    }
}

/// Report prefer-const violations
fn report_prefer_const_violations(
    stmt: &Statement,
    let_variables: &HashSet<String>,
    reassigned_variables: &HashSet<String>,
    _source_code: &str,
    named_source: &NamedSource<String>,
    lint_errors: &mut Vec<LintError>,
    lint_config: &LintConfig,
) {
    use oxc_ast::ast::{Statement, VariableDeclarationKind};

    match stmt {
        Statement::VariableDeclaration(decl) => {
            if matches!(decl.kind, VariableDeclarationKind::Let) {
                for declarator in &decl.declarations {
                    if let Some(id) = declarator.id.get_binding_identifier() {
                        let var_name = id.name.to_string();
                        if let_variables.contains(&var_name)
                            && !reassigned_variables.contains(&var_name)
                            && is_rule_enabled("prefer_const", lint_config)
                        {
                            let span = SourceSpan::new(
                                (id.span.start as usize).into(),
                                id.span.size() as usize,
                            );

                            lint_errors.push(LintError::PreferConst {
                                span,
                                source_code: named_source.clone(),
                                variable_name: var_name,
                            });
                        }
                    }
                }
            }
        }
        Statement::BlockStatement(block) => {
            for stmt in &block.body {
                report_prefer_const_violations(
                    stmt,
                    let_variables,
                    reassigned_variables,
                    _source_code,
                    named_source,
                    lint_errors,
                    lint_config,
                );
            }
        }
        Statement::IfStatement(if_stmt) => {
            report_prefer_const_violations(
                &if_stmt.consequent,
                let_variables,
                reassigned_variables,
                _source_code,
                named_source,
                lint_errors,
                lint_config,
            );
            if let Some(alternate) = &if_stmt.alternate {
                report_prefer_const_violations(
                    alternate,
                    let_variables,
                    reassigned_variables,
                    _source_code,
                    named_source,
                    lint_errors,
                    lint_config,
                );
            }
        }
        Statement::SwitchStatement(switch_stmt) => {
            for case in &switch_stmt.cases {
                for stmt in &case.consequent {
                    report_prefer_const_violations(
                        stmt,
                        let_variables,
                        reassigned_variables,
                        _source_code,
                        named_source,
                        lint_errors,
                        lint_config,
                    );
                }
            }
        }
        Statement::FunctionDeclaration(func) => {
            if let Some(body) = &func.body {
                for stmt in &body.statements {
                    report_prefer_const_violations(
                        stmt,
                        let_variables,
                        reassigned_variables,
                        _source_code,
                        named_source,
                        lint_errors,
                        lint_config,
                    );
                }
            }
        }
        _ => {}
    }
}

/// Lint a single JS/TS file with configuration
#[allow(clippy::result_large_err)]
pub fn lint_file_with_config(
    path: &PathBuf,
    config_override: Option<AndromedaConfig>,
) -> Result<()> {
    let content =
        fs::read_to_string(path).map_err(|e| AndromedaError::file_read_error(path.clone(), e))?;

    // Load configuration
    let config = config_override.unwrap_or_else(|| ConfigManager::load_or_default(None));

    match lint_file_content_with_config(path, &content, Some(config.clone())) {
        Ok(lint_errors) => {
            display_lint_results_with_config(path, &lint_errors, Some(&config));
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Lint file content directly with configuration (useful for LSP)
#[allow(clippy::result_large_err)]
pub fn lint_file_content_with_config(
    path: &PathBuf,
    content: &str,
    config_override: Option<AndromedaConfig>,
) -> Result<Vec<LintError>> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap_or_default();
    let ret = Parser::new(&allocator, content, source_type).parse();
    let program = &ret.program;
    let mut lint_errors = Vec::new();

    // Load configuration
    let config = config_override.unwrap_or_else(|| ConfigManager::load_or_default(None));
    let lint_config = &config.lint;

    let source_name = path.display().to_string();
    let named_source = NamedSource::new(source_name.clone(), content.to_string());

    for stmt in &program.body {
        check_statement_for_expressions(
            stmt,
            content,
            &named_source,
            &mut lint_errors,
            lint_config,
        );

        match stmt {
            Statement::EmptyStatement(empty_stmt) => {
                if is_rule_enabled("empty_statement", lint_config) {
                    let span = SourceSpan::new(
                        (empty_stmt.span().start as usize).into(),
                        empty_stmt.span().size() as usize,
                    );
                    lint_errors.push(LintError::EmptyStatement {
                        span,
                        source_code: named_source.clone(),
                    });
                }
            }
            Statement::VariableDeclaration(decl) => {
                if decl.kind.is_var() && is_rule_enabled("var_usage", lint_config) {
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
                    if body.statements.is_empty() && is_rule_enabled("empty_function", lint_config)
                    {
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

    check_prefer_const(
        &program.body,
        content,
        &named_source,
        &mut lint_errors,
        lint_config,
    );

    let semantic = SemanticBuilder::new().build(program);
    let scoping = semantic.semantic.scoping();
    for symbol_id in scoping.symbol_ids() {
        let flags = scoping.symbol_flags(symbol_id);
        let name = scoping.symbol_name(symbol_id);
        let symbol_span = scoping.symbol_span(symbol_id);

        if flags.intersects(
            SymbolFlags::BlockScopedVariable
                | SymbolFlags::ConstVariable
                | SymbolFlags::FunctionScopedVariable,
        ) && scoping.symbol_is_unused(symbol_id)
            && !name.starts_with('_')
            && is_rule_enabled("unused_variable", lint_config)
        {
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

    Ok(lint_errors)
}

/// Display lint results to the console with configuration
fn display_lint_results_with_config(
    path: &Path,
    lint_errors: &[LintError],
    config_override: Option<&AndromedaConfig>,
) {
    if !lint_errors.is_empty() {
        // Load configuration to check max_warnings
        let default_config = ConfigManager::load_or_default(None);
        let config = config_override.unwrap_or(&default_config);
        let max_warnings = config.lint.max_warnings.unwrap_or(0);

        // Limit displayed errors if max_warnings is set
        let errors_to_show = if max_warnings > 0 && lint_errors.len() > max_warnings as usize {
            &lint_errors[..max_warnings as usize]
        } else {
            lint_errors
        };

        let truncated_msg = if errors_to_show.len() < lint_errors.len() {
            format!(", showing first {}", errors_to_show.len())
        } else {
            String::new()
        };

        println!();
        println!(
            "{} {} ({} issue{} found{})",
            "🔍".bright_yellow(),
            "Lint Issues".bright_yellow().bold(),
            lint_errors.len(),
            if lint_errors.len() == 1 { "" } else { "s" },
            truncated_msg.bright_yellow()
        );
        println!("{}", "─".repeat(60).yellow());

        for (i, error) in errors_to_show.iter().enumerate() {
            if errors_to_show.len() > 1 {
                println!();
                println!(
                    "{} Issue {} of {}:",
                    "📍".cyan(),
                    (i + 1).to_string().bright_cyan(),
                    errors_to_show.len().to_string().bright_cyan()
                );
                println!("{}", "─".repeat(30).cyan());
            }
            println!("{:?}", oxc_miette::Report::new(error.clone()));
        }

        if errors_to_show.len() < lint_errors.len() {
            println!();
            println!(
                "{} {} more issue{} not shown (limited by max_warnings setting)",
                "⚠️".bright_yellow(),
                (lint_errors.len() - errors_to_show.len())
                    .to_string()
                    .bright_yellow(),
                if lint_errors.len() - errors_to_show.len() == 1 {
                    ""
                } else {
                    "s"
                }
            );
        }

        println!();
    } else {
        let ok = Style::new().green().bold().apply_to("✔");
        let file = Style::new().cyan().apply_to(path.display());
        let msg = Style::new().white().dim().apply_to("No lint issues found.");
        println!("{ok} {file}: {msg}");
    }
}
