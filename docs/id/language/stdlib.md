# Pustaka standar

Fenneth dilengkapi pustaka standar kecil. Pustaka ini ditulis dengan Fenneth sendiri dan sudah tertanam di compiler, jadi tidak ada yang perlu dipasang dan tidak ada file yang perlu dicari. Muat sebuah modul dengan `import`:

```fenneth
import "std/math";
import "std/string";

fn main() -> i32 {
    print(math_sqrt(2.0));                       // 1.41421
    print(string_to_upper("hello"));             // HELLO
    print(string_fixed(MATH_PI, 2));             // 3.14
    return 0;
}
```

| Modul | Isi |
|---|---|
| [`std/math`](#std-math) | Nilai mutlak, min/max, pangkat, FPB, akar kuadrat, trigonometri, pembulatan |
| [`std/string`](#std-string) | Mencari, memecah, menggabung, memangkas, mengganti, padding, parsing dan format angka |
| [`std/char`](#std-char) | Mengklasifikasi dan mengonversi satu byte |
| [`std/array`](#std-array) | Mengurutkan, mencari, mengiris, dan menjumlah array `i32`, `f64`, dan `String` |

Jalankan `fenneth std` untuk melihat daftar modul dan `fenneth std math` untuk mencetak kode sumber sebuah modul.

## Cara kerjanya

- **Satu namespace bersama.** Fenneth tidak punya namespace, jadi setiap nama publik diawali nama modulnya: `math_abs`, `string_split`, `array_sort_i32`. Nama yang diawali garis bawah (`_array_msort_i32`) adalah fungsi bantu privat; jangan dipanggil.
- **Awalan `std/` dicadangkan.** `import "std/math";` selalu berarti modul bawaan, walaupun ada folder `std` di sebelah file Anda.
- **Opt-in.** Program yang tidak meng-import sebuah modul tidak melihat nama-namanya, sehingga `max` atau `math_abs` buatan Anda tetap milik Anda. Jika Anda meng-import modul lalu juga mendefinisikan salah satu namanya, compiler melaporkan `function '...' is defined more than once`.
- **Hanya yang dipakai yang dikompilasi.** Fungsi std yang tidak pernah dijangkau program Anda dibuang dari C hasil generate. Semua isi modul yang di-import tetap diperiksa tipenya.
- **Modul bisa meng-import modul.** `std/array` meng-import `std/string`, jadi `string_compare` tersedia setelah `import "std/array";`.
- **Pustaka C.** `std/math` memakai pustaka matematika C. `fenneth run` dan `fenneth build` me-link-nya (`-lm`) di Linux, macOS, dan Termux. Jika Anda mengompilasi hasil `fenneth emit` sendiri, tambahkan `-lm`.
- **Error.** Salah pakai seperti `array_min_i32` pada array kosong menghentikan program dengan `panic: ...` dan exit code 101, sama seperti [pemeriksaan runtime](/id/language/limitations).

## std/math

```fenneth
import "std/math";
```

Konstanta: `MATH_PI`, `MATH_TAU`, `MATH_E` (`f64`), `MATH_I32_MAX`, `MATH_I32_MIN` (`i32`).

**Bilangan bulat (`i32`)** — aritmetika wrap saat overflow, seperti di seluruh Fenneth.

| Fungsi | Keterangan |
|---|---|
| `math_abs(x)` | Nilai mutlak. `math_abs(MATH_I32_MIN)` wrap dan tetap negatif. |
| `math_min(a, b)`, `math_max(a, b)` | Yang lebih kecil / lebih besar dari dua bilangan. |
| `math_clamp(x, lo, hi)` | Batasi `x` ke `lo..=hi`. `panic` bila `lo > hi`. |
| `math_sign(x)` | `-1`, `0`, atau `1`. |
| `math_mod(a, b)` | Modulo yang tidak pernah negatif untuk `b` positif: `math_mod(-7, 3)` adalah `2`. `panic` bila `b` adalah 0. |
| `math_ipow(base, exp)` | `base` pangkat `exp`. `panic` bila `exp < 0`. |
| `math_gcd(a, b)`, `math_lcm(a, b)` | Faktor persekutuan terbesar (FPB), kelipatan persekutuan terkecil (KPK). |

**Bilangan desimal (`f64`)**

| Fungsi | Keterangan |
|---|---|
| `math_absf(x)`, `math_minf(a, b)`, `math_maxf(a, b)`, `math_clampf(x, lo, hi)` | Kembaran `f64` dari fungsi di atas. |
| `math_is_nan(x)` | `true` untuk "bukan angka", misalnya `math_sqrt(-1.0)`. |
| `math_sqrt(x)`, `math_pow(x, y)`, `math_exp(x)` | Akar kuadrat, pangkat, `e` pangkat `x`. |
| `math_log(x)`, `math_log10(x)` | Logaritma natural dan basis 10. |
| `math_floor(x)`, `math_ceil(x)`, `math_trunc(x)`, `math_round(x)` | Pembulatan, menghasilkan `f64`. `math_round` membulatkan setengah menjauhi nol. |
| `math_floor_i32(x)`, `math_ceil_i32(x)`, `math_round_i32(x)` | Sama, tetapi menghasilkan `i32` (hasilnya harus muat dalam `i32`). |
| `math_sin(x)`, `math_cos(x)`, `math_tan(x)`, `math_atan(x)`, `math_atan2(y, x)` | Trigonometri dalam radian. |
| `math_to_radians(d)`, `math_to_degrees(r)` | Konversi derajat dan radian. |
| `math_hypot(x, y)` | `sqrt(x*x + y*y)`. |

## std/string

```fenneth
import "std/string";
```

String adalah string byte: panjang dan indeks dihitung dalam byte, dan fungsi huruf besar/kecil hanya mengubah huruf ASCII. Tidak ada fungsi yang mengubah argumennya.

**Memeriksa**

| Fungsi | Keterangan |
|---|---|
| `string_is_empty(s)` | `true` untuk `""`. |
| `string_compare(a, b)` | `-1`, `0`, atau `1`, membandingkan byte demi byte (`"Z"` datang sebelum `"a"`). |
| `string_starts_with(s, prefix)`, `string_ends_with(s, suffix)` | Uji awalan / akhiran. |
| `string_index_of(s, sub)` | Posisi `sub` pertama, atau `-1`. |
| `string_index_from(s, sub, from)` | Sama, mulai dari byte `from`. `panic` bila `from` di luar `0..=len(s)`. |
| `string_contains(s, sub)` | `true` bila `sub` ada di `s`. |
| `string_count(s, sub)` | Jumlah kemunculan yang tidak tumpang tindih. |

**Mengubah**

| Fungsi | Keterangan |
|---|---|
| `string_to_upper(s)`, `string_to_lower(s)` | Ubah huruf besar/kecil ASCII. |
| `string_reverse(s)` | Balik urutan byte. |
| `string_trim(s)`, `string_trim_start(s)`, `string_trim_end(s)` | Hapus spasi di kedua ujung / awal / akhir. |
| `string_repeat(s, n)` | `s` ditulis `n` kali. `panic` bila `n < 0`. |
| `string_replace(s, from, to)` | Ganti setiap kemunculan `from`. |
| `string_pad_left(s, width, fill)`, `string_pad_right(s, width, fill)` | Tambah padding sampai `width` byte. `fill` harus `String` satu byte. |

**Memecah dan menggabung**

| Fungsi | Keterangan |
|---|---|
| `string_split(s, sep)` | `[String]`. `string_split("a,b,,c", ",")` adalah `["a", "b", "", "c"]`. `sep` kosong memecah menjadi byte tunggal. |
| `string_lines(s)` | Baris-baris `s`. Menangani `\n` dan `\r\n`; akhir baris terakhir tidak menambah baris kosong. |
| `string_join(parts, sep)` | Gabung `[String]` dengan `sep`. |

**Angka**

| Fungsi | Keterangan |
|---|---|
| `string_is_int(s)` | `true` bila `s` adalah `i32` desimal (tanda opsional). Periksa ini sebelum `str_to_i32`, yang memberi `0` untuk masukan yang salah. |
| `string_parse_f64(s)` | Parse angka seperti `"3.25"` atau `"-1e3"`. Memberi `0.0` bila `s` bukan angka. |
| `string_fixed(x, decimals)` | Format dengan tepat `decimals` digit (0..40): `string_fixed(3.14159, 2)` adalah `"3.14"`. |

```fenneth
import "std/string";

fn main() -> i32 {
    let text: String = read_file("data.txt");
    for line in string_lines(text) {
        let fields: [String] = string_split(string_trim(line), ",");
        if len(fields) == 2 && string_is_int(fields[1]) {
            print(fields[0] + " = " + to_str(str_to_i32(fields[1])));
        }
    }
    return 0;
}
```

## std/char

```fenneth
import "std/char";
```

"Char" di sini adalah nilai byte seperti yang dikembalikan `str_at` (0..255). Hanya ASCII yang diklasifikasi. Pakai `char_str` untuk mengubah byte kembali menjadi `String`.

| Fungsi | Keterangan |
|---|---|
| `char_is_digit(c)`, `char_is_upper(c)`, `char_is_lower(c)` | `0-9`, `A-Z`, `a-z`. |
| `char_is_alpha(c)`, `char_is_alnum(c)` | Huruf / huruf dan angka. |
| `char_is_space(c)` | Spasi, tab, baris baru, carriage return, vertical tab, form feed. |
| `char_to_upper(c)`, `char_to_lower(c)` | Ubah huruf ASCII; byte lain dikembalikan apa adanya. |
| `char_digit_value(c)` | `'0'..'9'` menjadi 0..9, huruf menjadi 10..35 (huruf besar atau kecil), selain itu `-1`. |

## std/array

```fenneth
import "std/array";
```

Fenneth tidak punya generics, jadi setiap fungsi ada satu per tipe elemen, dengan tipenya sebagai akhiran: `_i32`, `_f64`, dan `_str` (untuk `[String]`). Di bawah, `T` mewakili akhiran itu. Menambah, menghapus, dan mengukur sudah bawaan (`push`, `pop`, `len`).

Array dipakai bersama saat di-assign, jadi fungsi yang ditandai *in place* mengubah array yang Anda berikan; yang lain mengembalikan array baru.

| Fungsi | Keterangan |
|---|---|
| `array_sort_T(a)` | Urutkan *in place*, terkecil dulu. Stabil, `O(n log n)`. String diurutkan byte demi byte. |
| `array_is_sorted_T(a)` | `true` bila tidak ada elemen yang lebih kecil dari elemen sebelumnya. |
| `array_reverse_T(a)` | Balik *in place*. |
| `array_copy_T(a)` | Array baru dengan elemen yang sama. |
| `array_slice_T(a, from, to)` | Elemen `[from, to)` sebagai array baru. `panic` bila rentang tidak ada di dalam `a`. |
| `array_concat_T(a, b)` | Array baru: elemen `a`, lalu elemen `b`. |
| `array_fill_T(n, v)` | Array baru berisi `n` salinan `v`. `panic` bila `n < 0`. |
| `array_contains_T(a, v)` | `true` bila ada elemen yang sama dengan `v`. |
| `array_index_of_T(a, v)` | Indeks elemen pertama yang sama dengan `v`, atau `-1`. |
| `array_min_T(a)`, `array_max_T(a)` | Elemen terkecil / terbesar. `panic` pada array kosong. |
| `array_sum_i32(a)`, `array_sum_f64(a)` | Jumlah elemen (`0` untuk array kosong). Hanya untuk angka. |
| `array_range(start, end)` | `[i32]` berisi `start, start+1, ..., end-1`. |

```fenneth
import "std/array";

fn main() -> i32 {
    let words: [String] = ["pear", "apple", "fig"];
    array_sort_str(words);
    print(words[0]);                          // apple
    print(array_sum_i32(array_range(1, 5)));  // 10
    return 0;
}
```

## Batasan

- Tanpa generics, setiap fungsi array ada dalam tiga salinan, dan tidak ada `array_sort` untuk array struct. Tulis loop sendiri (atau merge sort seperti yang ditunjukkan `fenneth std array`) untuk itu.
- Belum ada hash map, fungsi sistem file di luar yang bawaan, maupun fungsi string yang paham Unicode.
