//! Discovery of the types that need a C declaration: growable arrays and
//! struct ordering.

use std::collections::{BTreeMap, HashSet};

use crate::ast::*;

use super::names::vec_name;

fn add_type(t: &Type, vecs: &mut BTreeMap<String, Type>) {
    if let Type::Array(inner) = t {
        vecs.entry(vec_name(inner))
            .or_insert_with(|| (**inner).clone());
        add_type(inner, vecs);
    }
}

fn walk_expr(e: &Expr, vecs: &mut BTreeMap<String, Type>) {
    add_type(&e.ty, vecs);
    match &e.kind {
        ExprKind::Unary(_, a) => walk_expr(a, vecs),
        ExprKind::Binary(_, a, b) | ExprKind::Index(a, b) => {
            walk_expr(a, vecs);
            walk_expr(b, vecs);
        }
        ExprKind::Call(_, args) | ExprKind::ArrayLit(args) => {
            args.iter().for_each(|a| walk_expr(a, vecs))
        }
        ExprKind::Field(a, _) => walk_expr(a, vecs),
        ExprKind::StructLit(_, fs) => fs.iter().for_each(|(_, a)| walk_expr(a, vecs)),
        _ => {}
    }
}

fn walk_block(b: &Block, vecs: &mut BTreeMap<String, Type>) {
    for s in b {
        match &s.kind {
            StmtKind::Let { ty, init, .. } => {
                add_type(ty, vecs);
                walk_expr(init, vecs);
            }
            StmtKind::Assign { target, value, .. } => {
                walk_expr(target, vecs);
                walk_expr(value, vecs);
            }
            StmtKind::Expr(e) => walk_expr(e, vecs),
            StmtKind::Return(Some(e)) => walk_expr(e, vecs),
            StmtKind::Return(None) | StmtKind::Break | StmtKind::Continue => {}
            StmtKind::If {
                cond,
                then_block,
                else_block,
            } => {
                walk_expr(cond, vecs);
                walk_block(then_block, vecs);
                if let Some(eb) = else_block {
                    walk_block(eb, vecs);
                }
            }
            StmtKind::While { cond, body } => {
                walk_expr(cond, vecs);
                walk_block(body, vecs);
            }
            StmtKind::ForRange {
                start, end, body, ..
            } => {
                walk_expr(start, vecs);
                walk_expr(end, vecs);
                walk_block(body, vecs);
            }
            StmtKind::ForEach { iter, body, .. } => {
                walk_expr(iter, vecs);
                walk_block(body, vecs);
            }
            StmtKind::Block(b) => walk_block(b, vecs),
        }
    }
}

pub(super) fn collect_vecs(p: &Program) -> BTreeMap<String, Type> {
    let mut vecs = BTreeMap::new();
    // `args()` always needs [String].
    add_type(&Type::Array(Box::new(Type::Str)), &mut vecs);
    for s in &p.structs {
        for f in &s.fields {
            add_type(&f.ty, &mut vecs);
        }
    }
    for g in &p.globals {
        add_type(&g.ty, &mut vecs);
    }
    for f in &p.functions {
        for prm in &f.params {
            add_type(&prm.ty, &mut vecs);
        }
        add_type(&f.ret, &mut vecs);
        walk_block(&f.body, &mut vecs);
    }
    vecs
}

/// Orders structs so that every by-value field type is defined first.
pub(super) fn order_structs(p: &Program) -> Vec<&StructDef> {
    fn visit<'a>(
        s: &'a StructDef,
        p: &'a Program,
        done: &mut HashSet<String>,
        out: &mut Vec<&'a StructDef>,
    ) {
        if !done.insert(s.name.clone()) {
            return;
        }
        for f in &s.fields {
            if let Type::Struct(n) = &f.ty {
                if let Some(dep) = p.structs.iter().find(|d| &d.name == n) {
                    visit(dep, p, done, out);
                }
            }
        }
        out.push(s);
    }
    let mut done = HashSet::new();
    let mut out = Vec::new();
    for s in &p.structs {
        visit(s, p, &mut done, &mut out);
    }
    out
}
