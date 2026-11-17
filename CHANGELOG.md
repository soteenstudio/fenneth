# Changelog

Releases are numbered `0.xxx` (for example `0.003`). The full, bilingual changelog is also published in the docs.

## Unreleased

**Tooling**
- Release numbers now use the Luau-style `0.xxx` format. The number is baked into `fenneth --version`.
- The Release workflow can be started by pushing a tag or by hand, with automatic numbering and a dry-run mode.
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
