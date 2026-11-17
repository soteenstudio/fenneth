<div align="center">

# Fenneth

**A small, statically typed language that compiles to C.**

[![CI](https://github.com/OWNER/REPO/actions/workflows/ci.yml/badge.svg)](https://github.com/OWNER/REPO/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/OWNER/REPO?label=release)](https://github.com/OWNER/REPO/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

[Documentation](https://fenneth.vercel.app) · [Dokumentasi (Indonesia)](https://fenneth.vercel.app/id/) · [Releases](https://github.com/OWNER/REPO/releases) · [Changelog](CHANGELOG.md)

English | [Bahasa Indonesia](README.id.md)

</div>

Fenneth has a modern syntax and a type checker that reports every error at once. Programs are compiled to **C99**, and GCC or Clang turns that into a native executable. The compiler is written in Rust and has **no dependencies**.

```
.fen / .fenneth → Lexer → Parser → Type checker → C99 generator → gcc/clang → executable
```

## Features

- **Statically typed**: `i32`, `f64`, `bool`, `String`, arrays `[T]` and `struct`, with no implicit conversions.
- **Readable output**: `fenneth emit` prints the generated C.
- **Multi-file programs** with `import`, and **FFI** to C and Rust with `extern fn`.
- **Runtime safety checks**: bounds, empty `pop`, division by zero and bad `substr` stop with `panic` (exit code 101).
- **Clear diagnostics** with `file:line:column`.
- **Easy to install** on Linux, macOS, Windows and Termux; installers verify SHA-256 checksums.

## Install

**Linux, macOS, Termux**

```sh
curl -fsSL https://github.com/OWNER/REPO/releases/latest/download/install.sh | sh
```

**Windows (PowerShell)**

```powershell
irm https://github.com/OWNER/REPO/releases/latest/download/install.ps1 | iex
```

`fenneth run` and `fenneth build` also need a C compiler (`gcc`, `clang` or `cc`). Details for every platform, installer variables and building from source: see the [installation guide](https://fenneth.vercel.app/guide/installation).

## Quick start

Create `hello.fen`:

```fenneth
struct Point { x: i32, y: i32, }

fn dist2(a: Point, b: Point) -> i32 {
    let dx: i32 = a.x - b.x;
    let dy: i32 = a.y - b.y;
    return dx * dx + dy * dy;
}

fn main() -> i32 {
    let p: Point = Point { x: 3, y: 4 };
    print("distance squared = " + to_str(dist2(p, Point { x: 0, y: 0 })));
    return 0;
}
```

```sh
fenneth hello.fen            # build and run
fenneth build hello.fen -o hello
fenneth emit hello.fen       # print the generated C
fenneth check hello.fen      # type-check only
```

## Documentation

The full documentation lives in [`docs/`](docs) and is published on Vercel, in English and Indonesian.

| Topic | English | Indonesia |
|---|---|---|
| Installation | [Guide](https://fenneth.vercel.app/guide/installation) | [Panduan](https://fenneth.vercel.app/id/guide/installation) |
| Command line | [CLI](https://fenneth.vercel.app/guide/cli) | [CLI](https://fenneth.vercel.app/id/guide/cli) |
| Code examples | [Examples](https://fenneth.vercel.app/guide/examples) | [Contoh](https://fenneth.vercel.app/id/guide/examples) |
| Keywords | [Keywords](https://fenneth.vercel.app/language/keywords) | [Keyword](https://fenneth.vercel.app/id/language/keywords) |
| Built-in functions | [Built-ins](https://fenneth.vercel.app/language/builtins) | [Fungsi bawaan](https://fenneth.vercel.app/id/language/builtins) |
| Modules and FFI | [Modules and FFI](https://fenneth.vercel.app/language/modules-ffi) | [Modul dan FFI](https://fenneth.vercel.app/id/language/modules-ffi) |
| Limitations | [Limitations](https://fenneth.vercel.app/language/limitations) | [Batasan](https://fenneth.vercel.app/id/language/limitations) |

To run the docs locally: `cd docs && npm install && npm run dev`.

## Development

```sh
cargo build --release      # target/release/fenneth
cargo test --locked
cargo fmt --all -- --check
sh scripts/check-docs-i18n.sh   # every docs page needs an English and an Indonesian version
```

| Path | Contents |
|---|---|
| `src/` | The compiler: lexer, parser, type checker, C code generator, C runtime, CLI |
| `tests/` | Integration tests |
| `scripts/` | `install.sh`, `install.ps1`, docs parity check |
| `docs/` | Documentation site (VitePress) |
| `.github/workflows/` | `ci.yml` and `release.yml` |

## Releases

Releases are numbered `0.xxx` like Luau (`0.003`, `0.004`, ...). Push a tag such as `0.003`, or run the **Release** workflow from the Actions tab, and it builds Linux (x86_64, aarch64, static), macOS (Apple Silicon, Intel) and Windows binaries and publishes them with checksums and the installers. See [Releases](https://fenneth.vercel.app/project/releasing).

## License

[MIT](LICENSE)
