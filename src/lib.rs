//! Fenneth - a small statically typed language that compiles to C.
//!
//! Pipeline: source -> [lexer] -> tokens -> [parser] -> AST
//!           -> [checker] -> typed AST -> [codegen] -> C source.
//! Multi-file programs go through [`loader`] first (`import "file";`).

pub mod ast;
pub mod checker;
pub mod codegen;
pub mod diagnostic;
pub mod lexer;
pub mod loader;
pub mod parser;

pub use codegen::VERSION;
pub use diagnostic::Diagnostic;
pub use loader::CompileError;

use std::path::Path;

/// Compiles a single self-contained Fenneth source text into C source text.
/// `import` is not available here because there is no file to resolve
/// against; use [`compile_path`] for multi-file programs.
pub fn compile(source: &str) -> Result<String, Vec<Diagnostic>> {
    let tokens = lexer::Lexer::new(source).tokenize().map_err(|d| vec![d])?;
    let mut program = parser::Parser::new(tokens)
        .parse_program()
        .map_err(|d| vec![d])?;
    if let Some(imp) = program.imports.first() {
        return Err(vec![Diagnostic::new(
            imp.pos,
            "'import' needs a source file; compile with compile_path() or the CLI",
        )]);
    }
    checker::check(&mut program)?;
    Ok(codegen::generate(&program))
}

/// Compiles the file at `entry` (and everything it imports) into C source.
pub fn compile_path(entry: &Path) -> Result<String, CompileError> {
    let (mut program, files) = loader::load_program(entry)?;
    if let Err(diagnostics) = checker::check(&mut program) {
        return Err(CompileError { files, diagnostics });
    }
    Ok(codegen::generate(&program))
}
