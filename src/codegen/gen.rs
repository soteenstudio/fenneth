//! Shared bits of the code generator: output buffering and the helper that
//! removes redundant parentheses in conditions.

/// Removes one redundant pair of parentheses wrapping the whole expression,
/// so `if ((a == b))` becomes `if (a == b)` (clang warns about the former).
/// String literals are skipped so parentheses inside them are ignored.
pub(super) fn strip_outer_parens(s: &str) -> &str {
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

pub struct CodeGen {
    pub(super) out: String,
    pub(super) indent: usize,
    pub(super) tmp: usize,
    pub(super) externs: std::collections::HashSet<String>,
}

impl CodeGen {
    pub(super) fn line(&mut self, s: &str) {
        for _ in 0..self.indent {
            self.out.push_str("    ");
        }
        self.out.push_str(s);
        self.out.push('\n');
    }

    pub(super) fn fresh(&mut self, prefix: &str) -> String {
        self.tmp += 1;
        format!("{}{}", prefix, self.tmp)
    }
}
