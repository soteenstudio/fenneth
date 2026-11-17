# Keywords

Fenneth has 16 reserved words. They cannot be used as variable, function, field or struct names.

| Keyword | Purpose | Example |
|---|---|---|
| `let` | Declare a variable (type and initial value required). | `let n: i32 = 0;` |
| `const` | Declare a constant. | `const MAX: i32 = 10;` |
| `fn` | Declare a function. | `fn add(a: i32, b: i32) -> i32 { ... }` |
| `return` | Leave a function, optionally with a value. | `return a + b;` |
| `if` | Conditional. | `if n > 0 { ... }` |
| `else` | Alternative branch; combine as `else if`. | `else if n == 0 { ... } else { ... }` |
| `while` | Loop while a condition holds. | `while i < 10 { ... }` |
| `for` | Loop over a range or an array. | `for i in 0..3 { ... }` |
| `in` | Separates the loop variable from what is iterated. | `for x in xs { ... }` |
| `break` | Leave the innermost loop. | `break;` |
| `continue` | Skip to the next loop iteration. | `continue;` |
| `true` | Boolean literal. | `let ok: bool = true;` |
| `false` | Boolean literal. | `let ok: bool = false;` |
| `struct` | Declare a struct type. | `struct P { x: i32, }` |
| `extern` | Declare a function implemented outside Fenneth (used with `fn`). | `extern fn f(a: i32) -> i32;` |
| `import` | Load another source file. | `import "geometry";` |

## Reserved type names

`i32`, `f64`, `bool` and `String` are built-in type names. A struct cannot use them as its name.

## Built-in function names

The following names belong to the language and cannot be redefined as your own functions: `print`, `write`, `eprint`, `to_f64`, `to_i32`, `to_str`, `len`, `push`, `pop`, `str_at`, `substr`, `char_str`, `str_to_i32`, `read_file`, `write_file`, `file_exists`, `read_line`, `args`, `exit`, `panic`. See [Built-in functions](/language/builtins).

## Symbols

| Symbol | Meaning |
|---|---|
| `->` | Return type in a function signature |
| `..` | Exclusive range in `for i in a..b` |
| `:` | Type annotation, struct field, struct literal field |
| `;` | Statement terminator |
| `{ }` | Block, struct body, struct literal |
| `[ ]` | Array type, array literal, indexing |
| `.` | Struct field access |
| `//` `/* */` | Comments |
