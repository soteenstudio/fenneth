# Dasar-dasar

## Struktur program

File Fenneth berisi item tingkat atas: `import`, `struct`, `let` / `const` global, dan `fn` (termasuk `extern fn`). Eksekusi dimulai dari `main`, yang harus dideklarasikan persis seperti ini:

```fenneth
fn main() -> i32 {
    return 0;
}
```

`main` tidak menerima parameter (pakai `args()` untuk argumen baris perintah) dan harus mengembalikan `i32`. Nilai yang dikembalikan menjadi exit code proses.

## Komentar

```fenneth
// komentar satu baris
/* komentar blok */
```

Komentar blok yang tidak ditutup adalah error kompilasi.

## Identifier

Identifier diawali huruf ASCII atau `_`, lalu dilanjutkan huruf ASCII, angka, atau `_`. [Keyword](/id/language/keywords) dan [nama fungsi bawaan](/id/language/builtins) tidak bisa dipakai sebagai nama fungsi Anda sendiri.

## Tipe

| Tipe | Keterangan | Literal |
|---|---|---|
| `i32` | Bilangan bulat bertanda 32-bit | `0`, `42`, `-2147483648` |
| `f64` | Bilangan pecahan 64-bit | `1.5`, `0.25` (ada angka di kedua sisi titik) |
| `bool` | Boolean | `true`, `false` |
| `String` | String byte yang tidak bisa diubah | `"teks"` dengan escape `\n \t \r \\ \"` |
| `[T]` | Array berisi `T`, bisa bertingkat (`[[i32]]`) | `[1, 2, 3]`, `[]` |
| `Nama` | [Struct](/id/language/data) buatan sendiri | `Nama { field: nilai }` |

Literal bilangan bulat yang tidak muat di `i32` adalah error kompilasi. Escape yang tidak dikenal di dalam string (mis. `\q`) juga error kompilasi.

**Tidak ada konversi implisit.** Konversi secara eksplisit dengan `to_f64()` dan `to_i32()`:

```fenneth
let n: i32 = 7;
let half: f64 = to_f64(n) / 2.0;   // 3.5
```

## Variabel dan konstanta

Keduanya membutuhkan **anotasi tipe** dan **nilai awal**:

```fenneth
let count: i32 = 0;
const PI: f64 = 3.14159;

count = count + 1;     // boleh
// PI = 3.0;           // error: cannot assign to constant 'PI'
```

Aturan:

- **Tanpa shadowing.** Nama tidak boleh dideklarasikan ulang selama masih terlihat dari scope luar, termasuk global.
- `const` tidak bisa di-assign, dan field-nya pun tidak bisa diubah.
- Variabel `for` tidak bisa di-assign.
- Literal array kosong butuh anotasi tipe: `let xs: [i32] = [];`.

### Global

Global dideklarasikan di tingkat atas dan hanya boleh diberi nilai awal berupa **literal** (angka, string, `true` / `false`, atau literal angka negatif):

```fenneth
const LIMIT: i32 = 3;      // boleh
let greeting: String = "hi";   // boleh
// const N: i32 = 1 + 2;   // error: global initializers must be literal values
```

## Operator

Dari presedensi terendah ke tertinggi (semua operator biner asosiatif kiri):

| Tingkat | Operator | Tipe operand |
|---|---|---|
| 1 | `\|\|` | `bool` |
| 2 | `&&` | `bool` |
| 3 | `==` `!=` | kedua sisi bertipe sama: `i32`, `f64`, `bool`, atau `String` |
| 4 | `<` `<=` `>` `>=` | kedua sisi bertipe numerik yang sama (`i32` atau `f64`) |
| 5 | `+` `-` | kedua sisi bertipe numerik yang sama; `+` juga menyambung dua `String` |
| 6 | `*` `/` | kedua sisi bertipe numerik yang sama |
| 6 | `%` | hanya `i32` |
| unary | `-` `!` | `-` untuk angka, `!` untuk `bool` |

Bentuk assignment: `=`, `+=`, `-=`, `*=`, `/=`, `%=`. Targetnya bisa variabel, field struct (`p.x`), atau elemen array (`a[i]`). Assignment adalah statement, bukan ekspresi.

Membandingkan `String` dengan `<` atau `>` belum didukung. Membagi dengan literal `0` adalah error kompilasi; membagi dengan nilai nol saat runtime akan `panic`.

::: warning Urutan evaluasi
Urutan evaluasi operand dan argumen mengikuti C dan tidak ditentukan. Jangan menulis ekspresi seperti `f() + g()` yang bergantung pada urutan efek samping.
:::
