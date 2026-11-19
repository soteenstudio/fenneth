//! Expression and builtin-call translation.

use crate::ast::*;

use super::gen::CodeGen;
use super::names::{c_escape, c_type, field_name, fn_name, var_name, vec_name};

impl CodeGen {
    pub(super) fn binary(&self, op: BinOp, operand: &Type, l: &str, r: &str) -> String {
        match (op, operand) {
            (BinOp::Add, Type::Str) => format!("rt_concat({}, {})", l, r),
            (BinOp::Div, Type::I32) => format!("rt_div({}, {})", l, r),
            (BinOp::Rem, Type::I32) => format!("rt_rem({}, {})", l, r),
            (BinOp::Eq, Type::Str) => format!("(strcmp({}, {}) == 0)", l, r),
            (BinOp::Ne, Type::Str) => format!("(strcmp({}, {}) != 0)", l, r),
            _ => format!("({} {} {})", l, op.symbol(), r),
        }
    }

    pub(super) fn expr(&self, e: &Expr) -> String {
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
