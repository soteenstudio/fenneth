# Fenneth v0.2.0

Fenneth adalah bahasa pemrograman bertipe statis dengan sintaks modern yang
**dikompilasi ke C99**, lalu di-compile oleh GCC/Clang menjadi executable
native. Compiler-nya ditulis dalam Rust tanpa dependensi eksternal.

```
.fen / .fenneth -> Lexer -> Parser (AST) -> Type Checker -> C Code Generator -> .c -> gcc/clang -> executable
```

File sumber boleh berekstensi **`.fen`** atau **`.fenneth`** (keduanya setara,
termasuk untuk `import`).

## Instalasi

Setelah rilis dipublikasikan di GitHub (lihat [Membuat rilis](#membuat-rilis)),
ganti `OWNER/REPO` dengan repositori Anda.

**Linux, macOS, dan Termux (Android)**

```sh
curl -fsSL https://github.com/OWNER/REPO/releases/latest/download/install.sh | sh
```

**Windows (PowerShell)**

```powershell
irm https://github.com/OWNER/REPO/releases/latest/download/install.ps1 | iex
```

Lalu panggil dari mana saja:

```sh
fenneth --version
fenneth hello.fen
```

Installer memverifikasi checksum SHA-256, memasang binary ke
`~/.fenneth/bin` (Windows: `%LOCALAPPDATA%\Fenneth\bin`; Termux: `$PREFIX/bin`),
dan menambahkannya ke `PATH`. Variabel opsional:

| Variabel | Fungsi |
|---|---|
| `FENNETH_VERSION` | pasang tag tertentu, mis. `v0.2.0` (default: terbaru) |
| `FENNETH_INSTALL` | folder instalasi (binary di `<folder>/bin`) |
| `FENNETH_NO_MODIFY_PATH=1` | jangan ubah profil shell / PATH pengguna |

### Kebutuhan: compiler C

Fenneth menghasilkan C, jadi `fenneth run` dan `fenneth build` membutuhkan
compiler C. (`fenneth emit` dan `fenneth check` tidak membutuhkannya.)

| Platform | Cara memasang |
|---|---|
| Termux | `pkg install clang` (dan `pkg install curl` untuk installer) |
| Debian/Ubuntu | `sudo apt install gcc` |
| macOS | `xcode-select --install` |
| Windows | `winget install -e --id BrechtSanders.WinLibs.POSIX.UCRT` (MinGW-w64 gcc) atau MSYS2 |

Compiler dicari berurutan: `--cc`, `$FENNETH_CC`, `gcc`, `clang`, `cc`.

### Dari source

```sh
cargo install --git https://github.com/OWNER/REPO fenneth
# atau, dari checkout lokal:
cargo build --release     # binary: target/release/fenneth
```

Di Termux: `pkg install rust clang` lalu perintah `cargo install` di atas
(installer juga melakukan ini otomatis bila tidak ada binary siap pakai untuk
arsitektur Anda, misalnya Termux 32-bit).

## Cara pakai

```sh
fenneth hello.fen                       # sama dengan: fenneth run hello.fen
fenneth run app.fen -- arg1 arg2        # argumen setelah -- masuk ke args()
fenneth build app.fen -o app            # executable native
fenneth emit app.fen -o app.c           # hanya hasilkan kode C
fenneth check app.fen                   # type-check saja
```

Opsi `build`/`run`: `--cc <compiler>`, `--link <arg>` (bisa berulang, untuk
library tambahan), `--keep-c`. Ekstensi boleh dihilangkan: `fenneth app`
mencari `app.fen`, lalu `app.fenneth`.

## Bahasa

```fenneth
import "geometry";                      // memuat geometry.fen / geometry.fenneth

struct Point { x: i32, y: i32, }

const LIMIT: i32 = 3;                   // global: hanya nilai literal

fn dist2(a: Point, b: Point) -> i32 {
    let dx: i32 = a.x - b.x;
    let dy: i32 = a.y - b.y;
    return dx * dx + dy * dy;
}

fn main() -> i32 {
    let pts: [Point] = [];
    for i in 0..LIMIT {                 // 0, 1, 2 (batas atas eksklusif)
        push(pts, Point { x: i, y: i * 2 });
    }
    let total: i32 = 0;
    for p in pts {                      // iterasi array
        total += dist2(p, Point { x: 0, y: 0 });
    }
    print("total = " + to_str(total));
    return 0;
}
```

| Fitur | Sintaks |
|---|---|
| Variabel / konstanta | `let x: i32 = 1;` / `const y: f64 = 2.5;` (tipe & nilai awal wajib) |
| Tipe dasar | `i32`, `f64`, `bool`, `String` |
| Array | `[T]` (bisa bertingkat `[[i32]]`), literal `[1, 2]`, indeks `a[i]` |
| Struct | `struct P { x: i32, }`, literal `P { x: 1 }`, akses `p.x`; nama harus berawalan huruf kapital |
| Fungsi | `fn f(a: i32) -> bool { ... }` (tanpa `->` = tidak mengembalikan nilai) |
| Kontrol alur | `if / else if / else`, `while`, `for i in a..b`, `for x in array`, `break`, `continue` |
| Operator | `+ - * / %`, `== != < <= > >=`, `&& \|\| !`, `= += -= *= /= %=` (`+` juga menyambung `String`) |
| Modul | `import "path";` (relatif terhadap file yang meng-import) |
| FFI | `extern fn nama(a: i32) -> f64;` (tautkan ke simbol C/Rust) |
| Komentar | `// baris` dan `/* blok */` |

Semantik: **struct disalin per nilai**, **array berbagi referensi** (seperti
objek). `String` tidak bisa diubah (immutable).

### Fungsi bawaan

| Fungsi | Keterangan |
|---|---|
| `print(x)` / `write(x)` / `eprint(x)` | cetak `i32`, `f64`, `bool`, `String` (dengan baris baru / tanpa / ke stderr) |
| `to_str(x)` | `i32`/`f64`/`bool` menjadi `String` |
| `to_f64(x)` / `to_i32(x)` | konversi eksplisit (tidak ada konversi implisit) |
| `len(x)` | panjang `String` (byte) atau array |
| `push(arr, v)` / `pop(arr)` | tambah / ambil elemen terakhir |
| `str_at(s, i)` | kode byte ke-`i` (`i32`) |
| `substr(s, a, b)` | potongan `[a, b)` |
| `char_str(code)` | `String` satu karakter dari kode 0..255 |
| `str_to_i32(s)` | parse bilangan bulat |
| `read_file(path)` / `write_file(path, s)` / `file_exists(path)` | berkas (`read_file` mengembalikan `""` bila gagal) |
| `read_line()` | satu baris dari stdin (`""` saat EOF) |
| `args()` | `[String]` argumen program |
| `exit(code)` / `panic(msg)` | keluar / hentikan dengan pesan (exit code 101) |

Pemeriksaan saat runtime: indeks di luar batas, `pop` array kosong, pembagian
dengan nol, dan `substr` di luar rentang akan `panic` (exit code 101).

## Memakai Rust dari Fenneth

`extern fn` menautkan fungsi Fenneth ke simbol C, termasuk fungsi Rust
`#[no_mangle] pub extern "C"`. Tipe yang boleh: `i32`, `f64`, `bool`, `String`
(`*const c_char` di sisi Rust).

```sh
rustc --edition 2021 --crate-type staticlib -O examples/ffi/host.rs -o libhost.a
fenneth run examples/ffi/ffi.fen --link ./libhost.a --link -lpthread --link -ldl --link -lm
# 42
# 1.5
# Hello from Rust, Fenneth!
```

(Di Windows, library sistem yang diperlukan staticlib Rust berbeda; sesuaikan
argumen `--link`.)

## Menulis ulang Fenneth dengan Fenneth

v0.2.0 menambahkan fondasi untuk *self-hosting*: struct, array, string
dinamis, I/O berkas, `import`, dan FFI. Sebagai bukti, **lexer Fenneth sudah
ditulis ulang dengan Fenneth** di [`examples/selfhost/`](examples/selfhost) dan
diuji terhadap lexer Rust (jumlah token dan posisi baris:kolom setiap token
harus identik pada beberapa file sumber nyata).

```sh
fenneth run examples/selfhost/main.fen -- examples/hello.fenneth --dump
```

Parser, type checker, dan code generator **belum** ditulis ulang; yang masih
kurang (enum + `match`, peta/hash map, dsb.) dan urutan langkahnya ada di
[`docs/BOOTSTRAP.md`](docs/BOOTSTRAP.md).

## Batasan v0.2.0

- **Memori tidak pernah dibebaskan** (tanpa GC/`free`): string hasil `+`,
  `substr`, dan array hanya bertambah. Cocok untuk program singkat seperti
  compiler, bukan proses yang berjalan lama.
- **Urutan evaluasi** operand dan argumen mengikuti C, yaitu tidak ditentukan.
  Hindari ekspresi seperti `f() + g()` yang bergantung pada urutan efek samping.
- Overflow `i32` membungkus (wrap) bila di-compile lewat `fenneth build/run`
  (memakai `-fwrapv`); jika Anda meng-compile hasil `emit` sendiri, tambahkan
  `-fwrapv` untuk perilaku yang sama.
- Global hanya boleh bernilai literal; belum ada enum, generics, closure,
  maupun perbandingan `<`/`>` pada `String`.
- Tidak ada shadowing: nama variabel tidak boleh dideklarasikan ulang selama
  masih terlihat dari scope luar (termasuk global).
- Fungsi bernilai balik harus `return` di semua jalur; `panic()`/`exit()`
  tidak dihitung sebagai `return`.
- Compiler berhenti pada error lexer/parser pertama; error type checker
  dilaporkan sekaligus.

## Membuat rilis

1. Buat repositori GitHub dan push proyek ini.
2. Pastikan workflow **CI** (`.github/workflows/ci.yml`) hijau.
3. Naikkan `version` di `Cargo.toml`, lalu beri tag yang sama:
   ```sh
   git tag v0.2.0 && git push origin v0.2.0
   ```
4. Workflow **Release** membangun binary untuk Linux x86_64 dan aarch64
   (statis/musl, termasuk Termux aarch64), Windows x86_64, serta macOS arm64
   dan x86_64, lalu memublikasikannya bersama `install.sh` dan `install.ps1`
   (nama repositori sudah ditanam otomatis) dan file checksum.

Workflow **CI** menjalankan seluruh test di Linux, macOS, dan Windows, serta
menguji kedua installer terhadap arsip rilis yang dibangun secara lokal.

## Struktur proyek

```
fenneth/
├── Cargo.toml
├── src/
│   ├── main.rs        CLI (run / build / emit / check)
│   ├── lib.rs         pipeline compile() dan compile_path()
│   ├── loader.rs      import multi-file, ekstensi .fen/.fenneth
│   ├── lexer.rs       source -> tokens
│   ├── parser.rs      tokens -> AST
│   ├── ast.rs         definisi AST
│   ├── checker.rs     type checker statis
│   ├── codegen.rs     AST -> C99
│   ├── runtime.c      runtime C yang disisipkan ke setiap hasil
│   └── diagnostic.rs  posisi & pesan error
├── examples/          hello, demo, selfhost/ (lexer dalam Fenneth), ffi/ (Rust)
├── tests/pipeline.rs  test integrasi
├── scripts/           install.sh (Linux/macOS/Termux), install.ps1 (Windows)
├── docs/BOOTSTRAP.md  peta jalan menuju self-hosting
└── .github/workflows/ ci.yml, release.yml
```

Lisensi: MIT.
