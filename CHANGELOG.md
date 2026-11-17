# Changelog

## 0.2.0

Fondasi untuk self-hosting dan distribusi.

**Bahasa**
- `struct` (nilai), array `[T]` (referensi, bisa bertingkat), literal struct dan array.
- `for i in a..b`, `for x in array`, `break`, `continue`.
- Global `let`/`const` bernilai literal.
- Penyambungan string dengan `+`; fungsi bawaan string (`len`, `substr`,
  `str_at`, `char_str`, `str_to_i32`, `to_str`).
- `push`, `pop`, `len` untuk array.
- I/O: `read_file`, `write_file`, `file_exists`, `read_line`, `args`, `eprint`,
  `exit`, `panic`.
- `import "path";` multi-file dengan deteksi siklus; ekstensi `.fen` dan `.fenneth`.
- `extern fn` untuk FFI ke C dan Rust.
- Literal `-2147483648` sekarang valid.
- Pemeriksaan saat runtime (indeks, `pop` kosong, bagi nol, `substr`) dengan `panic`.

**Tooling**
- CLI: `fenneth <file>`, `run` (dengan `-- args`), `build`, `emit`, `check`,
  opsi `--link`, `--cc`, `--keep-c`.
- Installer `install.sh` (Linux/macOS/Termux) dan `install.ps1` (Windows),
  dengan verifikasi checksum SHA-256 dan pengaturan PATH.
- Workflow GitHub: CI (Linux/macOS/Windows) dan Release (binary multi-platform).
- Lexer Fenneth ditulis dalam Fenneth (`examples/selfhost`) dan diuji terhadap lexer Rust.

**Perubahan**
- `main` C sekarang menerima `argc/argv`.
- Pesan error menyebut file asal (`file:baris:kolom`) untuk program multi-file.

## 0.1.0

Rilis pertama: lexer, parser, type checker, dan code generator ke C untuk
`i32`, `f64`, `bool`, `String`, `let`/`const`, `fn`, `if`/`while`, `print`.
