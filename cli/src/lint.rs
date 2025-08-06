// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::config::{AndromedaConfig, ConfigManager, LintConfig};
use crate::error::{AndromedaError, Result};
use console::Style;
use miette as oxc_miette;
use owo_colors::OwoColorize;
use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, Statement};
use oxc_miette::{Diagnostic, NamedSource, SourceSpan};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_semantic::SymbolFlags;
use oxc_span::{GetSpan, SourceType};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Comprehensive lint error types based on Deno's rule set with enhanced diagnostics
#[derive(Diagnostic, Debug, Clone)]
pub enum LintError {
    /// Empty statement found (no-empty)
    #[diagnostic(
        code(andromeda::lint::no_empty),
        help(
            "🔍 Remove unnecessary semicolons that create empty statements.\n💡 Empty statements can make code harder to read and may indicate errors."
        ),
        url("https://docs.deno.com/lint/rules/no-empty")
    )]
    NoEmpty {
        #[label("Empty statement found here")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
    },

    /// Usage of 'var' keyword (no-var)
    #[diagnostic(
        code(andromeda::lint::no_var),
        help(
            "🔍 Replace 'var' with 'let' or 'const' for better scoping.\n💡 'var' has function-level scoping which can lead to unexpected behavior.\n📖 Use 'let' for variables that will be reassigned, 'const' for constants."
        ),
        url("https://docs.deno.com/lint/rules/no-var")
    )]
    NoVar {
        #[label("'var' keyword used here")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
        variable_name: String,
    },

    /// Unused variable (no-unused-vars)
    #[diagnostic(
        code(andromeda::lint::no_unused_vars),
        help(
            "🔍 Remove the unused variable or prefix it with '_' if intentionally unused.\n💡 Unused variables can indicate dead code or typos in variable names.\n🧹 Removing unused variables helps keep code clean and maintainable."
        ),
        url("https://docs.deno.com/lint/rules/no-unused-vars")
    )]
    NoUnusedVars {
        #[label("Unused variable '{variable_name}'")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
        variable_name: String,
    },

    /// Variable could be const (prefer-const)
    #[diagnostic(
        code(andromeda::lint::prefer_const),
        help(
            "🔍 Use 'const' instead of 'let' for variables that are never reassigned.\n💡 'const' prevents accidental reassignment and makes intent clearer.\n📖 Save 'let' for variables that will be modified."
        ),
        url("https://docs.deno.com/lint/rules/prefer-const")
    )]
    PreferConst {
        #[label("Variable '{variable_name}' is never reassigned, use 'const'")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
        variable_name: String,
    },

    /// Console.log usage (no-console)
    #[diagnostic(
        code(andromeda::lint::no_console),
        help(
            "🔍 Remove console statements from production code.\n💡 Console statements should not be left in production code.\n📖 Use proper logging or remove console statements."
        ),
        url("https://docs.deno.com/lint/rules/no-console")
    )]
    NoConsole {
        #[label("Console statement found here")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
        method_name: String,
    },

    /// Debugger statement (no-debugger)
    #[diagnostic(
        code(andromeda::lint::no_debugger),
        help(
            "🔍 Remove debugger statements from production code.\n💡 Debugger statements should not be left in production code.\n🚨 This can cause applications to stop in production."
        ),
        url("https://docs.deno.com/lint/rules/no-debugger")
    )]
    NoDebugger {
        #[label("Debugger statement found here")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
    },

    /// Explicit any type (no-explicit-any)
    #[diagnostic(
        code(andromeda::lint::no_explicit_any),
        help(
            "🔍 Use specific types instead of 'any'.\n💡 The 'any' type defeats the purpose of TypeScript.\n📖 Consider using specific types, union types, or generic constraints."
        ),
        url("https://docs.deno.com/lint/rules/no-explicit-any")
    )]
    NoExplicitAny {
        #[label("Explicit 'any' type used here")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
    },

    /// Missing await in async function (require-await)
    #[diagnostic(
        code(andromeda::lint::require_await),
        help(
            "🔍 Add await keyword or remove async from function.\n💡 Async functions should contain await expressions.\n📖 Functions without await don't need to be async."
        ),
        url("https://docs.deno.com/lint/rules/require-await")
    )]
    RequireAwait {
        #[label("Async function without await")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
        function_name: String,
    },

    /// Use of eval (no-eval)
    #[diagnostic(
        code(andromeda::lint::no_eval),
        help(
            "🔍 Avoid using eval() as it's a security risk.\n💡 eval() can execute arbitrary code and is a security vulnerability.\n🚨 Consider alternative approaches for dynamic code execution."
        ),
        url("https://docs.deno.com/lint/rules/no-eval")
    )]
    NoEval {
        #[label("eval() usage found here")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
    },

    /// Loose equality comparison (eqeqeq)
    #[diagnostic(
        code(andromeda::lint::eqeqeq),
        help(
            "🔍 Use strict equality (=== or !==) instead of loose equality (== or !=).\n💡 Strict equality prevents type coercion bugs.\n📖 Use === and !== for safer comparisons."
        ),
        url("https://docs.deno.com/lint/rules/eqeqeq")
    )]
    Eqeqeq {
        #[label("Use strict equality (=== or !==) instead")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
        operator: String,
    },

    /// Camelcase naming convention (camelcase)
    #[diagnostic(
        code(andromeda::lint::camelcase),
        help(
            "🔍 Use camelCase naming convention.\n💡 Consistent naming improves code readability.\n📖 Use camelCase for variables, functions, and methods."
        ),
        url("https://docs.deno.com/lint/rules/camelcase")
    )]
    Camelcase {
        #[label("Identifier '{name}' is not in camelCase")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
        name: String,
    },

    /// Boolean literal as argument (no-boolean-literal-for-arguments)
    #[diagnostic(
        code(andromeda::lint::no_boolean_literal_for_arguments),
        help(
            "🔍 Avoid passing boolean literals as arguments.\n� Boolean arguments make code harder to understand.\n📖 Consider using named objects or enums instead."
        ),
        url("https://docs.deno.com/lint/rules/no-boolean-literal-for-arguments")
    )]
    NoBooleanLiteralForArguments {
        #[label("Boolean literal passed as argument")]
        span: SourceSpan,
        #[source_code]
        source_code: NamedSource<String>,
        value: bool,
    },
}

