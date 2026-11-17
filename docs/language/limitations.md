# Limitations and runtime checks

## Runtime checks

These situations stop the program with `panic: ...` on stderr and exit code **101**:

- array index out of bounds
- `pop` on an empty array
- integer division or remainder by zero
- `substr` with an invalid range, `str_at` out of range, `char_str` outside 0..255
- running out of memory
- an explicit `panic(msg)`

`i32` overflow **wraps around** (the C compiler is invoked with `-fwrapv`). If you compile the output of `fenneth emit` yourself, add `-fwrapv` to keep the same behaviour.

## Known limitations

- **Memory is never freed** (no GC and no `free`). Strings produced by `+`, `substr` and arrays only grow. That is fine for short-lived programs such as compilers and tools, but not for long-running processes.
- **Evaluation order** of operands and call arguments follows C and is unspecified. Avoid expressions such as `f() + g()` that depend on the order of side effects.
- **Globals** can only be initialised with literal values.
- **Missing features**: enums, generics, closures, hash maps, and `<` / `>` on `String`.
- **No shadowing**: a variable name cannot be re-declared while it is visible from an outer scope, including globals.
- **Return on every path**: a function with a return type must `return` on all paths. `panic()` and `exit()` do not count.
- **Error reporting**: the compiler stops at the first lexer or parser error; type checker errors are reported together.
- **Strings are bytes**: lengths and indices count bytes, not Unicode characters.
- **`main`** must be `fn main() -> i32` with no parameters.
