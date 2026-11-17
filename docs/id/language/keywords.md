# Keyword

Fenneth punya 16 kata cadangan. Kata-kata ini tidak bisa dipakai sebagai nama variabel, fungsi, field, atau struct.

| Keyword | Fungsi | Contoh |
|---|---|---|
| `let` | Mendeklarasikan variabel (tipe dan nilai awal wajib). | `let n: i32 = 0;` |
| `const` | Mendeklarasikan konstanta. | `const MAX: i32 = 10;` |
| `fn` | Mendeklarasikan fungsi. | `fn add(a: i32, b: i32) -> i32 { ... }` |
| `return` | Keluar dari fungsi, boleh dengan nilai. | `return a + b;` |
| `if` | Percabangan. | `if n > 0 { ... }` |
| `else` | Cabang alternatif; bisa digabung menjadi `else if`. | `else if n == 0 { ... } else { ... }` |
| `while` | Mengulang selama kondisi benar. | `while i < 10 { ... }` |
| `for` | Mengulang pada rentang atau array. | `for i in 0..3 { ... }` |
| `in` | Memisahkan variabel loop dari yang diiterasi. | `for x in xs { ... }` |
| `break` | Keluar dari loop terdalam. | `break;` |
| `continue` | Lompat ke iterasi loop berikutnya. | `continue;` |
| `true` | Literal boolean. | `let ok: bool = true;` |
| `false` | Literal boolean. | `let ok: bool = false;` |
| `struct` | Mendeklarasikan tipe struct. | `struct P { x: i32, }` |
| `extern` | Mendeklarasikan fungsi yang diimplementasikan di luar Fenneth (dipakai bersama `fn`). | `extern fn f(a: i32) -> i32;` |
| `import` | Memuat file sumber lain. | `import "geometry";` |

## Nama tipe cadangan

`i32`, `f64`, `bool`, dan `String` adalah nama tipe bawaan. Struct tidak boleh memakainya sebagai nama.

## Nama fungsi bawaan

Nama-nama berikut milik bahasa dan tidak bisa didefinisikan ulang sebagai fungsi Anda: `print`, `write`, `eprint`, `to_f64`, `to_i32`, `to_str`, `len`, `push`, `pop`, `str_at`, `substr`, `char_str`, `str_to_i32`, `read_file`, `write_file`, `file_exists`, `read_line`, `args`, `exit`, `panic`. Lihat [Fungsi bawaan](/id/language/builtins).

## Simbol

| Simbol | Arti |
|---|---|
| `->` | Tipe kembalian pada tanda tangan fungsi |
| `..` | Rentang eksklusif pada `for i in a..b` |
| `:` | Anotasi tipe, field struct, field pada literal struct |
| `;` | Penutup statement |
| `{ }` | Blok, badan struct, literal struct |
| `[ ]` | Tipe array, literal array, pengindeksan |
| `.` | Akses field struct |
| `//` `/* */` | Komentar |
