<div align="center">

# Fenneth

**Bahasa pemrograman bertipe statis yang kecil dan dikompilasi ke C.**

[![CI](https://github.com/OWNER/REPO/actions/workflows/ci.yml/badge.svg)](https://github.com/OWNER/REPO/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/OWNER/REPO?label=release)](https://github.com/OWNER/REPO/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

[Dokumentasi](https://fenneth.vercel.app/id/) · [Documentation (English)](https://fenneth.vercel.app) · [Rilis](https://github.com/OWNER/REPO/releases) · [Changelog](CHANGELOG.md)

[English](README.md) | Bahasa Indonesia

</div>

Fenneth punya sintaks modern dan type checker yang melaporkan semua error sekaligus. Program dikompilasi ke **C99**, lalu GCC atau Clang mengubahnya menjadi executable native. Compiler-nya ditulis dalam Rust dan **tanpa dependensi**.

```
.fen / .fenneth → Lexer → Parser → Type checker → Generator C99 → gcc/clang → executable
```

## Fitur

- **Bertipe statis**: `i32`, `f64`, `bool`, `String`, array `[T]`, dan `struct`, tanpa konversi implisit.
- **Keluaran mudah dibaca**: `fenneth emit` mencetak kode C hasil generate.
- **Program multi-file** dengan `import`, dan **FFI** ke C dan Rust dengan `extern fn`.
- **Pemeriksaan keamanan saat runtime**: batas indeks, `pop` kosong, pembagian dengan nol, dan `substr` yang salah berhenti dengan `panic` (exit code 101).
- **Diagnostik yang jelas** dengan `file:baris:kolom`.
- **Mudah dipasang** di Linux, macOS, Windows, dan Termux; installer memverifikasi checksum SHA-256.

## Instalasi

**Linux, macOS, Termux**

```sh
curl -fsSL https://github.com/OWNER/REPO/releases/latest/download/install.sh | sh
```

**Windows (PowerShell)**

```powershell
irm https://github.com/OWNER/REPO/releases/latest/download/install.ps1 | iex
```

`fenneth run` dan `fenneth build` juga membutuhkan compiler C (`gcc`, `clang`, atau `cc`). Detail tiap platform, variabel installer, dan build dari source ada di [panduan instalasi](https://fenneth.vercel.app/id/guide/installation).

## Mulai cepat

Buat `hello.fen`:

```fenneth
struct Point { x: i32, y: i32, }

fn dist2(a: Point, b: Point) -> i32 {
    let dx: i32 = a.x - b.x;
    let dy: i32 = a.y - b.y;
    return dx * dx + dy * dy;
}

fn main() -> i32 {
    let p: Point = Point { x: 3, y: 4 };
    print("jarak kuadrat = " + to_str(dist2(p, Point { x: 0, y: 0 })));
    return 0;
}
```

```sh
fenneth hello.fen            # build lalu jalankan
fenneth build hello.fen -o hello
fenneth emit hello.fen       # cetak kode C hasil generate
fenneth check hello.fen      # type-check saja
```

## Dokumentasi

Dokumentasi lengkap ada di [`docs/`](docs) dan dipublikasikan di Vercel dalam bahasa Inggris dan Indonesia.

| Topik | Indonesia | English |
|---|---|---|
| Instalasi | [Panduan](https://fenneth.vercel.app/id/guide/installation) | [Guide](https://fenneth.vercel.app/guide/installation) |
| Baris perintah | [CLI](https://fenneth.vercel.app/id/guide/cli) | [CLI](https://fenneth.vercel.app/guide/cli) |
| Contoh kode | [Contoh](https://fenneth.vercel.app/id/guide/examples) | [Examples](https://fenneth.vercel.app/guide/examples) |
| Keyword | [Keyword](https://fenneth.vercel.app/id/language/keywords) | [Keywords](https://fenneth.vercel.app/language/keywords) |
| Fungsi bawaan | [Fungsi bawaan](https://fenneth.vercel.app/id/language/builtins) | [Built-ins](https://fenneth.vercel.app/language/builtins) |
| Modul dan FFI | [Modul dan FFI](https://fenneth.vercel.app/id/language/modules-ffi) | [Modules and FFI](https://fenneth.vercel.app/language/modules-ffi) |
| Batasan | [Batasan](https://fenneth.vercel.app/id/language/limitations) | [Limitations](https://fenneth.vercel.app/language/limitations) |

Untuk menjalankan docs secara lokal: `cd docs && npm install && npm run dev`.

## Pengembangan

```sh
cargo build --release      # target/release/fenneth
cargo test --locked
cargo fmt --all -- --check
sh scripts/check-docs-i18n.sh   # setiap halaman docs wajib punya versi Inggris dan Indonesia
```

| Path | Isi |
|---|---|
| `src/` | Compiler: lexer, parser, type checker, generator kode C, runtime C, CLI |
| `tests/` | Test integrasi |
| `scripts/` | `install.sh`, `install.ps1`, pengecekan kesamaan docs |
| `docs/` | Situs dokumentasi (VitePress) |
| `.github/workflows/` | `ci.yml` dan `release.yml` |

## Rilis

Rilis diberi nomor `0.xxx` seperti Luau (`0.003`, `0.004`, ...). Push tag seperti `0.003`, atau jalankan workflow **Release** dari tab Actions, lalu binary Linux (x86_64, aarch64, statis), macOS (Apple Silicon, Intel), dan Windows dibangun dan dipublikasikan bersama checksum dan installer. Lihat [Rilis](https://fenneth.vercel.app/id/project/releasing).

## Lisensi

[MIT](LICENSE)
