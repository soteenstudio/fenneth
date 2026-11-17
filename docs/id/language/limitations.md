# Batasan dan pemeriksaan runtime

## Pemeriksaan runtime

Situasi berikut menghentikan program dengan `panic: ...` di stderr dan exit code **101**:

- indeks array di luar batas
- `pop` pada array kosong
- pembagian atau sisa bagi bilangan bulat dengan nol
- `substr` dengan rentang tidak valid, `str_at` di luar rentang, `char_str` di luar 0..255
- kehabisan memori
- `panic(msg)` eksplisit

Overflow `i32` **membungkus** (compiler C dipanggil dengan `-fwrapv`). Jika Anda meng-compile hasil `fenneth emit` sendiri, tambahkan `-fwrapv` agar perilakunya sama.

## Batasan yang diketahui

- **Memori tidak pernah dibebaskan** (tanpa GC dan tanpa `free`). String hasil `+`, `substr`, dan array hanya bertambah. Cocok untuk program singkat seperti compiler dan tool, bukan proses yang berjalan lama.
- **Urutan evaluasi** operand dan argumen mengikuti C dan tidak ditentukan. Hindari ekspresi seperti `f() + g()` yang bergantung pada urutan efek samping.
- **Global** hanya boleh diberi nilai awal berupa literal.
- **Fitur yang belum ada**: enum, generics, closure, hash map, dan `<` / `>` pada `String`.
- **Tanpa shadowing**: nama variabel tidak boleh dideklarasikan ulang selama masih terlihat dari scope luar, termasuk global.
- **Return di semua jalur**: fungsi bertipe kembalian harus `return` di semua jalur. `panic()` dan `exit()` tidak dihitung.
- **Pelaporan error**: compiler berhenti pada error lexer atau parser yang pertama; error type checker dilaporkan sekaligus.
- **String adalah byte**: panjang dan indeks dihitung dalam byte, bukan karakter Unicode.
- **`main`** harus `fn main() -> i32` tanpa parameter.
