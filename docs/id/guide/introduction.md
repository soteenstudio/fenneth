# Pengantar

Fenneth adalah bahasa pemrograman bertipe statis dengan sintaks modern. Program Fenneth dikompilasi ke **C99**, lalu GCC atau Clang mengubah C tersebut menjadi executable native. Compiler-nya ditulis dalam Rust tanpa dependensi eksternal.

```
.fen / .fenneth → Lexer → Parser (AST) → Type checker → Generator kode C → .c → gcc/clang → executable
```

File sumber berekstensi **`.fen`** atau **`.fenneth`**. Keduanya setara di mana pun, termasuk pada `import`.

## Seperti apa tampilannya

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

## Rancangan secara singkat

- **Tipe eksplisit.** Setiap variabel punya anotasi tipe dan nilai awal. Tidak ada konversi implisit antara `i32` dan `f64`.
- **Permukaan kecil.** `struct`, array, `String`, `if` / `while` / `for`, fungsi, modul, dan FFI. Belum ada generics, enum, maupun closure.
- **Struct bernilai, array berbagi referensi.** Struct disalin saat di-assign; array berbagi referensi.
- **Diperiksa saat runtime.** Indeks di luar batas, `pop` array kosong, pembagian dengan nol, dan rentang `substr` yang salah menghentikan program dengan `panic` (exit code 101).
- **Diagnostik yang jelas.** Error menyertakan `file:baris:kolom`, dan type checker melaporkan semua errornya sekaligus.

## Yang Anda butuhkan

| Tugas | Butuh compiler C? |
|---|---|
| `fenneth run`, `fenneth build` | Ya (GCC, Clang, atau `cc`) |
| `fenneth emit`, `fenneth check` | Tidak |

Berikutnya: [pasang Fenneth](/id/guide/installation), lalu coba [contoh kode](/id/guide/examples).
