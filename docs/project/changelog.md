# Changelog

## Unreleased

**Standard library**
- New standard library, written in Fenneth and built into the compiler. Load a module with `import "std/<module>";`: `std/math`, `std/string`, `std/char` and `std/array` (see [Standard library](/language/stdlib)).
- Public names are prefixed with the module name (`math_sqrt`, `string_split`, `array_sort_i32`), so a program that does not import a module never sees its names.
- Std functions that a program never reaches are left out of the generated C.
- `import "std/..."` also works with `fenneth::compile()` (a single source with no file).
- `fenneth std` lists the modules and `fenneth std <module>` prints a module's source.
- `fenneth run` / `fenneth build` now link the C math library (`-lm`) on non-Windows systems.
- The C runtime now includes `<math.h>` and the helpers that the std modules call (`fen_*`).

**Tooling**
- Release numbers now use the Luau-style `0.xxx` format (for example `0.003`). The number is baked into `fenneth --version`.
- Release workflow can be started by pushing a tag or by hand, with automatic numbering and a dry-run mode.
- Documentation moved to a bilingual (English and Indonesian) VitePress site for Vercel, and `README.md` was rewritten.

## 0.2.0

Foundations for self-hosting and distribution.

**Language**
- `struct` (by value), arrays `[T]` (by reference, can be nested), struct and array literals.
- `for i in a..b`, `for x in array`, `break`, `continue`.
- Global `let` / `const` with literal values.
- String joining with `+`; string built-ins (`len`, `substr`, `str_at`, `char_str`, `str_to_i32`, `to_str`).
- `push`, `pop`, `len` for arrays.
- I/O: `read_file`, `write_file`, `file_exists`, `read_line`, `args`, `eprint`, `exit`, `panic`.
- Multi-file `import "path";` with cycle detection; `.fen` and `.fenneth` extensions.
- `extern fn` for FFI to C and Rust.
- The literal `-2147483648` is now valid.
- Runtime checks (index, empty `pop`, division by zero, `substr`) with `panic`.

**Tooling**
- CLI: `fenneth <file>`, `run` (with `-- args`), `build`, `emit`, `check`, and the `--link`, `--cc`, `--keep-c` options.
- `install.sh` (Linux/macOS/Termux) and `install.ps1` (Windows) with SHA-256 verification and PATH setup.
- GitHub workflows: CI (Linux/macOS/Windows) and Release (multi-platform binaries).

**Changed**
- The generated C `main` now receives `argc` / `argv`.
- Error messages name the source file (`file:line:column`) for multi-file programs.
