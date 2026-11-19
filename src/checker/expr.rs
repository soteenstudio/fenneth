//! Expressions: literals, operators, calls, indexing and struct literals.

use std::collections::HashSet;

use crate::ast::*;
use crate::diagnostic::Pos;

use super::Checker;

impl Checker {
    /// Checks an expression, records its type in `e.ty`, and returns it.
    /// `expected` is a hint used to type empty array literals.
    /// Returns `None` after an error has already been reported.
    pub(super) fn check_expr(&mut self, e: &mut Expr, expected: Option<&Type>) -> Option<Type> {
        let pos = e.pos;
        let ty = match &mut e.kind {
            ExprKind::Int(v) => {
                if *v > i32::MAX as i64 {
                    self.err(pos, format!("integer literal {} does not fit in i32", v));
                    None
                } else {
                    Some(Type::I32)
                }
            }
            ExprKind::Float(_) => Some(Type::F64),
            ExprKind::Bool(_) => Some(Type::Bool),
            ExprKind::Str(_) => Some(Type::Str),
            ExprKind::Ident(name) => match self.lookup(name) {
                Some(v) => Some(v.ty.clone()),
                None => {
                    let msg = format!("undefined variable '{}'", name);
                    self.err(pos, msg);
                    None
                }
            },
            ExprKind::Unary(op, inner) => {
                let op = *op;
                let t = if op == UnaryOp::Neg && matches!(inner.kind, ExprKind::Int(2147483648)) {
                    // `-2147483648` is the one literal that only fits when negated.
                    inner.ty = Type::I32;
                    Type::I32
                } else {
                    self.check_expr(inner, None)?
                };
                match (op, &t) {
                    (UnaryOp::Neg, Type::I32) | (UnaryOp::Neg, Type::F64) => Some(t),
                    (UnaryOp::Not, Type::Bool) => Some(Type::Bool),
                    (UnaryOp::Neg, _) => {
                        self.err(pos, format!("cannot negate a value of type {}", t.name()));
                        None
                    }
                    (UnaryOp::Not, _) => {
                        self.err(pos, format!("'!' needs a bool, found {}", t.name()));
                        None
                    }
                }
            }
            ExprKind::Binary(op, l, r) => {
                let op = *op;
                let lt = self.check_expr(l, None);
                let rt = self.check_expr(r, None);
                match (lt, rt) {
                    (Some(lt), Some(rt)) => {
                        if matches!(op, BinOp::Div | BinOp::Rem) {
                            if let ExprKind::Int(0) = r.kind {
                                self.err(r.pos, "division by zero");
                            }
                        }
                        self.binary_type(op, &lt, &rt, pos)
                    }
                    _ => None,
                }
            }
            ExprKind::Call(name, args) => {
                let name = name.clone();
                self.check_call(&name, args, pos)
            }
            ExprKind::Index(base, idx) => {
                let bt = self.check_expr(base, None);
                let it = self.check_expr(idx, None);
                match bt {
                    Some(Type::Array(el)) => match it {
                        Some(Type::I32) => Some(*el),
                        Some(t) => {
                            self.err(
                                idx.pos,
                                format!("array index must be i32, found {}", t.name()),
                            );
                            None
                        }
                        None => None,
                    },
                    Some(t) => {
                        self.err(pos, format!("cannot index a value of type {}", t.name()));
                        None
                    }
                    None => None,
                }
            }
            ExprKind::Field(base, name) => {
                let bt = self.check_expr(base, None)?;
                match bt {
                    Type::Struct(sn) => {
                        let found = self
                            .structs
                            .get(&sn)
                            .and_then(|fs| fs.iter().find(|(n, _)| n.as_str() == name.as_str()))
                            .map(|(_, t)| t.clone());
                        if found.is_none() {
                            self.err(pos, format!("struct '{}' has no field '{}'", sn, name));
                        }
                        found
                    }
                    other => {
                        self.err(
                            pos,
                            format!(
                                "cannot access field '{}' on a value of type {}",
                                name,
                                other.name()
                            ),
                        );
                        None
                    }
                }
            }
            ExprKind::ArrayLit(items) => {
                let exp_elem = match expected {
                    Some(Type::Array(t)) => Some((**t).clone()),
                    _ => None,
                };
                if items.is_empty() {
                    match exp_elem {
                        Some(t) => Some(Type::Array(Box::new(t))),
                        None => {
                            self.err(
                                pos,
                                "cannot infer the type of an empty array; add a type annotation",
                            );
                            None
                        }
                    }
                } else {
                    let mut elem = exp_elem;
                    let mut ok = true;
                    for it in items.iter_mut() {
                        match self.check_expr(it, elem.as_ref()) {
                            None => ok = false,
                            Some(t) => {
                                if elem.is_none() {
                                    elem = Some(t);
                                } else if elem.as_ref() != Some(&t) {
                                    let want = elem.as_ref().map(|t| t.name()).unwrap_or_default();
                                    self.err(
                                        it.pos,
                                        format!(
                                            "array element has type {} but earlier elements have type {}",
                                            t.name(),
                                            want
                                        ),
                                    );
                                    ok = false;
                                }
                            }
                        }
                    }
                    if ok {
                        elem.map(|t| Type::Array(Box::new(t)))
                    } else {
                        None
                    }
                }
            }
            ExprKind::StructLit(name, fields) => {
                let Some(def) = self.structs.get(name.as_str()).cloned() else {
                    self.err(pos, format!("unknown struct '{}'", name));
                    return None;
                };
                let mut ok = true;
                let mut seen = HashSet::new();
                for (fname, fexpr) in fields.iter_mut() {
                    match def.iter().find(|(n, _)| n.as_str() == fname.as_str()) {
                        None => {
                            self.err(
                                fexpr.pos,
                                format!("struct '{}' has no field '{}'", name, fname),
                            );
                            self.check_expr(fexpr, None);
                            ok = false;
                        }
                        Some((_, ft)) => {
                            if !seen.insert(fname.clone()) {
                                self.err(
                                    fexpr.pos,
                                    format!("field '{}' is given more than once", fname),
                                );
                                ok = false;
                            }
                            match self.check_expr(fexpr, Some(ft)) {
                                Some(t) if t == *ft => {}
                                Some(t) => {
                                    self.err(
                                        fexpr.pos,
                                        format!(
                                            "field '{}' of '{}' has type {}, found {}",
                                            fname,
                                            name,
                                            ft.name(),
                                            t.name()
                                        ),
                                    );
                                    ok = false;
                                }
                                None => ok = false,
                            }
                        }
                    }
                }
                for (n, _) in &def {
                    if !seen.contains(n) {
                        self.err(pos, format!("missing field '{}' in '{}' literal", n, name));
                        ok = false;
                    }
                }
                if ok {
                    Some(Type::Struct(name.clone()))
                } else {
                    None
                }
            }
        };
        e.ty = ty.clone().unwrap_or(Type::Void);
        ty
    }

