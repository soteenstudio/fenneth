# Built-in functions

Built-ins are always available and cannot be redefined. Every call is type-checked.

## Output

| Function | Description |
|---|---|
| `print(x)` | Print `x` followed by a newline. `x` is `i32`, `f64`, `bool` or `String`. |
| `write(x)` | Same as `print` but **without** a newline. |
| `eprint(x)` | Same as `print` but to **stderr**. |

## Conversion

| Function | Description |
|---|---|
| `to_str(x)` | `i32`, `f64` or `bool` to `String`. |
| `to_f64(x)` | `i32` to `f64`. |
| `to_i32(x)` | `f64` to `i32`. |

There are no implicit numeric conversions, so use these whenever you mix `i32` and `f64`.

## Strings

| Function | Description |
|---|---|
| `len(s)` | Length of a `String` in bytes (also works on arrays). |
| `str_at(s, i)` | Byte at index `i` as an `i32` (0..255). Panics when out of range. |
| `substr(s, a, b)` | Bytes `[a, b)` as a new `String`. Panics when the range is invalid. |
| `char_str(code)` | One-character `String` for `code` in 0..255. Panics otherwise. |
| `str_to_i32(s)` | Parse a decimal integer. Gives `0` when `s` is not a number. |

## Arrays

| Function | Description |
|---|---|
| `len(a)` | Number of elements. |
| `push(a, v)` | Append `v` (its type must match the array's element type). |
| `pop(a)` | Remove and return the last element. Panics on an empty array. |

## Files and input

| Function | Description |
|---|---|
| `read_file(path)` | Whole file as a `String`. Returns `""` if the file cannot be opened. |
| `write_file(path, s)` | Write `s` to `path`, replacing it. Returns `true` on success. |
| `file_exists(path)` | `true` if the file can be opened for reading. |
| `read_line()` | One line from stdin without the line ending. Returns `""` at end of input. |
| `args()` | The program arguments as `[String]` (see [CLI](/guide/cli)). |

::: tip
`read_line()` returns `""` both for a blank line and for end of input, so a loop like `while len(line) > 0` also stops at the first blank line.
:::

## Program control

| Function | Description |
|---|---|
| `exit(code)` | Stop the program with exit code `code`. |
| `panic(msg)` | Print `panic: msg` to stderr and exit with code `101`. |

Neither `exit` nor `panic` counts as a `return` for the "return on every path" check.
