// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::error::{AndromedaError, Result};
use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_semantic::SymbolFlags;
use oxc_span::SourceType;
use std::fs;
use std::path::PathBuf;

/// Lint a single JS/TS file
pub fn lint_file(path: &PathBuf) -> Result<()> {
    let content =
        fs::read_to_string(path).map_err(|e| AndromedaError::file_read_error(path.clone(), e))?;
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap_or(SourceType::default());
    let ret = Parser::new(&allocator, &content, source_type).parse();
    let program = &ret.program;
    let mut has_issues = false;

    for stmt in &program.body {
        if let Statement::EmptyStatement(_) = stmt {
            println!("Lint warning: Empty statement in {}", path.display());
            has_issues = true;
        }
        if let Statement::VariableDeclaration(decl) = stmt {
            if decl.kind.is_var() {
                println!(
                    "Lint warning: Usage of 'var' in {} at line {}",
                    path.display(),
                    decl.span.start
                );
                has_issues = true;
            }
        }
        if let Statement::FunctionDeclaration(func) = stmt {
            if let Some(body) = &func.body {
                if body.statements.is_empty() {
                    println!(
                        "Lint warning: Function '{}' has an empty body in {} at line {}",
                        func.id
                            .as_ref()
                            .map(|id| id.name.as_str())
                            .unwrap_or("<anonymous>"),
                        path.display(),
                        func.span.start
                    );
                    has_issues = true;
                }
            }
        }
    }

    let semantic = SemanticBuilder::new().build(program);
   
    // Lint for unused variables using oxc_semantic
    let scoping = semantic.semantic.scoping();
    for symbol_id in scoping.symbol_ids() {
        let flags = scoping.symbol_flags(symbol_id);
        if flags.intersects(SymbolFlags::BlockScopedVariable | SymbolFlags::ConstVariable | SymbolFlags::FunctionScopedVariable) {
            if scoping.symbol_is_unused(symbol_id) {
                let name = scoping.symbol_name(symbol_id);
                // Allow unused variables that start with '_'
                if !name.starts_with('_') {
                    let span = scoping.symbol_span(symbol_id);
                    println!(
                        "Lint warning: Unused variable '{}' in {} at line {}",
                        name,
                        path.display(),
                        span.start
                    );
                    has_issues = true;
                }
            }
        }
    }

    if !has_issues {
        println!("{}: No lint issues found.", path.display());
    }
    Ok(())
}
