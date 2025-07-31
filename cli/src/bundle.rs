use oxc_allocator::Allocator;
use oxc_mangler::MangleOptions;
use oxc_minifier::{CompressOptions, Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

/// Minifies a JavaScript or TypeScript file and writes the result to output.
pub fn bundle(input: &str, output: &str) -> Result<(), Box<dyn std::error::Error>> {
    let source_text = std::fs::read_to_string(input)?;
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(std::path::Path::new(input)).unwrap_or_default();
    let ret = Parser::new(&allocator, &source_text, source_type).parse();
    let mut program = ret.program;
    let options = MinifierOptions {
        mangle: Some(MangleOptions::default()),
        compress: Some(CompressOptions::smallest()),
    };
    let minified = Minifier::new(options).build(&allocator, &mut program);
    let code = oxc_codegen::Codegen::new()
        .with_options(oxc_codegen::CodegenOptions {
            minify: true,
            comments: oxc_codegen::CommentOptions::disabled(),
            ..oxc_codegen::CodegenOptions::default()
        })
        .with_scoping(minified.scoping)
        .build(&program)
        .code;
    std::fs::write(output, code)?;
    Ok(())
}
