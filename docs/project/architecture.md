# Architecture

The compiler is a single Rust crate with a library (`fenneth`) and a command-line binary.

```
source → Lexer → Parser (AST) → Type checker → C99 code generator → .c → gcc/clang → executable
```

For multi-file programs, the loader runs first and merges every imported file into one program. `import "std/..."` is answered from the standard library that is embedded in the compiler.

## Project layout

```text
fenneth/
├── Cargo.toml
├── src/
│   ├── main.rs        CLI (run / build / emit / check)
│   ├── lib.rs         compile() and compile_path() pipeline
│   ├── loader.rs      multi-file import, .fen/.fenneth extensions, std/ imports
│   ├── stdlib.rs      embedded standard library table, unused-function pruning
│   ├── lexer.rs       source -> tokens
│   ├── parser.rs      tokens -> AST
│   ├── ast.rs         AST definitions
│   ├── checker.rs     static type checker
│   ├── codegen.rs     AST -> C99
│   ├── runtime.c      C runtime inlined into every output
│   └── diagnostic.rs  positions and error messages
├── stdlib/            standard library modules written in Fenneth (math, string, char, array)
├── tests/             integration tests (pipeline.rs, stdlib.rs)
├── scripts/           install.sh (Linux/macOS/Termux), install.ps1 (Windows),
│                      check-docs-i18n.sh (docs parity check)
├── docs/              this documentation site (VitePress, English + Indonesian)
└── .github/workflows/ ci.yml, release.yml
```

## Stages

| Stage | Module | Notes |
|---|---|---|
| Loading | `loader.rs` | Resolves `import` relative to the importing file, tries `.fen` then `.fenneth`, loads each file once, and puts a module's items before its importer's. |
| Standard library | `stdlib.rs`, `stdlib/*.fen` | The modules are embedded with `include_str!` and loaded by the loader like any other file. Their functions are marked `from_std`; after type checking, the ones no user function can reach are removed. |
| Lexing | `lexer.rs` | Produces tokens with `file:line:column` positions. Stops at the first error. |
| Parsing | `parser.rs` | Builds the AST. Stops at the first error. |
| Type checking | `checker.rs` | Resolves names and types, enforces the rules in this documentation, and collects all errors. |
| Code generation | `codegen.rs` | Emits C99. Arrays are pointers to generated vector structs; structs become C structs. |
| Runtime | `runtime.c` | Included with `include_str!` and copied into every generated file: panics, bounds checks, strings, file I/O. |

## Library API

```rust
// Single self-contained source (no `import`):
let c: Result<String, Vec<fenneth::Diagnostic>> = fenneth::compile(source);

// A file and everything it imports:
let c = fenneth::compile_path(std::path::Path::new("main.fen"));
```

`fenneth::VERSION` holds the compiler version (see [Releases](/project/releasing)).

## Tests

```sh
cargo test --locked
```

`tests/pipeline.rs` contains the integration tests for the compile pipeline and the CLI, and `tests/stdlib.rs` the ones for the standard library (its edge-case program is `tests/std/edge.fen`).
