//! C code generator: type-checked AST -> C99 source text.
//!
//! Mapping:
//!   i32 -> int32_t, f64 -> double, bool -> bool, String -> const char*
//!   struct Foo  -> struct S_Foo (by value)
//!   [T]         -> Vec_<T>*     (growable array, reference semantics)
//! User functions are emitted as `fn_<name>`, variables as `v_<name>` and
//! struct fields as `f_<name>`, so Fenneth names can never clash with C
//! keywords or libc. `extern fn` keeps its plain C symbol name.

use std::collections::{BTreeMap, HashSet};

use crate::ast::*;

/// Release builds set `FENNETH_VERSION` (format `0.xxx`, e.g. `0.003`) at
/// compile time; local builds fall back to the version in `Cargo.toml`.
pub const VERSION: &str = match option_env!("FENNETH_VERSION") {
    Some(v) => v,
    None => env!("CARGO_PKG_VERSION"),
};

const RUNTIME: &str = include_str!("runtime.c");

fn mangle(t: &Type) -> String {
    match t {
        Type::I32 => "i32".into(),
        Type::F64 => "f64".into(),
        Type::Bool => "bool".into(),
        Type::Str => "str".into(),
        Type::Void => "void".into(),
        Type::Struct(n) => format!("S_{}", n),
        Type::Array(inner) => format!("arr_{}", mangle(inner)),
    }
}

fn vec_name(elem: &Type) -> String {
    format!("Vec_{}", mangle(elem))
}

fn c_type(t: &Type) -> String {
    match t {
        Type::I32 => "int32_t".into(),
        Type::F64 => "double".into(),
        Type::Bool => "bool".into(),
        Type::Str => "const char*".into(),
        Type::Void => "void".into(),
        Type::Struct(n) => format!("S_{}", n),
        Type::Array(inner) => format!("{}*", vec_name(inner)),
    }
}

/// A C declaration of `name` with type `t`, optionally constant.
fn decl(t: &Type, name: &str, is_const: bool) -> String {
    match (t, is_const) {
        (Type::Str, true) => format!("const char* const {}", name),
        (Type::Array(_), true) => format!("{} const {}", c_type(t), name),
        (_, true) => format!("const {} {}", c_type(t), name),
        (_, false) => format!("{} {}", c_type(t), name),
    }
}

