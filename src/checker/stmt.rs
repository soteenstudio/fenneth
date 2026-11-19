//! Statements, blocks and the bodies of `if`/`while`/`for`.

use std::collections::HashMap;

use crate::ast::*;
use crate::diagnostic::Pos;

use super::Checker;

/// True if every path through the block ends in a `return`.
pub(super) fn always_returns(block: &Block) -> bool {
    block.iter().any(|s| match &s.kind {
        StmtKind::Return(_) => true,
        StmtKind::If {
            then_block,
            else_block: Some(else_block),
            ..
        } => always_returns(then_block) && always_returns(else_block),
        StmtKind::Block(b) => always_returns(b),
        _ => false,
    })
}

impl Checker {
    pub(super) fn check_function(&mut self, f: &mut Function) {
        self.cur_ret = f.ret.clone();
        self.loop_depth = 0;
        self.scopes.push(HashMap::new());
        for p in &f.params {
            self.declare(&p.name, p.ty.clone(), false, p.pos);
        }
        self.check_stmts(&mut f.body);
        if f.ret != Type::Void && !always_returns(&f.body) {
            self.err(
                f.pos,
                format!(
                    "function '{}' must return a value of type {} on every path",
                    f.name,
                    f.ret.name()
                ),
            );
        }
        self.scopes.pop();
    }

    pub(super) fn check_block(&mut self, block: &mut Block) {
        self.scopes.push(HashMap::new());
        self.check_stmts(block);
        self.scopes.pop();
    }

    pub(super) fn check_stmts(&mut self, stmts: &mut [Stmt]) {
        for s in stmts.iter_mut() {
            self.check_stmt(s);
        }
    }

    fn check_loop_body(&mut self, var: &str, var_ty: Type, pos: Pos, body: &mut Block) {
        self.scopes.push(HashMap::new());
        self.declare(var, var_ty, true, pos);
        self.loop_depth += 1;
        self.check_stmts(body);
        self.loop_depth -= 1;
        self.scopes.pop();
    }

    fn assign_error(&self, target: &Expr) -> Option<String> {
        match &target.kind {
            ExprKind::Ident(n) => match self.lookup(n) {
                Some(v) if v.is_const => Some(format!("cannot assign to constant '{}'", n)),
                _ => None,
            },
            ExprKind::Index(..) => None,
            ExprKind::Field(base, _) => self.field_root_error(base),
            _ => Some("invalid assignment target".to_string()),
        }
    }

    fn field_root_error(&self, e: &Expr) -> Option<String> {
        match &e.kind {
            ExprKind::Ident(n) => match self.lookup(n) {
                Some(v) if v.is_const => Some(format!("cannot modify a field of constant '{}'", n)),
                _ => None,
            },
            ExprKind::Field(b, _) => self.field_root_error(b),
            // Arrays have reference semantics, so element fields are mutable.
            ExprKind::Index(..) => None,
            _ => Some("cannot assign to a field of a temporary value".to_string()),
        }
    }

    fn check_stmt(&mut self, s: &mut Stmt) {
        let pos = s.pos;
        match &mut s.kind {
            StmtKind::Let {
                name,
                ty,
                init,
                is_const,
            } => {
                self.validate_type(&*ty, pos);
                // The initializer is checked before the name is declared.
                if let Some(t) = self.check_expr(init, Some(&*ty)) {
                    if t != *ty {
                        self.err(
                            init.pos,
                            format!(
                                "cannot initialize '{}' of type {} with a value of type {}",
                                name,
                                ty.name(),
                                t.name()
                            ),
                        );
                    }
                }
                self.declare(name, ty.clone(), *is_const, pos);
            }
            StmtKind::Assign { target, op, value } => {
                let tt = self.check_expr(target, None);
                let vt = self.check_expr(value, tt.as_ref());
                if let Some(msg) = self.assign_error(target) {
                    self.err(pos, msg);
                }
                if let (Some(tt), Some(vt)) = (tt, vt) {
                    match op {
                        None => {
                            if tt != vt {
                                self.err(
                                    value.pos,
                                    format!(
                                        "cannot assign a value of type {} to a target of type {}",
                                        vt.name(),
                                        tt.name()
                                    ),
                                );
                            }
                        }
                        Some(op) => {
                            if matches!(op, BinOp::Div | BinOp::Rem) {
                                if let ExprKind::Int(0) = value.kind {
                                    self.err(value.pos, "division by zero");
                                }
                            }
                            if let Some(r) = self.binary_type(*op, &tt, &vt, pos) {
                                if r != tt {
                                    self.err(
                                        pos,
                                        format!(
                                            "operator '{}=' cannot be used on {}",
                                            op.symbol(),
                                            tt.name()
                                        ),
                                    );
                                }
                            }
                        }
                    }
                }
            }
            StmtKind::Expr(e) => {
                self.check_expr(e, None);
            }
            StmtKind::Return(value) => {
                let cur = self.cur_ret.clone();
                match value {
                    None => {
                        if cur != Type::Void {
                            self.err(
                                pos,
                                format!("'return' needs a value of type {}", cur.name()),
                            );
                        }
                    }
                    Some(e) => {
                        let t = self.check_expr(e, Some(&cur));
                        if cur == Type::Void {
                            self.err(
                                pos,
                                "this function returns no value, so 'return' must be bare",
                            );
                        } else if let Some(t) = t {
                            if t != cur {
                                self.err(
                                    e.pos,
                                    format!(
                                        "returning {} from a function declared to return {}",
                                        t.name(),
                                        cur.name()
                                    ),
                                );
                            }
                        }
                    }
                }
            }
            StmtKind::If {
                cond,
                then_block,
                else_block,
            } => {
                self.expect_bool(cond, "if");
                self.check_block(then_block);
                if let Some(b) = else_block {
                    self.check_block(b);
                }
            }
            StmtKind::While { cond, body } => {
                self.expect_bool(cond, "while");
                self.loop_depth += 1;
                self.check_block(body);
                self.loop_depth -= 1;
            }
            StmtKind::ForRange {
                var,
                start,
                end,
                body,
            } => {
                for e in [&mut *start, &mut *end] {
                    if let Some(t) = self.check_expr(e, None) {
                        if t != Type::I32 {
                            self.err(
                                e.pos,
                                format!("range bounds must be i32, found {}", t.name()),
                            );
                        }
                    }
                }
                self.check_loop_body(var, Type::I32, pos, body);
            }
            StmtKind::ForEach { var, iter, body } => {
                let elem = match self.check_expr(iter, None) {
                    Some(Type::Array(el)) => *el,
                    Some(t) => {
                        self.err(
                            iter.pos,
                            format!("can only iterate over arrays or ranges, found {}", t.name()),
                        );
                        Type::I32
                    }
                    None => Type::I32,
                };
                self.check_loop_body(var, elem, pos, body);
            }
            StmtKind::Break => {
                if self.loop_depth == 0 {
                    self.err(pos, "'break' outside of a loop");
                }
            }
            StmtKind::Continue => {
                if self.loop_depth == 0 {
                    self.err(pos, "'continue' outside of a loop");
                }
            }
            StmtKind::Block(b) => self.check_block(b),
        }
    }

    fn expect_bool(&mut self, cond: &mut Expr, what: &str) {
        if let Some(t) = self.check_expr(cond, None) {
            if t != Type::Bool {
                self.err(
                    cond.pos,
                    format!("'{}' condition must be bool, found {}", what, t.name()),
                );
            }
        }
    }
}
