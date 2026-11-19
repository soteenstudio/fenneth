//! Statement translation.

use crate::ast::*;

use super::gen::{strip_outer_parens, CodeGen};
use super::names::{c_type, decl, var_name};

impl CodeGen {
    pub(super) fn stmts(&mut self, stmts: &[Stmt]) {
        for s in stmts {
            self.stmt(s);
        }
    }

    fn block(&mut self, header: &str, body: &[Stmt]) {
        self.line(&format!("{} {{", header));
        self.indent += 1;
        self.stmts(body);
        self.indent -= 1;
        self.line("}");
    }

    fn stmt(&mut self, s: &Stmt) {
        match &s.kind {
            StmtKind::Let {
                name,
                ty,
                init,
                is_const,
            } => {
                let line = format!(
                    "{} = {};",
                    decl(ty, &var_name(name), *is_const),
                    self.expr(init)
                );
                self.line(&line);
            }
            StmtKind::Assign { target, op, value } => self.assign(target, *op, value),
            StmtKind::Expr(e) => {
                let line = format!("{};", self.expr(e));
                self.line(&line);
            }
            StmtKind::Return(None) => self.line("return;"),
            StmtKind::Return(Some(e)) => {
                let line = format!("return {};", self.expr(e));
                self.line(&line);
            }
            StmtKind::If {
                cond,
                then_block,
                else_block,
            } => {
                let c = self.expr(cond);
                let header = format!("if ({})", strip_outer_parens(&c));
                self.block(&header, then_block);
                if let Some(eb) = else_block {
                    self.block("else", eb);
                }
            }
            StmtKind::While { cond, body } => {
                let c = self.expr(cond);
                let header = format!("while ({})", strip_outer_parens(&c));
                self.block(&header, body);
            }
            StmtKind::ForRange {
                var,
                start,
                end,
                body,
            } => {
                let end_var = self.fresh("end_");
                self.line("{");
                self.indent += 1;
                let l = format!("const int32_t {} = {};", end_var, self.expr(end));
                self.line(&l);
                let header = format!(
                    "for (int32_t {v} = {s}; {v} < {e}; {v}++)",
                    v = var_name(var),
                    s = self.expr(start),
                    e = end_var
                );
                self.block(&header, body);
                self.indent -= 1;
                self.line("}");
            }
            StmtKind::ForEach { var, iter, body } => {
                let arr = self.fresh("arr_");
                let idx = self.fresh("i_");
                let elem = match &iter.ty {
                    Type::Array(el) => (**el).clone(),
                    _ => Type::I32,
                };
                self.line("{");
                self.indent += 1;
                let l = format!("{} {} = {};", c_type(&iter.ty), arr, self.expr(iter));
                self.line(&l);
                self.line(&format!(
                    "for (int32_t {i} = 0; {i} < {a}->len; {i}++) {{",
                    i = idx,
                    a = arr
                ));
                self.indent += 1;
                self.line(&format!(
                    "{} {} = {}->data[{}];",
                    c_type(&elem),
                    var_name(var),
                    arr,
                    idx
                ));
                self.stmts(body);
                self.indent -= 1;
                self.line("}");
                self.indent -= 1;
                self.line("}");
            }
            StmtKind::Break => self.line("break;"),
            StmtKind::Continue => self.line("continue;"),
            StmtKind::Block(b) => self.block("", b),
        }
    }

    fn assign(&mut self, target: &Expr, op: Option<BinOp>, value: &Expr) {
        let rhs = self.expr(value);
        match op {
            None => {
                let l = format!("{} = {};", self.expr(target), rhs);
                self.line(&l);
            }
            Some(op) => {
                if matches!(target.kind, ExprKind::Ident(_)) {
                    let t = self.expr(target);
                    let l = format!("{} = {};", t, self.binary(op, &target.ty, &t, &rhs));
                    self.line(&l);
                } else {
                    // Evaluate the target address once.
                    let p = self.fresh("p_");
                    self.line("{");
                    self.indent += 1;
                    let l = format!("{}* {} = &({});", c_type(&target.ty), p, self.expr(target));
                    self.line(&l);
                    let deref = format!("(*{})", p);
                    let l = format!("*{} = {};", p, self.binary(op, &target.ty, &deref, &rhs));
                    self.line(&l);
                    self.indent -= 1;
                    self.line("}");
                }
            }
        }
    }
}
