// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![allow(clippy::result_large_err)]

use crate::error::{BundleStage, CliError, CliResult};
use oxc_allocator::Allocator;
use oxc_mangler::MangleOptions;
use oxc_minifier::{Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer, TypeScriptOptions};
use std::path::{Path, PathBuf};

/// Bundles, transforms, and minifies a JavaScript or TypeScript file.
pub fn bundle(input: &str, output: &str) -> CliResult<()> {
    let input_path = Path::new(input);
    let output_path = Path::new(output);
    let source_text = crate::error::read_file_with_context(input_path)?;
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(input_path).unwrap_or_default();

    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    if !ret.errors.is_empty() {
        eprintln!("Parser errors:");
        let diagnostics: Vec<String> = ret.errors.iter().map(|e| e.to_string()).collect();
        for d in &diagnostics {
            eprintln!("  {d}");
        }
        return Err(CliError::bundle_error(
            PathBuf::from(input),
            BundleStage::Parse,
            format!("Failed to parse {input}"),
            diagnostics,
        ));
    }

    let mut program = ret.program;

    let need_ts_transform = should_transform_typescript(input_path, output_path);

    if need_ts_transform {
        let transform_options = TransformOptions {
            typescript: TypeScriptOptions {
                ..Default::default()
            },
            ..Default::default()
        };

        let semantic_ret = SemanticBuilder::new().build(&program);

        if !semantic_ret.errors.is_empty() {
            eprintln!("Semantic analysis errors:");
            let diagnostics: Vec<String> =
                semantic_ret.errors.iter().map(|e| e.to_string()).collect();
            for d in &diagnostics {
                eprintln!("  {d}");
            }
            return Err(CliError::bundle_error(
                PathBuf::from(input),
                BundleStage::Semantic,
                "Failed semantic analysis",
                diagnostics,
            ));
        }

        let scoping = semantic_ret.semantic.into_scoping();
        let transformer_ret = Transformer::new(&allocator, input_path, &transform_options)
            .build_with_scoping(scoping, &mut program);

        if !transformer_ret.errors.is_empty() {
            eprintln!("Transform errors:");
            let diagnostics: Vec<String> = transformer_ret
                .errors
                .iter()
                .map(|e| e.to_string())
                .collect();
            for d in &diagnostics {
                eprintln!("  {d}");
            }
            return Err(CliError::bundle_error(
                PathBuf::from(input),
                BundleStage::Transform,
                "Failed to transform TypeScript",
                diagnostics,
            ));
        }
    }

    // TODO: get minifier settings from config file
    let options = MinifierOptions {
        mangle: Some(MangleOptions::default()),
        compress: None,
    };
    let minified = Minifier::new(options).minify(&allocator, &mut program);

    let code = oxc_codegen::Codegen::new()
        .with_options(oxc_codegen::CodegenOptions {
            minify: true,
            comments: oxc_codegen::CommentOptions::disabled(),
            ..oxc_codegen::CodegenOptions::default()
        })
        .with_scoping(minified.scoping)
        .build(&program)
        .code;

    std::fs::write(output, code)
        .map_err(|e| CliError::file_read_error(PathBuf::from(output), e))?;

    Ok(())
}

/// Determines if TypeScript transformation is needed based on input/output extensions
fn should_transform_typescript(input_path: &Path, output_path: &Path) -> bool {
    let input_ext = input_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let output_ext = output_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    matches!(
        (input_ext, output_ext),
        ("ts", "js") | ("tsx", "jsx") | ("ts", "jsx") | ("tsx", "js")
    )
}
