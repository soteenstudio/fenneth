# Basics

## Program structure

A Fenneth file contains top-level items: `import`, `struct`, global `let` / `const`, and `fn` (including `extern fn`). Execution starts at `main`, which must be declared exactly like this:

```fenneth
fn main() -> i32 {
    return 0;
}
```

`main` takes no parameters (use `args()` for command-line arguments) and must return `i32`. The returned value is the process exit code.

## Comments

```fenneth
// a line comment
/* a block comment */
```

An unterminated block comment is a compile error.

## Identifiers

Identifiers start with an ASCII letter or `_` and continue with ASCII letters, digits or `_`. [Keywords](/language/keywords) and [built-in function names](/language/builtins) cannot be used as names for your own functions.

## Types

| Type | Description | Literals |
|---|---|---|
| `i32` | 32-bit signed integer | `0`, `42`, `-2147483648` |
| `f64` | 64-bit floating point | `1.5`, `0.25` (digits on both sides of the dot) |
| `bool` | Boolean | `true`, `false` |
| `String` | Immutable byte string | `"text"` with escapes `\n \t \r \\ \"` |
| `[T]` | Array of `T`, can be nested (`[[i32]]`) | `[1, 2, 3]`, `[]` |
| `Name` | A user-defined [struct](/language/data) | `Name { field: value }` |

An integer literal that does not fit in `i32` is a compile error. An unknown escape sequence in a string (for example `\q`) is a compile error.

There are **no implicit conversions**. Convert explicitly with `to_f64()` and `to_i32()`:

```fenneth
let n: i32 = 7;
let half: f64 = to_f64(n) / 2.0;   // 3.5
```

## Variables and constants

Both need a **type annotation** and an **initial value**:

```fenneth
let count: i32 = 0;
const PI: f64 = 3.14159;

count = count + 1;     // ok
// PI = 3.0;           // error: cannot assign to constant 'PI'
```

Rules:

- **No shadowing.** A name cannot be declared again while it is still visible from an outer scope, including globals.
- A `const` cannot be assigned to, and its fields cannot be modified either.
- A `for` loop variable cannot be assigned to.
- An empty array literal needs a type annotation: `let xs: [i32] = [];`.

### Globals

Globals are declared at the top level and may only be initialised with a **literal** (a number, a string, `true` / `false`, or a negative number literal):

```fenneth
const LIMIT: i32 = 3;      // ok
let greeting: String = "hi";   // ok
// const N: i32 = 1 + 2;   // error: global initializers must be literal values
```

## Operators

From the lowest to the highest precedence (all binary operators are left-associative):

| Level | Operators | Operand types |
|---|---|---|
| 1 | `\|\|` | `bool` |
| 2 | `&&` | `bool` |
| 3 | `==` `!=` | both sides the same type: `i32`, `f64`, `bool` or `String` |
| 4 | `<` `<=` `>` `>=` | both sides the same numeric type (`i32` or `f64`) |
| 5 | `+` `-` | both sides the same numeric type; `+` also joins two `String`s |
| 6 | `*` `/` | both sides the same numeric type |
| 6 | `%` | `i32` only |
| unary | `-` `!` | `-` on numbers, `!` on `bool` |

Assignment forms: `=`, `+=`, `-=`, `*=`, `/=`, `%=`. A target can be a variable, a struct field (`p.x`) or an array element (`a[i]`). Assignments are statements, not expressions.

Comparing `String` values with `<` or `>` is not supported yet. Dividing by a literal `0` is a compile error; dividing by a zero value at runtime panics.

::: warning Evaluation order
The order in which operands and call arguments are evaluated follows C and is unspecified. Do not write expressions like `f() + g()` that depend on the order of side effects.
:::
