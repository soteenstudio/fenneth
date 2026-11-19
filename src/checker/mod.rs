//! Static type checker. Validates the AST and annotates every expression
//! with its type (`Expr::ty`) for the code generator.
//!
//! The work is split by AST level: declarations ([`decl`]), statements
//! ([`stmt`]) and expressions ([`expr`]), with the built-in function
//! signatures in [`builtins`].

mod builtins;
mod decl;
mod expr;
mod stmt;

#[cfg(test)]
mod tests;

use std::collections::HashMap;

use crate::ast::*;
use crate::diagnostic::{Diagnostic, Pos};

/// Names that are built into the language and cannot be redefined.
pub const BUILTINS: [&str; 20] = [
    "print",
    "write",
    "eprint",
    "to_f64",
    "to_i32",
    "to_str",
    "len",
    "push",
    "pop",
    "str_at",
    "substr",
    "char_str",
    "str_to_i32",
    "read_file",
    "write_file",
    "file_exists",
    "read_line",
    "args",
    "exit",
    "panic",
];

pub(super) struct FuncSig {
    pub(super) params: Vec<Type>,
    pub(super) ret: Type,
}

pub(super) struct VarInfo {
    pub(super) ty: Type,
    pub(super) is_const: bool,
}

pub(super) struct Checker {
    pub(super) structs: HashMap<String, Vec<(String, Type)>>,
    pub(super) funcs: HashMap<String, FuncSig>,
    /// `scopes[0]` holds the globals; one more scope per block.
    pub(super) scopes: Vec<HashMap<String, VarInfo>>,
    pub(super) errors: Vec<Diagnostic>,
    pub(super) cur_ret: Type,
    pub(super) loop_depth: u32,
}

/// Type-checks the program in place. Returns every error found.
pub fn check(program: &mut Program) -> Result<(), Vec<Diagnostic>> {
    let mut c = Checker {
        structs: HashMap::new(),
        funcs: HashMap::new(),
        scopes: vec![HashMap::new()],
        errors: Vec::new(),
        cur_ret: Type::Void,
        loop_depth: 0,
    };
    c.collect(program);
    for f in program.functions.iter_mut() {
        if !f.is_extern {
            c.check_function(f);
        }
    }
    if c.errors.is_empty() {
        Ok(())
    } else {
        Err(c.errors)
    }
}

impl Checker {
    pub(super) fn err(&mut self, pos: Pos, msg: impl Into<String>) {
        self.errors.push(Diagnostic::new(pos, msg));
    }

    pub(super) fn declare(&mut self, name: &str, ty: Type, is_const: bool, pos: Pos) {
        // Redeclaring a visible name (including globals) is rejected; this
        // keeps the generated C, which has C scoping rules, exactly
        // equivalent to the Fenneth semantics.
        if self.scopes.iter().any(|s| s.contains_key(name)) {
            self.err(pos, format!("variable '{}' is already declared", name));
            return;
        }
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), VarInfo { ty, is_const });
        }
    }

    pub(super) fn lookup(&self, name: &str) -> Option<&VarInfo> {
        self.scopes.iter().rev().find_map(|s| s.get(name))
    }
}
