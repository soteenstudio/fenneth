# Arsitektur

Compiler adalah satu crate Rust dengan library (`fenneth`) dan binary baris perintah.

```
source → Lexer → Parser (AST) → Type checker → Generator C99 → .c → gcc/clang → executable
```

Untuk program multi-file, loader berjalan lebih dulu dan menggabungkan semua file yang di-import menjadi satu program. `import "std/..."` dijawab dari pustaka standar yang tertanam di compiler.

## Struktur proyek

```text
fenneth/
├── Cargo.toml
├── src/
│   ├── main.rs        CLI (run / build / emit / check)
│   ├── lib.rs         pipeline compile() dan compile_path()
│   ├── loader.rs      import multi-file, ekstensi .fen/.fenneth, import std/
│   ├── stdlib.rs      tabel pustaka standar tertanam, pembuangan fungsi tak terpakai
│   ├── lexer/         source -> token
│   │   ├── mod.rs     kursor, melewati trivia, tokenize()
│   │   ├── token.rs   TokenKind, Token, describe()
│   │   ├── literals.rs  identifier, angka, string
│   │   └── symbols.rs operator dan tanda baca
│   ├── parser/        token -> AST
│   │   ├── mod.rs     kursor, entri precedence climbing, parse_program()
│   │   ├── items.rs   tipe, fungsi, struct, global, import
│   │   ├── stmt.rs    statement dan blok
│   │   └── expr.rs    ekspresi, presedensi unary/binary, postfix
│   ├── ast.rs         definisi AST
│   ├── checker/       type checker statis
│   │   ├── mod.rs     state Checker, scope, check()
│   │   ├── decl.rs    struct, fungsi, global, main
│   │   ├── stmt.rs    statement, alur kontrol, pemeriksaan return
│   │   ├── expr.rs    ekspresi, operator, indeks, literal struct
│   │   └── builtins.rs  tanda tangan builtin dan pemeriksaan pemanggilan
│   ├── codegen/       AST -> C99
│   │   ├── mod.rs     VERSION, runtime tertanam
│   │   ├── gen.rs     buffer output, indentasi, nama baru
│   │   ├── names.rs   pemetaan tipe/identifier, escape string
│   │   ├── collect.rs  tipe vektor yang perlu dideklarasikan, urutan struct
│   │   ├── program.rs  satu unit translasi utuh
│   │   ├── stmt.rs    statement
│   │   └── expr.rs    ekspresi dan pemanggilan builtin
│   ├── runtime.c      runtime C yang disisipkan ke setiap hasil
│   ├── runtime/       C tambahan yang hanya disisipkan untuk modul yang butuh:
│   │                  fs.c, http.c, zip.c
│   └── diagnostic.rs  posisi dan pesan error
├── stdlib/            modul pustaka standar yang ditulis dengan Fenneth
│                      (math, string, char, array, fs, http, zip)
├── tests/             test integrasi (pipeline.rs, stdlib.rs)
├── scripts/           install.sh (Linux/macOS/Termux), install.ps1 (Windows),
│                      check-docs-i18n.sh (pengecekan kesamaan docs)
├── docs/              situs dokumentasi ini (VitePress, Inggris + Indonesia)
└── .github/workflows/ ci.yml, release.yml
```

## Tahapan

| Tahap | Modul | Catatan |
|---|---|---|
| Pustaka standar | `stdlib.rs`, `stdlib/*.fen` | Modul ditanam dengan `include_str!` dan dimuat loader seperti file biasa. Fungsinya ditandai `from_std`; setelah pemeriksaan tipe, fungsi yang tidak dapat dijangkau fungsi pengguna dibuang. Modul yang `extern fn fen_*`-nya butuh C nyata (`fs`, `http`, `zip`) menyimpannya di `src/runtime/*.c`, yang baru disisipkan code generator bila ada `extern` dari unit itu yang bertahan setelah pruning. |
| Memuat | `loader.rs` | Menyelesaikan `import` relatif terhadap file yang meng-import, mencoba `.fen` lalu `.fenneth`, memuat tiap file sekali, dan menaruh item modul sebelum item file pengimpornya. |
| Lexing | `lexer/` | Menghasilkan token dengan posisi `file:baris:kolom`. Berhenti pada error pertama. |
| Parsing | `parser/` | Membangun AST. Berhenti pada error pertama. |
| Type checking | `checker/` | Menyelesaikan nama dan tipe, menegakkan aturan di dokumentasi ini, dan mengumpulkan semua error. |
| Code generation | `codegen/` | Menghasilkan C99. Array adalah pointer ke struct vektor hasil generate; struct menjadi struct C. |
| Runtime | `runtime.c` | Disertakan dengan `include_str!` dan disalin ke setiap file hasil: panic, pemeriksaan batas, string, I/O berkas. |

## API library

```rust
// Satu sumber mandiri (tanpa `import`):
let c: Result<String, Vec<fenneth::Diagnostic>> = fenneth::compile(source);

// Sebuah file beserta semua yang di-import-nya:
let c = fenneth::compile_path(std::path::Path::new("main.fen"));
```

`fenneth::VERSION` berisi versi compiler (lihat [Rilis](/id/project/releasing)).

## Test

```sh
cargo test --locked
```

`tests/pipeline.rs` berisi test integrasi untuk pipeline kompilasi dan CLI, dan `tests/stdlib.rs` untuk pustaka standar (program kasus tepinya adalah `tests/std/edge.fen`).