    pub(super) fn binary_type(&mut self, op: BinOp, l: &Type, r: &Type, pos: Pos) -> Option<Type> {
        let result = match op {
            BinOp::Add => {
                if l == r && (l.is_numeric() || *l == Type::Str) {
                    Some(l.clone())
                } else {
                    None
                }
            }
            BinOp::Sub | BinOp::Mul | BinOp::Div => {
                if l == r && l.is_numeric() {
                    Some(l.clone())
                } else {
                    None
                }
            }
            BinOp::Rem => {
                if *l == Type::I32 && *r == Type::I32 {
                    Some(Type::I32)
                } else {
                    None
                }
            }
            BinOp::Eq | BinOp::Ne => {
                if l == r && matches!(l, Type::I32 | Type::F64 | Type::Bool | Type::Str) {
                    Some(Type::Bool)
                } else {
                    None
                }
            }
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                if l == r && l.is_numeric() {
                    Some(Type::Bool)
                } else {
                    None
                }
            }
            BinOp::And | BinOp::Or => {
                if *l == Type::Bool && *r == Type::Bool {
                    Some(Type::Bool)
                } else {
                    None
                }
            }
        };
        if result.is_none() {
            let hint = if l != r && l.is_numeric() && r.is_numeric() {
                " (use to_f64()/to_i32() to convert explicitly)"
            } else {
                ""
            };
            self.err(
                pos,
                format!(
                    "operator '{}' cannot be applied to {} and {}{}",
                    op.symbol(),
                    l.name(),
                    r.name(),
                    hint
                ),
            );
        }
        result
    }
}
