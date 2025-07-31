// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::error::{AndromedaError, Result};
use console::Style;
use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_semantic::SymbolFlags;
use oxc_span::SourceType;
use std::fs;
use std::path::PathBuf;

/// Lint a single JS/TS file
#[allow(clippy::result_large_err)]
pub fn lint_file(path: &PathBuf) -> Result<()> {
    let content =
        fs::read_to_string(path).map_err(|e| AndromedaError::file_read_error(path.clone(), e))?;
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap_or_default();
    let ret = Parser::new(&allocator, &content, source_type).parse();
    let program = &ret.program;
    let mut has_issues = false;

    for stmt in &program.body {
        match stmt {
            Statement::EmptyStatement(_) => {
                let warning = Style::new().yellow().bold().apply_to("Lint warning:");
                let msg = Style::new().white().apply_to("Empty statement");
                let file = Style::new().cyan().apply_to(path.display());
                println!("{warning} {msg} in {file}");
                has_issues = true;
            }
            Statement::VariableDeclaration(decl) => {
                if decl.kind.is_var() {
                    let warning = Style::new().yellow().bold().apply_to("Lint warning:");
                    let msg = Style::new().white().apply_to("Usage of 'var'");
                    let file = Style::new().cyan().apply_to(path.display());
                    let line = Style::new().magenta().apply_to(decl.span.start.to_string());
                    println!("{warning} {msg} in {file} at line {line}");
                    has_issues = true;
                }
            }
            Statement::FunctionDeclaration(func) => {
                if let Some(body) = &func.body {
                    if body.statements.is_empty() {
                        let warning = Style::new().yellow().bold().apply_to("Lint warning:");
                        let msg = Style::new().white().apply_to("Function");
                        let func_name = Style::new().green().apply_to(
                            func.id
                                .as_ref()
                                .map(|id| id.name.as_str())
                                .unwrap_or("<anonymous>"),
                        );
                        let file = Style::new().cyan().apply_to(path.display());
                        let line = Style::new().magenta().apply_to(func.span.start.to_string());
                        let empty = Style::new().red().apply_to("has an empty body");
                        println!("{warning} {msg} '{func_name}' {empty} in {file} at line {line}");
                        has_issues = true;
                    }
                }
            }
            _ => {}
        }
    }

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
                let span = scoping.symbol_span(symbol_id);
                let warning = Style::new().yellow().bold().apply_to("Lint warning:");
                let msg = Style::new().white().apply_to("Unused variable");
                let var = Style::new().green().apply_to(name);
                let file = Style::new().cyan().apply_to(path.display());
                let line = Style::new().magenta().apply_to(span.start.to_string());
                println!("{warning} {msg} '{var}' in {file} at line {line}");
                has_issues = true;
            }
        }
    }

    if !has_issues {
        let ok = Style::new().green().bold().apply_to("✔");
        let file = Style::new().cyan().apply_to(path.display());
        let msg = Style::new().white().dim().apply_to("No lint issues found.");
        println!("{ok} {file}: {msg}");
    }
    Ok(())
}
