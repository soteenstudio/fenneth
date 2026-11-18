//! Module loader: resolves `import "path";` across files and merges every
//! module into a single `Program`.
//!
//! Imports are resolved relative to the importing file. If the path has no
//! extension, `.fen` and then `.fenneth` are tried. Each file is loaded once
//! (cycles are tolerated), and a module's items are placed before those of
//! the file that imports it.
//!
//! Paths that start with `std/` are not files: they name modules of the
//! embedded standard library (see [`crate::stdlib`]).

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::ast::Program;
use crate::diagnostic::{Diagnostic, Pos};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::stdlib;

/// Source file extensions, in lookup order.
pub const EXTENSIONS: [&str; 2] = ["fen", "fenneth"];

/// Compile errors together with the file table needed to render them.
#[derive(Debug)]
pub struct CompileError {
    pub files: Vec<String>,
    pub diagnostics: Vec<Diagnostic>,
}

impl CompileError {
    pub fn render(&self) -> Vec<String> {
        self.diagnostics
            .iter()
            .map(|d| d.render(&self.files))
            .collect()
    }
}

/// Resolves a user-supplied path, adding a `.fen`/`.fenneth` extension when
/// the path has none and the bare name does not exist.
pub fn resolve_path(path: &Path) -> Option<PathBuf> {
    if path.is_file() {
        return Some(path.to_path_buf());
    }
    if path.extension().is_none() {
        for ext in EXTENSIONS {
            let mut s = path.as_os_str().to_os_string();
            s.push(".");
            s.push(ext);
            let cand = PathBuf::from(s);
            if cand.is_file() {
                return Some(cand);
            }
        }
    }
    None
}

struct Loader {
    files: Vec<String>,
    seen: HashSet<PathBuf>,
    seen_std: HashSet<String>,
}

/// Loads the standard library modules imported by a single self-contained
/// source (one that has no file to resolve other imports against). Returns
/// the merged program, with the std modules' items placed first.
pub fn load_with_std(own: Program) -> Result<Program, Vec<Diagnostic>> {
    let mut l = Loader {
        files: vec!["<input>".to_string()],
        seen: HashSet::new(),
        seen_std: HashSet::new(),
    };
    l.merge(own, None)
}

/// Loads `entry` and everything it imports.
pub fn load_program(entry: &Path) -> Result<(Program, Vec<String>), CompileError> {
    let mut l = Loader {
        files: Vec::new(),
        seen: HashSet::new(),
        seen_std: HashSet::new(),
    };
    let resolved = match resolve_path(entry) {
        Some(p) => p,
        None => {
            return Err(CompileError {
                files: vec![entry.display().to_string()],
                diagnostics: vec![Diagnostic::new(
                    Pos::new(1, 1),
                    format!("cannot find '{}'", entry.display()),
                )],
            })
        }
    };
    match l.load(&resolved) {
        Ok(p) => Ok((p, l.files)),
        Err(diagnostics) => Err(CompileError {
            files: l.files,
            diagnostics,
        }),
    }
}

impl Loader {
    fn load(&mut self, path: &Path) -> Result<Program, Vec<Diagnostic>> {
        let canon = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        let file_id = self.files.len() as u32;
        self.files.push(path.display().to_string());
        self.seen.insert(canon);

        let source = fs::read_to_string(path).map_err(|e| {
            vec![Diagnostic::new(
                Pos::in_file(file_id, 1, 1),
                format!("cannot read '{}': {}", path.display(), e),
            )]
        })?;
        let tokens = Lexer::with_file(&source, file_id)
            .tokenize()
            .map_err(|d| vec![d])?;
        let own = Parser::new(tokens).parse_program().map_err(|d| vec![d])?;

        let dir = path.parent().map(Path::to_path_buf).unwrap_or_default();
        self.merge(own, Some(&dir))
    }

    /// Loads one standard library module (and the std modules it imports).
    fn load_std(&mut self, name: &str, source: &str) -> Result<Program, Vec<Diagnostic>> {
        let file_id = self.files.len() as u32;
        self.files.push(format!("{}{}", stdlib::PREFIX, name));
        let tokens = Lexer::with_file(source, file_id)
            .tokenize()
            .map_err(|d| vec![d])?;
        let mut own = Parser::new(tokens).parse_program().map_err(|d| vec![d])?;
        for f in own.functions.iter_mut() {
            f.from_std = true;
        }
        self.merge(own, None)
    }

    /// Resolves the imports of `own` (relative to `dir`, or std-only when
    /// there is no directory) and returns everything as one program.
    fn merge(&mut self, own: Program, dir: Option<&Path>) -> Result<Program, Vec<Diagnostic>> {
        let mut merged = Program::default();
        for imp in &own.imports {
            let dep = if let Some(name) = stdlib::module_of_import(&imp.path) {
                let Some(source) = stdlib::module_source(name) else {
                    return Err(vec![Diagnostic::new(
                        imp.pos,
                        format!(
                            "unknown standard library module \"{}\" (available: {})",
                            imp.path,
                            stdlib::module_names()
                                .iter()
                                .map(|n| format!("{}{}", stdlib::PREFIX, n))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                    )]);
                };
                if !self.seen_std.insert(name.to_string()) {
                    continue;
                }
                self.load_std(name, source)?
            } else {
                let Some(dir) = dir else {
                    return Err(vec![Diagnostic::new(
                        imp.pos,
                        "'import' needs a source file; compile with compile_path() or the CLI \
                         (only \"std/...\" imports work here)",
                    )]);
                };
                let target = match resolve_path(&dir.join(&imp.path)) {
                    Some(t) => t,
                    None => {
                        return Err(vec![Diagnostic::new(
                            imp.pos,
                            format!(
                            "cannot find module \"{}\" (looked for .fen and .fenneth next to '{}')",
                            imp.path,
                            dir.display()
                        ),
                        )])
                    }
                };
                let canon = fs::canonicalize(&target).unwrap_or_else(|_| target.clone());
                if self.seen.contains(&canon) {
                    continue;
                }
                self.load(&target)?
            };
            merged.structs.extend(dep.structs);
            merged.globals.extend(dep.globals);
            merged.functions.extend(dep.functions);
        }
        merged.structs.extend(own.structs);
        merged.globals.extend(own.globals);
        merged.functions.extend(own.functions);
        Ok(merged)
    }
}
