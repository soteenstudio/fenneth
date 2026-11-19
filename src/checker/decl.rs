//! Top-level declarations: structs, functions, globals and `main`.

use std::collections::HashSet;

use crate::ast::*;
use crate::diagnostic::Pos;

use super::{Checker, FuncSig, BUILTINS};

pub(super) fn is_literal(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Int(_) | ExprKind::Float(_) | ExprKind::Bool(_) | ExprKind::Str(_) => true,
        ExprKind::Unary(UnaryOp::Neg, inner) => {
            matches!(inner.kind, ExprKind::Int(_) | ExprKind::Float(_))
        }
        _ => false,
    }
}

impl Checker {
    pub(super) fn collect(&mut self, p: &mut Program) {
        // Structs
        for s in &p.structs {
            if Type::from_name(&s.name).is_some() {
                self.err(s.pos, format!("'{}' is a built-in type name", s.name));
                continue;
            }
            if !s
                .name
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_uppercase())
            {
                self.err(
                    s.pos,
                    format!(
                        "struct name '{}' must start with an uppercase letter",
                        s.name
                    ),
                );
                continue;
            }
            if self.structs.contains_key(&s.name) {
                self.err(
                    s.pos,
                    format!("struct '{}' is defined more than once", s.name),
                );
                continue;
            }
            if s.fields.is_empty() {
                self.err(
                    s.pos,
                    format!("struct '{}' must have at least one field", s.name),
                );
            }
            let mut seen = HashSet::new();
            let mut fields = Vec::new();
            for f in &s.fields {
                if !seen.insert(f.name.clone()) {
                    self.err(
                        f.pos,
                        format!("duplicate field '{}' in struct '{}'", f.name, s.name),
                    );
                }
                fields.push((f.name.clone(), f.ty.clone()));
            }
            self.structs.insert(s.name.clone(), fields);
        }
        for s in &p.structs {
            for f in &s.fields {
                self.validate_type(&f.ty, f.pos);
            }
        }
        for s in &p.structs {
            let mut visited = HashSet::new();
            if self.contains_by_value(&s.name, &s.name, &mut visited) {
                self.err(
                    s.pos,
                    format!(
                        "struct '{}' contains itself (use an array '[{}]' instead)",
                        s.name, s.name
                    ),
                );
            }
        }

        // Functions
        for f in &p.functions {
            if BUILTINS.contains(&f.name.as_str()) {
                self.err(
                    f.pos,
                    format!(
                        "'{}' is a built-in function and cannot be redefined",
                        f.name
                    ),
                );
                continue;
            }
            if self.funcs.contains_key(&f.name) {
                self.err(
                    f.pos,
                    format!("function '{}' is defined more than once", f.name),
                );
                continue;
            }
            for prm in &f.params {
                self.validate_type(&prm.ty, prm.pos);
            }
            self.validate_type(&f.ret, f.pos);
            if f.is_extern {
                let ffi_ok = |t: &Type| {
                    matches!(
                        t,
                        Type::I32 | Type::F64 | Type::Bool | Type::Str | Type::Void
                    )
                };
                if !f.params.iter().all(|p| ffi_ok(&p.ty)) || !ffi_ok(&f.ret) {
                    self.err(
                        f.pos,
                        format!(
                            "extern function '{}' may only use i32, f64, bool and String types",
                            f.name
                        ),
                    );
                }
            }
            self.funcs.insert(
                f.name.clone(),
                FuncSig {
                    params: f.params.iter().map(|p| p.ty.clone()).collect(),
                    ret: f.ret.clone(),
                },
            );
        }

        // Globals (literal initializers only, so they are valid C constants)
        for g in p.globals.iter_mut() {
            let ty = g.ty.clone();
            self.validate_type(&ty, g.pos);
            if !is_literal(&g.init) {
                self.err(g.init.pos, "global initializers must be literal values");
            }
            if let Some(t) = self.check_expr(&mut g.init, Some(&ty)) {
                if t != ty {
                    self.err(
                        g.init.pos,
                        format!(
                            "cannot initialize '{}' of type {} with a value of type {}",
                            g.name,
                            ty.name(),
                            t.name()
                        ),
                    );
                }
            }
            self.declare(&g.name, ty, g.is_const, g.pos);
        }

        // main
        match p.functions.iter().find(|f| f.name == "main") {
            None => self.err(Pos::new(1, 1), "program has no 'main' function"),
            Some(m) => {
                if m.is_extern {
                    self.err(m.pos, "'main' cannot be extern");
                }
                if !m.params.is_empty() {
                    self.err(
                        m.pos,
                        "'main' must not take parameters (use args() instead)",
                    );
                }
                if m.ret != Type::I32 {
                    self.err(
                        m.pos,
                        "'main' must return i32 (declare it as 'fn main() -> i32')",
                    );
                }
            }
        }
    }

    fn contains_by_value(&self, start: &str, cur: &str, visited: &mut HashSet<String>) -> bool {
        let Some(fields) = self.structs.get(cur) else {
            return false;
        };
        for (_, ty) in fields {
            if let Type::Struct(n) = ty {
                if n == start {
                    return true;
                }
                if visited.insert(n.clone()) && self.contains_by_value(start, n, visited) {
                    return true;
                }
            }
        }
        false
    }

    pub(super) fn validate_type(&mut self, t: &Type, pos: Pos) {
        match t {
            Type::Struct(n) => {
                if !self.structs.contains_key(n) {
                    self.err(
                        pos,
                        format!(
                            "unknown type '{}' (expected i32, f64, bool, String, [T] or a declared struct)",
                            n
                        ),
                    );
                }
            }
            Type::Array(inner) => self.validate_type(inner, pos),
            _ => {}
        }
    }
}