impl std::fmt::Display for LintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LintError::NoEmpty { .. } => write!(f, "Empty statement found"),
            LintError::NoVar { variable_name, .. } => {
                write!(f, "Usage of 'var' for variable '{variable_name}'")
            }
            LintError::NoUnusedVars { variable_name, .. } => {
                write!(f, "Unused variable '{variable_name}'")
            }
            LintError::PreferConst { variable_name, .. } => {
                write!(f, "Variable '{variable_name}' could be const")
            }
            LintError::NoConsole { method_name, .. } => {
                write!(f, "Console.{method_name}() usage found")
            }
            LintError::NoDebugger { .. } => write!(f, "Debugger statement found"),
            LintError::NoExplicitAny { .. } => write!(f, "Explicit 'any' type used"),
            LintError::RequireAwait { function_name, .. } => {
                write!(f, "Async function '{function_name}' lacks await")
            }
            LintError::NoEval { .. } => write!(f, "eval() usage found"),
            LintError::Eqeqeq { operator, .. } => {
                write!(f, "Use strict equality instead of '{operator}'")
            }
            LintError::Camelcase { name, .. } => {
                write!(f, "Identifier '{name}' is not in camelCase")
            }
            LintError::NoBooleanLiteralForArguments { value, .. } => {
                write!(f, "Boolean literal '{value}' passed as argument")
            }
        }
    }
}

impl std::error::Error for LintError {}

/// Helper function to check if a lint rule should be applied
/// Rules are enabled if:
/// 1. They are explicitly in the `rules` list, OR
/// 2. They are in the default enabled rules list AND not in the `disabled_rules` list
fn is_rule_enabled(rule_name: &str, lint_config: &LintConfig) -> bool {
    // If disabled_rules contains the rule, it's disabled
    if lint_config.disabled_rules.contains(&rule_name.to_string()) {
        return false;
    }

    // If rules list is not empty, only explicitly listed rules are enabled
    if !lint_config.rules.is_empty() {
        return lint_config.rules.contains(&rule_name.to_string());
    }

    // Default enabled rules when no rules are explicitly configured
    let default_rules = [
        "no-var",
        "no-debugger", 
        // "eqeqeq",
        "prefer-const",
        "no-unused-vars",
        // "camelcase",
        // "no-console"
        "no-boolean-literal-for-arguments",
        "no-explicit-any",
        "require-await",
        "no-eval",
        "no-empty"
    ];

    default_rules.contains(&rule_name)
}

