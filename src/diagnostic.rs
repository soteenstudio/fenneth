//! Source positions and compiler error messages.

use std::fmt;

/// A position in a source file. `file` indexes the list of loaded files
/// (0 for single-source compilation); `line` and `col` are 1-based.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub file: u32,
    pub line: u32,
    pub col: u32,
}

impl Pos {
    pub fn new(line: u32, col: u32) -> Self {
        Pos { file: 0, line, col }
    }

    pub fn in_file(file: u32, line: u32, col: u32) -> Self {
        Pos { file, line, col }
    }
}

/// A single compile error attached to a source position.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub pos: Pos,
    pub message: String,
}

impl Diagnostic {
    pub fn new(pos: Pos, message: impl Into<String>) -> Self {
        Diagnostic {
            pos,
            message: message.into(),
        }
    }

    /// Formats as `file:line:col: error: message`, looking the file name up
    /// in `files`.
    pub fn render(&self, files: &[String]) -> String {
        let name = files
            .get(self.pos.file as usize)
            .map(|s| s.as_str())
            .unwrap_or("<input>");
        format!(
            "{}:{}:{}: error: {}",
            name, self.pos.line, self.pos.col, self.message
        )
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.pos.line, self.pos.col, self.message)
    }
}
