//! Static type checker. Validates the AST and annotates every expression
//! with its type (`Expr::ty`) for the code generator.

use std::collections::{HashMap, HashSet};

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

struct FuncSig {
    params: Vec<Type>,
    ret: Type,
}

struct VarInfo {
    ty: Type,
    is_const: bool,
}

struct Checker {
    structs: HashMap<String, Vec<(String, Type)>>,
    funcs: HashMap<String, FuncSig>,
    /// `scopes[0]` holds the globals; one more scope per block.
    scopes: Vec<HashMap<String, VarInfo>>,
    errors: Vec<Diagnostic>,
    cur_ret: Type,
    loop_depth: u32,
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

/// True if every path through the block ends in a `return`.
fn always_returns(block: &Block) -> bool {
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

fn is_literal(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Int(_) | ExprKind::Float(_) | ExprKind::Bool(_) | ExprKind::Str(_) => true,
        ExprKind::Unary(UnaryOp::Neg, inner) => {
            matches!(inner.kind, ExprKind::Int(_) | ExprKind::Float(_))
        }
        _ => false,
    }
}

/// Signatures of built-ins that have a fixed, non-generic type.
fn fixed_sig(name: &str) -> Option<(Vec<Type>, Type)> {
    use Type::*;
    Some(match name {
        "str_at" => (vec![Str, I32], I32),
        "substr" => (vec![Str, I32, I32], Str),
        "char_str" => (vec![I32], Str),
        "str_to_i32" => (vec![Str], I32),
        "read_file" => (vec![Str], Str),
        "write_file" => (vec![Str, Str], Bool),
        "file_exists" => (vec![Str], Bool),
        "read_line" => (vec![], Str),
        "args" => (vec![], Array(Box::new(Str))),
        "exit" => (vec![I32], Void),
        "panic" => (vec![Str], Void),
        _ => return None,
    })
}

impl Checker {
    fn err(&mut self, pos: Pos, msg: impl Into<String>) {
        self.errors.push(Diagnostic::new(pos, msg));
    }

    // ---- declarations ------------------------------------------------------