/// Helper function to check expressions for lint issues
fn check_expression_for_issues(
    expr: &oxc_ast::ast::Expression,
    source_code: &str,
    named_source: &NamedSource<String>,
    lint_errors: &mut Vec<LintError>,
    lint_config: &LintConfig,
) {
    match expr {
        Expression::CallExpression(call) => {
            // Check for console usage (no-console)
            if is_rule_enabled("no-console", lint_config) {
                if let Expression::StaticMemberExpression(member) = &call.callee {
                    if let Expression::Identifier(ident) = &member.object {
                        if ident.name == "console" {
                            let span = SourceSpan::new(
                                (call.span.start as usize).into(),
                                call.span.size() as usize,
                            );
                            lint_errors.push(LintError::NoConsole {
                                span,
                                source_code: named_source.clone(),
                                method_name: member.property.name.to_string(),
                            });
                        }
                    }
                }
            }

            // Check for eval usage (no-eval)
            if is_rule_enabled("no-eval", lint_config) {
                if let Expression::Identifier(ident) = &call.callee {
                    if ident.name == "eval" {
                        let span = SourceSpan::new(
                            (call.span.start as usize).into(),
                            call.span.size() as usize,
                        );
                        lint_errors.push(LintError::NoEval {
                            span,
                            source_code: named_source.clone(),
                        });
                    }
                }
            }

            // Check for boolean literals as arguments (no-boolean-literal-for-arguments)
            if is_rule_enabled("no-boolean-literal-for-arguments", lint_config) {
                for arg in &call.arguments {
                    if let Some(Expression::BooleanLiteral(bool_lit)) = arg.as_expression() {
                        let span = SourceSpan::new(
                            (bool_lit.span.start as usize).into(),
                            bool_lit.span.size() as usize,
                        );
                        lint_errors.push(LintError::NoBooleanLiteralForArguments {
                            span,
                            source_code: named_source.clone(),
                            value: bool_lit.value,
                        });
                    }
                }
            }

            // Recursively check arguments
            for arg in &call.arguments {
                if let Some(expr) = arg.as_expression() {
                    check_expression_for_issues(
                        expr,
                        source_code,
                        named_source,
                        lint_errors,
                        lint_config,
                    );
                }
            }
        }
        Expression::BinaryExpression(bin_expr) => {
            // Check for loose equality (eqeqeq)
            if is_rule_enabled("eqeqeq", lint_config) {
                match bin_expr.operator {
                    oxc_ast::ast::BinaryOperator::Equality => {
                        let span = SourceSpan::new(
                            (bin_expr.span.start as usize).into(),
                            bin_expr.span.size() as usize,
                        );
                        lint_errors.push(LintError::Eqeqeq {
                            span,
                            source_code: named_source.clone(),
                            operator: "==".to_string(),
                        });
                    }
                    oxc_ast::ast::BinaryOperator::Inequality => {
                        let span = SourceSpan::new(
                            (bin_expr.span.start as usize).into(),
                            bin_expr.span.size() as usize,
                        );
                        lint_errors.push(LintError::Eqeqeq {
                            span,
                            source_code: named_source.clone(),
                            operator: "!=".to_string(),
                        });
                    }
                    _ => {}
                }
            }
            check_expression_for_issues(
                &bin_expr.left,
                source_code,
                named_source,
                lint_errors,
                lint_config,
            );
            check_expression_for_issues(
                &bin_expr.right,
                source_code,
                named_source,
                lint_errors,
                lint_config,
            );
        }
        Expression::TSTypeAssertion(type_assertion) => {
            // Check for explicit any (no-explicit-any)
            if is_rule_enabled("no-explicit-any", lint_config) {
                if let oxc_ast::ast::TSType::TSAnyKeyword(_) = &type_assertion.type_annotation {
                    let span = SourceSpan::new(
                        (type_assertion.span.start as usize).into(),
                        type_assertion.span.size() as usize,
                    );
                    lint_errors.push(LintError::NoExplicitAny {
                        span,
                        source_code: named_source.clone(),
                    });
                }
            }
            check_expression_for_issues(
                &type_assertion.expression,
                source_code,
                named_source,
                lint_errors,
                lint_config,
            );
        }
        _ => {}
    }
}

/// Helper function to check if an identifier follows camelCase convention
fn is_camel_case(name: &str) -> bool {
    if name.is_empty() {
        return true;
    }

    // Allow leading underscore for intentionally unused variables
    let name = name.strip_prefix('_').unwrap_or(name);

    // First character should be lowercase
    let mut chars = name.chars();
    if let Some(first) = chars.next() {
        if !first.is_ascii_lowercase() {
            return false;
        }
    }

    // No underscores allowed in camelCase (except leading underscore)
    !name.contains('_')
}

/// Helper function to check if a function is async
fn is_async_function(func: &oxc_ast::ast::Function) -> bool {
    func.r#async
}

/// Helper function to check if a function body contains await expressions
fn contains_await_expression(statements: &[oxc_ast::ast::Statement]) -> bool {
    for stmt in statements {
        if contains_await_in_statement(stmt) {
            return true;
        }
    }
    false
}

