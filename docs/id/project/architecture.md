# Arsitektur

Compiler adalah satu crate Rust dengan library (`fenneth`) dan binary baris perintah.

```
source → Lexer → Parser (AST) → Type checker → Generator C99 → .c → gcc/clang → executable
```

Untuk program multi-file, loader berjalan lebih dulu dan menggabungkan semua file yang di-import menjadi satu program.

## Struktur proyek

```text
fenneth/
├── Cargo.toml
├── src/
│   ├── main.rs        CLI (run / build / emit / check)
│   ├── lib.rs         pipeline compile() dan compile_path()
│   ├── loader.rs      import multi-file, ekstensi .fen/.fenneth
│   ├── lexer.rs       source -> token
│   ├── parser.rs      token -> AST
│   ├── ast.rs         definisi AST
│   ├── checker.rs     type checker statis
│   ├── codegen.rs     AST -> C99
│   ├── runtime.c      runtime C yang disisipkan ke setiap hasil
│   └── diagnostic.rs  posisi dan pesan error
├── tests/pipeline.rs  test integrasi
├── scripts/           install.sh (Linux/macOS/Termux), install.ps1 (Windows),
│                      check-docs-i18n.sh (pengecekan kesamaan docs)
├── docs/              situs dokumentasi ini (VitePress, Inggris + Indonesia)
└── .github/workflows/ ci.yml, release.yml
```

## Tahapan

| Tahap | Modul | Catatan |
|---|---|---|
| Memuat | `loader.rs` | Menyelesaikan `import` relatif terhadap file yang meng-import, mencoba `.fen` lalu `.fenneth`, memuat tiap file sekali, dan menaruh item modul sebelum item file pengimpornya. |
| Lexing | `lexer.rs` | Menghasilkan token dengan posisi `file:baris:kolom`. Berhenti pada error pertama. |
| Parsing | `parser.rs` | Membangun AST. Berhenti pada error pertama. |
| Type checking | `checker.rs` | Menyelesaikan nama dan tipe, menegakkan aturan di dokumentasi ini, dan mengumpulkan semua error. |
| Code generation | `codegen.rs` | Menghasilkan C99. Array adalah pointer ke struct vektor hasil generate; struct menjadi struct C. |
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

`tests/pipeline.rs` berisi test integrasi untuk pipeline kompilasi dan CLI.