    fn collect(&mut self, p: &mut Program) {
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

    fn validate_type(&mut self, t: &Type, pos: Pos) {
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

    fn declare(&mut self, name: &str, ty: Type, is_const: bool, pos: Pos) {
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

    fn lookup(&self, name: &str) -> Option<&VarInfo> {
        self.scopes.iter().rev().find_map(|s| s.get(name))
    }

    // ---- statements --------------------------------------------------------

    fn check_function(&mut self, f: &mut Function) {
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

    fn check_block(&mut self, block: &mut Block) {
        self.scopes.push(HashMap::new());
        self.check_stmts(block);
        self.scopes.pop();
    }

    fn check_stmts(&mut self, stmts: &mut [Stmt]) {
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

    // ---- expressions -------------------------------------------------------

    /// Checks an expression, records its type in `e.ty`, and returns it.
    /// `expected` is a hint used to type empty array literals.
    /// Returns `None` after an error has already been reported.
    fn check_expr(&mut self, e: &mut Expr, expected: Option<&Type>) -> Option<Type> {
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

    fn binary_type(&mut self, op: BinOp, l: &Type, r: &Type, pos: Pos) -> Option<Type> {
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

    /// Checks `args` against `params` (arity and types).
    fn check_args(&mut self, name: &str, args: &mut [Expr], params: &[Type], pos: Pos) -> bool {
        if params.len() != args.len() {
            self.err(
                pos,
                format!(
                    "'{}' expects {} argument(s) but {} were given",
                    name,
                    params.len(),
                    args.len()
                ),
            );
            for a in args.iter_mut() {
                self.check_expr(a, None);
            }
            return false;
        }
        let mut ok = true;
        for (i, (p, a)) in params.iter().zip(args.iter_mut()).enumerate() {
            match self.check_expr(a, Some(p)) {
                None => ok = false,
                Some(t) if t != *p => {
                    self.err(
                        a.pos,
                        format!(
                            "argument {} of '{}' should be {}, found {}",
                            i + 1,
                            name,
                            p.name(),
                            t.name()
                        ),
                    );
                    ok = false;
                }
                Some(_) => {}
            }
        }
        ok
    }

    /// Checks a builtin that takes exactly one argument of any type.
    fn one_arg(&mut self, name: &str, args: &mut [Expr], pos: Pos) -> Option<Type> {
        if args.len() != 1 {
            self.err(pos, format!("'{}' takes exactly 1 argument", name));
            for a in args.iter_mut() {
                self.check_expr(a, None);
            }
            return None;
        }
        self.check_expr(&mut args[0], None)
    }

    fn check_call(&mut self, name: &str, args: &mut [Expr], pos: Pos) -> Option<Type> {
        match name {
            "print" | "write" | "eprint" => {
                let t = self.one_arg(name, args, pos)?;
                if !t.is_printable() {
                    self.err(
                        args[0].pos,
                        format!("cannot print a value of type {}", t.name()),
                    );
                    return None;
                }
                Some(Type::Void)
            }
            "to_f64" => {
                let t = self.one_arg(name, args, pos)?;
                if t.is_numeric() {
                    Some(Type::F64)
                } else {
                    self.err(
                        args[0].pos,
                        format!("'to_f64' cannot convert a value of type {}", t.name()),
                    );
                    None
                }
            }
            "to_i32" => {
                let t = self.one_arg(name, args, pos)?;
                if t.is_numeric() || t == Type::Bool {
                    Some(Type::I32)
                } else {
                    self.err(
                        args[0].pos,
                        format!("'to_i32' cannot convert a value of type {}", t.name()),
                    );
                    None
                }
            }
            "to_str" => {
                let t = self.one_arg(name, args, pos)?;
                if t.is_printable() {
                    Some(Type::Str)
                } else {
                    self.err(
                        args[0].pos,
                        format!("'to_str' cannot convert a value of type {}", t.name()),
                    );
                    None
                }
            }
            "len" => {
                let t = self.one_arg(name, args, pos)?;
                if t == Type::Str || matches!(t, Type::Array(_)) {
                    Some(Type::I32)
                } else {
                    self.err(
                        args[0].pos,
                        format!("'len' needs a String or an array, found {}", t.name()),
                    );
                    None
                }
            }
            "pop" => {
                let t = self.one_arg(name, args, pos)?;
                match t {
                    Type::Array(el) => Some(*el),
                    other => {
                        self.err(
                            args[0].pos,
                            format!("'pop' needs an array, found {}", other.name()),
                        );
                        None
                    }
                }
            }
            "push" => {
                if args.len() != 2 {
                    self.err(pos, "'push' takes 2 arguments: push(array, value)");
                    for a in args.iter_mut() {
                        self.check_expr(a, None);
                    }
                    return None;
                }
                let at = self.check_expr(&mut args[0], None)?;
                let Type::Array(el) = at else {
                    self.err(
                        args[0].pos,
                        format!("'push' needs an array, found {}", at.name()),
                    );
                    return None;
                };
                let vt = self.check_expr(&mut args[1], Some(&*el))?;
                if vt != *el {
                    self.err(
                        args[1].pos,
                        format!("cannot push a {} into an array of {}", vt.name(), el.name()),
                    );
                    return None;
                }
                Some(Type::Void)
            }
            _ => {
                if let Some((params, ret)) = fixed_sig(name) {
                    return if self.check_args(name, args, &params, pos) {
                        Some(ret)
                    } else {
                        None
                    };
                }
                let (params, ret) = match self.funcs.get(name) {
                    Some(sig) => (sig.params.clone(), sig.ret.clone()),
                    None => {
                        self.err(pos, format!("call to undefined function '{}'", name));
                        for a in args.iter_mut() {
                            self.check_expr(a, None);
                        }
                        return None;
                    }
                };
                if self.check_args(name, args, &params, pos) {
                    Some(ret)
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn run(src: &str) -> Result<(), Vec<Diagnostic>> {
        let toks = Lexer::new(src).tokenize().unwrap();
        let mut prog = Parser::new(toks).parse_program().unwrap();
        check(&mut prog)
    }

    fn first_error(src: &str) -> String {
        run(src).unwrap_err()[0].message.clone()
    }

    #[test]
    fn accepts_valid_program() {
        assert!(run("fn main() -> i32 { let x: i32 = 1; print(x); return 0; }").is_ok());
    }

    #[test]
    fn rejects_type_mismatch() {
        let m = first_error("fn main() -> i32 { let x: i32 = \"hi\"; return 0; }");
        assert!(m.contains("cannot initialize"), "{}", m);
    }

    #[test]
    fn rejects_mixed_arithmetic() {
        let m = first_error("fn main() -> i32 { let x: f64 = 1.0 + 2; return 0; }");
        assert!(m.contains("cannot be applied"), "{}", m);
    }

    #[test]
    fn rejects_const_assignment() {
        let m = first_error("fn main() -> i32 { const x: i32 = 1; x = 2; return 0; }");
        assert!(m.contains("constant"), "{}", m);
    }

    #[test]
    fn requires_main() {
        assert!(run("fn other() { }").is_err());
    }

    #[test]
    fn requires_return_on_all_paths() {
        let m = first_error("fn main() -> i32 { if true { return 1; } }");
        assert!(m.contains("every path"), "{}", m);
    }

    #[test]
    fn checks_structs_and_fields() {
        let ok = "struct P { x: i32, y: i32 } \
                  fn main() -> i32 { let p: P = P { x: 1, y: 2 }; p.x = 5; return p.x + p.y; }";
        assert!(run(ok).is_ok());
        let m = first_error(
            "struct P { x: i32 } fn main() -> i32 { let p: P = P { x: 1 }; return p.z; }",
        );
        assert!(m.contains("no field"), "{}", m);
        let m = first_error("struct P { x: i32 } fn main() -> i32 { let p: P = P { }; return 0; }");
        assert!(m.contains("missing field"), "{}", m);
    }

    #[test]
    fn rejects_recursive_struct_by_value() {
        let m = first_error("struct A { a: A } fn main() -> i32 { return 0; }");
        assert!(m.contains("contains itself"), "{}", m);
        // ...but arrays of itself are fine.
        assert!(run("struct A { kids: [A] } fn main() -> i32 { return 0; }").is_ok());
    }

    #[test]
    fn checks_arrays() {
        let ok = "fn main() -> i32 { let v: [i32] = []; push(v, 3); v[0] = 4; \
                  let s: i32 = 0; for x in v { s += x; } return s + len(v); }";
        assert!(run(ok).is_ok());
        let m = first_error("fn main() -> i32 { let v: [i32] = [1, \"a\"]; return 0; }");
        assert!(m.contains("array element"), "{}", m);
    }

    #[test]
    fn extern_signatures_are_ffi_safe() {
        assert!(
            run("extern fn f(a: i32, s: String) -> f64; fn main() -> i32 { return 0; }").is_ok()
        );
        let m = first_error("extern fn f(a: [i32]) -> i32; fn main() -> i32 { return 0; }");
        assert!(m.contains("extern"), "{}", m);
    }

    #[test]
    fn rejects_empty_array_without_context() {
        let m = first_error("fn main() -> i32 { print(len([])); return 0; }");
        assert!(m.contains("empty array"), "{}", m);
    }

    #[test]
    fn rejects_break_outside_loop_and_assign_to_loop_var() {
        let m = first_error("fn main() -> i32 { break; return 0; }");
        assert!(m.contains("outside of a loop"), "{}", m);
        let m = first_error("fn main() -> i32 { for i in 0..3 { i = 1; } return 0; }");
        assert!(m.contains("constant"), "{}", m);
    }

    #[test]
    fn globals_must_be_literals() {
        assert!(run("const N: i32 = 5; fn main() -> i32 { return N; }").is_ok());
        let m = first_error("const N: i32 = 1 + 2; fn main() -> i32 { return N; }");
        assert!(m.contains("literal"), "{}", m);
    }

    #[test]
    fn allows_most_negative_i32_literal() {
        assert!(run("fn main() -> i32 { let x: i32 = -2147483648; return 0; }").is_ok());
        assert!(run("fn main() -> i32 { let x: i32 = 2147483648; return 0; }").is_err());
    }

    #[test]
    fn string_concat_and_builtins() {
        let ok = "fn main() -> i32 { let s: String = \"a\" + to_str(1); \
                  let c: i32 = str_at(s, 0); s += substr(s, 0, 1); return c + len(s); }";
        assert!(run(ok).is_ok());
    }
}
