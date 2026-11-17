# Changelog

## Belum dirilis

**Tooling**
- Nomor rilis kini memakai format `0.xxx` ala Luau (mis. `0.003`). Nomornya ditanamkan ke `fenneth --version`.
- Workflow rilis bisa dijalankan dengan push tag atau manual, dengan penomoran otomatis dan mode dry run.
- Dokumentasi dipindah ke situs VitePress dwibahasa (Inggris dan Indonesia) untuk Vercel, dan `README.md` ditulis ulang.

## 0.2.0

Fondasi untuk self-hosting dan distribusi.

**Bahasa**
- `struct` (nilai), array `[T]` (referensi, bisa bertingkat), literal struct dan array.
- `for i in a..b`, `for x in array`, `break`, `continue`.
- Global `let` / `const` bernilai literal.
- Penyambungan string dengan `+`; fungsi bawaan string (`len`, `substr`, `str_at`, `char_str`, `str_to_i32`, `to_str`).
- `push`, `pop`, `len` untuk array.
- I/O: `read_file`, `write_file`, `file_exists`, `read_line`, `args`, `eprint`, `exit`, `panic`.
- `import "path";` multi-file dengan deteksi siklus; ekstensi `.fen` dan `.fenneth`.
- `extern fn` untuk FFI ke C dan Rust.
- Literal `-2147483648` sekarang valid.
- Pemeriksaan runtime (indeks, `pop` kosong, bagi nol, `substr`) dengan `panic`.

**Tooling**
- CLI: `fenneth <file>`, `run` (dengan `-- args`), `build`, `emit`, `check`, serta opsi `--link`, `--cc`, `--keep-c`.
- `install.sh` (Linux/macOS/Termux) dan `install.ps1` (Windows) dengan verifikasi SHA-256 dan pengaturan PATH.
- Workflow GitHub: CI (Linux/macOS/Windows) dan Release (binary multi-platform).

**Perubahan**
- `main` C hasil generate sekarang menerima `argc` / `argv`.
- Pesan error menyebut file sumber (`file:baris:kolom`) untuk program multi-file.
