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
| [`std/fs`](#std-fs) | Direktori, metadata file, menyalin, mengganti nama, menghapus, dan bantuan path |
| [`std/http`](#std-http) | Mengambil URL lewat HTTP(S): GET, POST, unduhan, encoding URL |
| [`std/zip`](#std-zip) | Membaca, membongkar, dan membuat arsip `.zip` |

Jalankan `fenneth std` untuk melihat daftar modul dan `fenneth std math` untuk mencetak kode sumber sebuah modul.

## Cara kerjanya

- **Satu namespace bersama.** Fenneth tidak punya namespace, jadi setiap nama publik diawali nama modulnya: `math_abs`, `string_split`, `array_sort_i32`. Nama yang diawali garis bawah (`_array_msort_i32`) adalah fungsi bantu privat; jangan dipanggil.
- **Awalan `std/` dicadangkan.** `import "std/math";` selalu berarti modul bawaan, walaupun ada folder `std` di sebelah file Anda.
- **Opt-in.** Program yang tidak meng-import sebuah modul tidak melihat nama-namanya, sehingga `max` atau `math_abs` buatan Anda tetap milik Anda. Jika Anda meng-import modul lalu juga mendefinisikan salah satu namanya, compiler melaporkan `function '...' is defined more than once`.
- **Hanya yang dipakai yang dikompilasi.** Fungsi std yang tidak pernah dijangkau program Anda dibuang dari C hasil generate. Semua isi modul yang di-import tetap diperiksa tipenya. `std/fs`, `std/http`, dan `std/zip` juga membawa C-nya sendiri, yang baru disisipkan ke hasil generate bila ada fungsi dari modul itu yang bertahan.
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

## std/fs

```fenneth
import "std/fs";
```

File dan direktori. Fungsi bawaan `read_file`, `write_file`, dan `file_exists` tetap menangani kasus satu file utuh; modul ini menambah direktori, metadata, penyalinan, dan pengelolaan path. Path adalah `String` biasa. Fungsi yang bisa gagal mengembalikan `false` (atau `-1`), tidak pernah panic.

**Menanyakan sebuah path**

| Fungsi | Keterangan |
|---|---|
| `fs_exists(path)` | `true` bila ada sesuatu di sana: file, direktori, atau link. |
| `fs_is_dir(path)`, `fs_is_file(path)` | Apakah direktori / file biasa? |
| `fs_size(path)` | Ukuran dalam byte, atau `-1` bila bukan file yang bisa dibaca. |

**Mengubah sistem file**

| Fungsi | Keterangan |
|---|---|
| `fs_mkdir(path)` | Membuat satu direktori. `true` bila dibuat atau sudah ada. |
| `fs_mkdir_all(path)` | Membuat `path` beserta semua induk yang belum ada: `fs_mkdir_all("a/b/c")`. |
| `fs_remove(path)` | Menghapus file (atau direktori kosong). |
| `fs_remove_dir(path)` | Menghapus direktori kosong. |
| `fs_rename(from, to)` | Memindahkan, mengganti `to` bila sudah ada. |
| `fs_copy(from, to)` | Menyalin isi satu file ke file lain. |
| `fs_append(path, text)` | Menambahkan teks di akhir, membuat file bila perlu. |

**Direktori**

| Fungsi | Keterangan |
|---|---|
| `fs_list(path)` | Nama entri, satu per baris (`""` bila tidak bisa dibaca). |
| `fs_entries(path)` | Sama, sudah dipecah menjadi `[String]`. |

**Lokasi**

| Fungsi | Keterangan |
|---|---|
| `fs_cwd()` | Direktori kerja saat ini. |
| `fs_temp_dir()` | `$TMPDIR` / `%TEMP%`, atau `/tmp`. |
| `fs_temp_path(prefix)` | Path baru yang belum dipakai di direktori temp (tidak dibuat). |

**Path**

| Fungsi | Keterangan |
|---|---|
| `fs_separator()` | `"\\"` di Windows, `"/"` di tempat lain. |
| `fs_join(a, b)` | Menggabung dengan separator, kecuali `b` absolut atau `a` sudah berakhir dengannya. |
| `fs_basename(path)` | Bagian terakhir: `"a/b/c.txt"` menghasilkan `"c.txt"`. |
| `fs_dirname(path)` | Semua kecuali bagian terakhir: `"a/b/c.txt"` menghasilkan `"a/b"`. |
| `fs_extension(path)` | Teks setelah titik terakhir, tanpa titiknya (`"gz"`). |

```fenneth
import "std/fs";

fn main() -> i32 {
    let dir: String = fs_join(fs_temp_dir(), "demo");
    fs_mkdir_all(fs_join(dir, "sub"));
    write_file(fs_join(dir, "a.txt"), "hello\n");
    for name in fs_entries(dir) {
        print(name + ": " + to_str(fs_size(fs_join(dir, name))));
    }
    return 0;
}
```

## std/http

```fenneth
import "std/http";
```

Mengambil URL lewat HTTP dan HTTPS. Permintaan dijalankan oleh perintah `curl` (tersedia di Windows 10+, macOS, dan sebagian besar Linux); `wget` dipakai bila curl tidak ada, dan hanya untuk URL biasa. Tidak ada yang di-link, tetapi program butuh salah satunya di `PATH` saat berjalan.

| Fungsi | Keterangan |
|---|---|
| `http_get(url)` | GET `url`; isi respons (`""` bila permintaan gagal). |
| `http_post(url, content_type, body)` | POST `body`; isi respons. |
| `http_post_json(url, json)` | POST dengan `Content-Type: application/json`. |
| `http_post_form(url, form)` | POST dengan `application/x-www-form-urlencoded`. |
| `http_download(url, path)` | Menyimpan respons ke file. `true` bila berhasil. Pakai ini untuk biner. |
| `http_last_status()` | Kode status permintaan terakhir (`200`, `404`, ...), `0` bila gagal. |
| `http_last_ok()` | `true` bila kode itu 2xx. |
| `http_last_error()` | Penyebab kegagalan terakhir (`""` bila berhasil). |
| `http_encode(s)`, `http_decode(s)` | Percent-encode / decode. `http_decode` juga mengubah `+` menjadi spasi. |
| `http_form_encode(pairs)` | Meng-encode nilai dari `"a=1&b=2"` dan mempertahankan kuncinya. |
| `http_with_query(url, query)` | Menambahkan query dengan `?` atau `&` sesuai kebutuhan. |

404 adalah respons normal: isinya tetap Anda terima dan `http_last_status()` bernilai `404`.

```fenneth
import "std/http";

fn main() -> i32 {
    let q: String = http_form_encode("q=hello world");
    let body: String = http_get(http_with_query("https://example.com/search", q));
    if !http_last_ok() {
        print("permintaan gagal: " + http_last_error());
        return 1;
    }
    print(len(body));
    return 0;
}
```

## std/zip

```fenneth
import "std/zip";
```

Membaca dan menulis arsip `.zip`. Pembaca menangani arsip normal apa pun (entri stored maupun deflated, termasuk biner di dalamnya); penulis menghasilkan arsip standar dengan entri stored (tanpa kompresi), jadi tetap valid di mana saja tetapi lebih besar daripada versi deflated.

| Fungsi | Keterangan |
|---|---|
| `zip_list(archive)` | Nama entri, satu per baris (`""` bila arsip tidak bisa dibaca). |
| `zip_entries(archive)` | Sama, sebagai `[String]`. |
| `zip_contains(archive, name)` | Apakah ada entri dengan nama ini? |
| `zip_entry_size(archive, name)` | Ukuran setelah dekompresi, atau `-1`. |
| `zip_read(archive, name)` | Satu entri sebagai `String` (`""` bila tidak ada). |
| `zip_extract(archive, dest_dir)` | Membongkar semua entri di bawah `dest_dir`, membuat folder yang diperlukan. |
| `zip_extract_file(archive, name, dest)` | Membongkar satu entri ke path tertentu. |
| `zip_create(archive, entries)` | Membuat arsip: `"nama=path"` per baris, `"nama="` untuk entri kosong. |
| `zip_create_files(archive, names, sources)` | Sama, dari dua array yang sepadan. |
| `zip_last_error()` | Penyebab kegagalan terakhir (`""` bila berhasil). |

```fenneth
import "std/zip";
import "std/fs";

fn main() -> i32 {
    let arc: String = fs_join(fs_temp_dir(), "out.zip");
    if !zip_create(arc, "VERSION=\nreadme.txt=README.md") {
        print(zip_last_error());
        return 1;
    }
    zip_extract(arc, fs_join(fs_temp_dir(), "unpacked"));
    print(zip_contains(arc, "readme.txt"));
    return 0;
}
```

## Batasan

- Tanpa generics, setiap fungsi array ada dalam tiga salinan, dan tidak ada `array_sort` untuk array struct. Tulis loop sendiri (atau merge sort seperti yang ditunjukkan `fenneth std array`) untuk itu.
- Belum ada hash map dan belum ada fungsi string yang paham Unicode.
- `std/http` butuh perintah `curl` (atau `wget`) di `PATH` saat program berjalan; tidak ada pustaka tambahan yang di-link.
- `std/zip` membaca entri stored maupun deflated (arsip normal apa pun) tetapi menulis entri stored tanpa kompresi.