/// Removes one redundant pair of parentheses wrapping the whole expression,
/// so `if ((a == b))` becomes `if (a == b)` (clang warns about the former).
/// String literals are skipped so parentheses inside them are ignored.
fn strip_outer_parens(s: &str) -> &str {
    let b = s.as_bytes();
    if b.len() < 2 || b[0] != b'(' || b[b.len() - 1] != b')' {
        return s;
    }
    let mut depth = 0i32;
    let mut in_str = false;
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if in_str {
            if c == b'\\' {
                i += 1;
            } else if c == b'"' {
                in_str = false;
            }
        } else {
            match c {
                b'"' => in_str = true,
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 && i != b.len() - 1 {
                        return s; // the first '(' closes before the end
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    &s[1..s.len() - 1]
}

fn fn_name(name: &str) -> String {
    format!("fn_{}", name)
}

fn var_name(name: &str) -> String {
    format!("v_{}", name)
}

fn field_name(name: &str) -> String {
    format!("f_{}", name)
}

/// Escapes a string for use inside a C string literal. Non-ASCII bytes are
/// written as octal escapes so UTF-8 text round-trips exactly.
fn c_escape(s: &str) -> String {
    let mut out = String::new();
    let mut prev_octal = false;
    for b in s.bytes() {
        let mut octal = false;
        match b {
            b'\\' => out.push_str("\\\\"),
            b'"' => out.push_str("\\\""),
            b'\n' => out.push_str("\\n"),
            b'\t' => out.push_str("\\t"),
            b'\r' => out.push_str("\\r"),
            b'?' => out.push_str("\\?"),
            0x20..=0x7e => out.push(b as char),
            _ => {
                out.push_str(&format!("\\{:03o}", b));
                octal = true;
            }
        }
        prev_octal = octal;
    }
    let _ = prev_octal; // octal escapes are always 3 digits, so no ambiguity
    out
}

// ---- type collection --------------------------------------------------------

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

fn collect_vecs(p: &Program) -> BTreeMap<String, Type> {
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
fn order_structs(p: &Program) -> Vec<&StructDef> {
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

// ---- generator ----------------------------------------------------------------

pub struct CodeGen {
    out: String,
    indent: usize,
    tmp: usize,
    externs: HashSet<String>,
}

/// Generates a complete C translation unit for a type-checked program.
pub fn generate(program: &Program) -> String {
    let mut g = CodeGen {
        out: String::new(),
        indent: 0,
        tmp: 0,
        externs: program
            .functions
            .iter()
            .filter(|f| f.is_extern)
            .map(|f| f.name.clone())
            .collect(),
    };
    g.program(program);
    g.out
}

impl CodeGen {
    fn line(&mut self, s: &str) {
        for _ in 0..self.indent {
            self.out.push_str("    ");
        }
        self.out.push_str(s);
        self.out.push('\n');
    }

    fn fresh(&mut self, prefix: &str) -> String {
        self.tmp += 1;
        format!("{}{}", prefix, self.tmp)
    }

    fn signature(&self, f: &Function) -> String {
        let params = if f.params.is_empty() {
            "void".to_string()
        } else {
            f.params
                .iter()
                .map(|p| format!("{} {}", c_type(&p.ty), var_name(&p.name)))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let name = if f.is_extern {
            f.name.clone()
        } else {
            fn_name(&f.name)
        };
        format!("{} {}({})", c_type(&f.ret), name, params)
    }

    fn program(&mut self, p: &Program) {
        self.line(&format!(
            "/* Generated by the Fenneth compiler v{} - do not edit. */",
            VERSION
        ));
        for l in RUNTIME.lines() {
            self.line(l);
        }
        self.line("");

        let vecs = collect_vecs(p);
        let structs = order_structs(p);

        // Forward typedefs (arrays are pointers, so these are enough for
        // struct fields and for mutually referencing types).
        for s in &p.structs {
            self.line(&format!("typedef struct S_{0} S_{0};", s.name));
        }
        for name in vecs.keys() {
            self.line(&format!("typedef struct {0} {0};", name));
        }
        self.line("");

        // Structs, dependencies first.
        for s in structs {
            self.line(&format!("struct S_{} {{", s.name));
            self.indent += 1;
            for f in &s.fields {
                self.line(&format!("{} {};", c_type(&f.ty), field_name(&f.name)));
            }
            self.indent -= 1;
            self.line("};");
            self.line("");
        }

        // Growable array types.
        for (name, elem) in &vecs {
            self.vec_type(name, elem);
        }
        self.line("FEN_UNUSED static Vec_str* rt_args(void) {");
        self.line("    Vec_str* v = vec_str_new();");
        self.line("    for (int i = 1; i < g_argc; i++) vec_str_push(v, g_argv[i]);");
        self.line("    return v;");
        self.line("}");
        self.line("");

        // Globals.
        for g in &p.globals {
            let line = format!(
                "FEN_UNUSED static {} = {};",
                decl(&g.ty, &var_name(&g.name), g.is_const),
                self.expr(&g.init)
            );
            self.line(&line);
        }
        if !p.globals.is_empty() {
            self.line("");
        }

        // Prototypes.
        for f in &p.functions {
            let sig = self.signature(f);
            if f.is_extern {
                self.line(&format!("extern {};", sig));
            } else {
                self.line(&format!("{};", sig));
            }
        }
        self.line("");

        for f in &p.functions {
            if f.is_extern {
                continue;
            }
            let sig = self.signature(f);
            self.line(&format!("{} {{", sig));
            self.indent += 1;
            self.stmts(&f.body);
            self.indent -= 1;
            self.line("}");
            self.line("");
        }

        self.line("int main(int argc, char** argv) {");
        self.indent += 1;
        self.line("g_argc = argc;");
        self.line("g_argv = argv;");
        self.line(&format!("return (int){}();", fn_name("main")));
        self.indent -= 1;
        self.line("}");
    }

    fn vec_type(&mut self, name: &str, elem: &Type) {
        let t = c_type(elem);
        self.line(&format!(
            "struct {} {{ {}* data; int32_t len; int32_t cap; }};",
            name, t
        ));
        self.line(&format!(
            "FEN_UNUSED static {n}* {n_l}_new(void) {{ {n}* v = ({n}*)rt_alloc(sizeof({n})); \
             v->data = NULL; v->len = 0; v->cap = 0; return v; }}",
            n = name,
            n_l = name.to_lowercase()
        ));
        self.line(&format!(
            "FEN_UNUSED static void {l}_push({n}* v, {t} x) {{ \
             if (v->len == v->cap) {{ int32_t nc = v->cap ? v->cap * 2 : 4; \
             v->data = ({t}*)rt_realloc(v->data, (size_t)nc * sizeof({t})); v->cap = nc; }} \
             v->data[v->len++] = x; }}",
            n = name,
            l = name.to_lowercase(),
            t = t
        ));
        self.line(&format!(
            "FEN_UNUSED static {t}* {l}_at({n}* v, int32_t i) {{ \
             if (i < 0 || i >= v->len) rt_index_panic(i, v->len); return &v->data[i]; }}",
            n = name,
            l = name.to_lowercase(),
            t = t
        ));
        self.line(&format!(
            "FEN_UNUSED static {t} {l}_pop({n}* v) {{ \
             if (v->len == 0) rt_panic(\"pop from an empty array\"); return v->data[--v->len]; }}",
            n = name,
            l = name.to_lowercase(),
            t = t
        ));
        self.line(&format!(
            "FEN_UNUSED static {n}* {l}_from(int32_t n, {t}* items) {{ \
             {n}* v = {l}_new(); for (int32_t i = 0; i < n; i++) {l}_push(v, items[i]); return v; }}",
            n = name,
            l = name.to_lowercase(),
            t = t
        ));
        self.line("");
    }

    // ---- statements -------------------------------------------------------

    fn stmts(&mut self, stmts: &[Stmt]) {
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

    // ---- expressions ------------------------------------------------------

    fn binary(&self, op: BinOp, operand: &Type, l: &str, r: &str) -> String {
        match (op, operand) {
            (BinOp::Add, Type::Str) => format!("rt_concat({}, {})", l, r),
            (BinOp::Div, Type::I32) => format!("rt_div({}, {})", l, r),
            (BinOp::Rem, Type::I32) => format!("rt_rem({}, {})", l, r),
            (BinOp::Eq, Type::Str) => format!("(strcmp({}, {}) == 0)", l, r),
            (BinOp::Ne, Type::Str) => format!("(strcmp({}, {}) != 0)", l, r),
            _ => format!("({} {} {})", l, op.symbol(), r),
        }
    }

    fn expr(&self, e: &Expr) -> String {
        match &e.kind {
            ExprKind::Int(v) => format!("{}", v),
            ExprKind::Float(v) => format!("{:?}", v),
            ExprKind::Bool(b) => (if *b { "true" } else { "false" }).to_string(),
            ExprKind::Str(s) => format!("\"{}\"", c_escape(s)),
            ExprKind::Ident(n) => var_name(n),
            ExprKind::Unary(op, inner) => match (op, &inner.kind) {
                // INT32_MIN has no positive counterpart in a 32-bit int literal.
                (UnaryOp::Neg, ExprKind::Int(2147483648)) => "(-2147483647 - 1)".to_string(),
                (UnaryOp::Neg, _) => format!("(-{})", self.expr(inner)),
                (UnaryOp::Not, _) => format!("(!{})", self.expr(inner)),
            },
            ExprKind::Binary(op, l, r) => self.binary(*op, &l.ty, &self.expr(l), &self.expr(r)),
            ExprKind::Call(name, args) => self.call(name, args),
            ExprKind::Index(base, idx) => {
                let elem = match &base.ty {
                    Type::Array(el) => (**el).clone(),
                    _ => Type::I32,
                };
                format!(
                    "(*{}_at({}, {}))",
                    vec_name(&elem).to_lowercase(),
                    self.expr(base),
                    self.expr(idx)
                )
            }
            ExprKind::Field(base, name) => format!("{}.{}", self.expr(base), field_name(name)),
            ExprKind::ArrayLit(items) => {
                let elem = match &e.ty {
                    Type::Array(el) => (**el).clone(),
                    _ => Type::I32,
                };
                let vn = vec_name(&elem).to_lowercase();
                if items.is_empty() {
                    format!("{}_new()", vn)
                } else {
                    let list = items
                        .iter()
                        .map(|i| self.expr(i))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!(
                        "{}_from({}, ({}[]){{{}}})",
                        vn,
                        items.len(),
                        c_type(&elem),
                        list
                    )
                }
            }
            ExprKind::StructLit(name, fields) => {
                let list = fields
                    .iter()
                    .map(|(n, v)| format!(".{} = {}", field_name(n), self.expr(v)))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("((S_{}){{{}}})", name, list)
            }
        }
    }

    fn print_call(&self, name: &str, a: &Expr) -> String {
        let s = self.expr(a);
        let (fmt, arg) = match a.ty {
            Type::I32 => ("%d", s),
            Type::F64 => ("%g", s),
            Type::Bool => ("%s", format!("({}) ? \"true\" : \"false\"", s)),
            _ => ("%s", s),
        };
        match name {
            "print" => format!("printf(\"{}\\n\", {})", fmt, arg),
            "write" => format!("printf(\"{}\", {})", fmt, arg),
            _ => format!("fprintf(stderr, \"{}\\n\", {})", fmt, arg),
        }
    }

    fn call(&self, name: &str, args: &[Expr]) -> String {
        let a = |i: usize| self.expr(&args[i]);
        match name {
            "print" | "write" | "eprint" => self.print_call(name, &args[0]),
            "to_f64" => format!("((double)({}))", a(0)),
            "to_i32" => format!("((int32_t)({}))", a(0)),
            "to_str" => match args[0].ty {
                Type::I32 => format!("rt_i32_str({})", a(0)),
                Type::F64 => format!("rt_f64_str({})", a(0)),
                Type::Bool => format!("rt_bool_str({})", a(0)),
                _ => a(0),
            },
            "len" => match args[0].ty {
                Type::Str => format!("((int32_t)strlen({}))", a(0)),
                _ => format!("({})->len", a(0)),
            },
            "push" | "pop" => {
                let elem = match &args[0].ty {
                    Type::Array(el) => (**el).clone(),
                    _ => Type::I32,
                };
                let vn = vec_name(&elem).to_lowercase();
                if name == "push" {
                    format!("{}_push({}, {})", vn, a(0), a(1))
                } else {
                    format!("{}_pop({})", vn, a(0))
                }
            }
            "str_at" => format!("rt_str_at({}, {})", a(0), a(1)),
            "substr" => format!("rt_substr({}, {}, {})", a(0), a(1), a(2)),
            "char_str" => format!("rt_char_str({})", a(0)),
            "str_to_i32" => format!("rt_str_to_i32({})", a(0)),
            "read_file" => format!("rt_read_file({})", a(0)),
            "write_file" => format!("rt_write_file({}, {})", a(0), a(1)),
            "file_exists" => format!("rt_file_exists({})", a(0)),
            "read_line" => "rt_read_line()".to_string(),
            "args" => "rt_args()".to_string(),
            "exit" => format!("exit({})", a(0)),
            "panic" => format!("rt_panic({})", a(0)),
            _ => {
                let list = args
                    .iter()
                    .map(|x| self.expr(x))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", self.callee(name), list)
            }
        }
    }

    /// Extern functions are called by their plain C symbol name; everything
    /// else goes through the `fn_` prefix.
    fn callee(&self, name: &str) -> String {
        if self.externs.contains(name) {
            name.to_string()
        } else {
            fn_name(name)
        }
    }
}
