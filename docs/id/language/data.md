# Struct, array, dan string

## Struct

```fenneth
struct Point { x: i32, y: i32, }

fn main() -> i32 {
    let p: Point = Point { x: 1, y: 2 };
    p.x = 10;
    print(p.x + p.y);        // 12
    return 0;
}
```

- Nama struct **harus diawali huruf kapital** dan tidak boleh sama dengan nama tipe bawaan.
- Struct butuh minimal satu field. Nama field harus unik.
- Literal harus mengisi **setiap** field tepat satu kali; urutannya bebas. Koma di akhir diperbolehkan.
- Struct tidak boleh memuat dirinya sendiri secara langsung, tetapi boleh memuat array dari dirinya, yang dipakai untuk membuat pohon:

```fenneth
struct Node { value: i32, children: [Node], }
```

**Struct disalin per nilai** saat di-assign dan saat dilewatkan ke fungsi. Mengubah salinan tidak mengubah aslinya.

## Array

`[T]` adalah array yang bisa bertambah. Array bisa bertingkat: `[[i32]]`.

```fenneth
let xs: [i32] = [1, 2, 3];
let empty: [String] = [];          // anotasi wajib untuk []
let grid: [[i32]] = [[1, 2], [3, 4]];

push(xs, 4);                       // [1, 2, 3, 4]
let last: i32 = pop(xs);           // 4
xs[0] = 9;
print(len(xs));                    // 3
print(grid[1][0]);                 // 3
```

- Semua elemen bertipe sama; literal campuran adalah error kompilasi.
- Indeks harus `i32`. Indeks di luar batas akan `panic` (exit code 101).
- `pop` pada array kosong akan `panic`.
- **Array berbagi referensi**: `let b: [i32] = a;` membuat `b` dan `a` menjadi array yang sama.
- Iterasi dengan `for x in xs { ... }`.

## String

`String` adalah urutan byte yang tidak bisa diubah. Indeks dan panjang dihitung dalam **byte**, bukan karakter Unicode.

```fenneth
let s: String = "hello";
let t: String = s + ", world";        // penyambungan membuat string baru
print(len(t));                        // 12
print(str_at(s, 0));                  // 104  (byte 'h')
print(substr(s, 1, 4));               // ell  (byte 1, 2, 3)
print(char_str(65));                  // A
print(s == "hello");                  // true
```

| Tugas | Cara |
|---|---|
| Menyambung | `a + b` |
| Panjang (byte) | `len(s)` |
| Byte pada indeks | `str_at(s, i)` mengembalikan `i32` dalam 0..255 |
| Potongan `[a, b)` | `substr(s, a, b)` |
| String satu karakter | `char_str(code)` dengan `code` dalam 0..255 |
| Kesamaan | `==`, `!=` |
| Angka ke string | `to_str(n)` |
| String ke bilangan bulat | `str_to_i32(s)` |

Pengindeksan `s[i]` pada string tidak tersedia; pakai `str_at`.