/// Recursively check if a statement contains await expressions
fn contains_await_in_statement(stmt: &oxc_ast::ast::Statement) -> bool {
    match stmt {
        Statement::ExpressionStatement(expr_stmt) => {
            contains_await_in_expression(&expr_stmt.expression)
        }
        Statement::VariableDeclaration(var_decl) => {
            for declarator in &var_decl.declarations {
                if let Some(init) = &declarator.init {
                    if contains_await_in_expression(init) {
                        return true;
                    }
                }
            }
            false
        }
        Statement::IfStatement(if_stmt) => {
            contains_await_in_expression(&if_stmt.test)
                || contains_await_in_statement(&if_stmt.consequent)
                || if_stmt
                    .alternate
                    .as_ref()
                    .map_or(false, |alt| contains_await_in_statement(alt))
        }
        Statement::BlockStatement(block) => contains_await_expression(&block.body),
        Statement::ReturnStatement(ret_stmt) => ret_stmt
            .argument
            .as_ref()
            .map_or(false, |arg| contains_await_in_expression(arg)),
        _ => false,
    }
}

/// Recursively check if an expression contains await expressions
fn contains_await_in_expression(expr: &oxc_ast::ast::Expression) -> bool {
    match expr {
        Expression::AwaitExpression(_) => true,
        Expression::CallExpression(call) => {
            if contains_await_in_expression(&call.callee) {
                return true;
            }
            for arg in &call.arguments {
                if let Some(expr) = arg.as_expression() {
                    if contains_await_in_expression(expr) {
                        return true;
                    }
                }
            }
            false
        }
        Expression::BinaryExpression(bin_expr) => {
            contains_await_in_expression(&bin_expr.left)
                || contains_await_in_expression(&bin_expr.right)
        }
        _ => false,
    }
}
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
                            && is_rule_enabled("prefer-const", lint_config)
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
                if is_rule_enabled("no-empty", lint_config) {
                    let span = SourceSpan::new(
                        (empty_stmt.span().start as usize).into(),
                        empty_stmt.span().size() as usize,
                    );
                    lint_errors.push(LintError::NoEmpty {
                        span,
                        source_code: named_source.clone(),
                    });
                }
            }
            Statement::VariableDeclaration(decl) => {
                if decl.kind.is_var() && is_rule_enabled("no-var", lint_config) {
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

                    lint_errors.push(LintError::NoVar {
                        span,
                        source_code: named_source.clone(),
                        variable_name,
                    });
                }

                // Check camelCase naming convention
                if is_rule_enabled("camelcase", lint_config) {
                    for declarator in &decl.declarations {
                        if let Some(id) = declarator.id.get_binding_identifier() {
                            if !is_camel_case(&id.name) {
                                let span = SourceSpan::new(
                                    (id.span.start as usize).into(),
                                    id.span.size() as usize,
                                );
                                lint_errors.push(LintError::Camelcase {
                                    span,
                                    source_code: named_source.clone(),
                                    name: id.name.to_string(),
                                });
                            }
                        }
                    }
                }
            }
            Statement::FunctionDeclaration(func) => {
                // Check camelCase for function names
                if is_rule_enabled("camelcase", lint_config) {
                    if let Some(id) = &func.id {
                        if !is_camel_case(&id.name) {
                            let span = SourceSpan::new(
                                (id.span.start as usize).into(),
                                id.span.size() as usize,
                            );
                            lint_errors.push(LintError::Camelcase {
                                span,
                                source_code: named_source.clone(),
                                name: id.name.to_string(),
                            });
                        }
                    }
                }

                // Check require-await for async functions
                if is_rule_enabled("require-await", lint_config) && is_async_function(func) {
                    if let Some(body) = &func.body {
                        if !contains_await_expression(&body.statements) {
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

                            lint_errors.push(LintError::RequireAwait {
                                span,
                                source_code: named_source.clone(),
                                function_name,
                            });
                        }
                    }
                }
            }
            Statement::DebuggerStatement(debugger_stmt) => {
                if is_rule_enabled("no-debugger", lint_config) {
                    let span = SourceSpan::new(
                        (debugger_stmt.span.start as usize).into(),
                        debugger_stmt.span.size() as usize,
                    );
                    lint_errors.push(LintError::NoDebugger {
                        span,
                        source_code: named_source.clone(),
                    });
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
            && is_rule_enabled("no-unused-vars", lint_config)
        {
            let span = SourceSpan::new(
                (symbol_span.start as usize).into(),
                symbol_span.size() as usize,
            );

            lint_errors.push(LintError::NoUnusedVars {
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
