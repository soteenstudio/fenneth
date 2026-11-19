//! Built-in function signatures and call checking.

use crate::ast::*;
use crate::diagnostic::Pos;

use super::Checker;

/// Signatures of built-ins that have a fixed, non-generic type.
pub(super) fn fixed_sig(name: &str) -> Option<(Vec<Type>, Type)> {
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

    pub(super) fn check_call(&mut self, name: &str, args: &mut [Expr], pos: Pos) -> Option<Type> {
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
