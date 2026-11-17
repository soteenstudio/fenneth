# Fungsi bawaan

Fungsi bawaan selalu tersedia dan tidak bisa didefinisikan ulang. Setiap pemanggilan diperiksa tipenya.

## Output

| Fungsi | Keterangan |
|---|---|
| `print(x)` | Cetak `x` diikuti baris baru. `x` bertipe `i32`, `f64`, `bool`, atau `String`. |
| `write(x)` | Sama seperti `print` tetapi **tanpa** baris baru. |
| `eprint(x)` | Sama seperti `print` tetapi ke **stderr**. |

## Konversi

| Fungsi | Keterangan |
|---|---|
| `to_str(x)` | `i32`, `f64`, atau `bool` menjadi `String`. |
| `to_f64(x)` | `i32` menjadi `f64`. |
| `to_i32(x)` | `f64` menjadi `i32`. |

Tidak ada konversi numerik implisit, jadi pakai fungsi ini setiap kali Anda mencampur `i32` dan `f64`.

## String

| Fungsi | Keterangan |
|---|---|
| `len(s)` | Panjang `String` dalam byte (juga berlaku untuk array). |
| `str_at(s, i)` | Byte pada indeks `i` sebagai `i32` (0..255). `panic` bila di luar rentang. |
| `substr(s, a, b)` | Byte `[a, b)` sebagai `String` baru. `panic` bila rentang tidak valid. |
| `char_str(code)` | `String` satu karakter untuk `code` dalam 0..255. Selain itu `panic`. |
| `str_to_i32(s)` | Parse bilangan bulat desimal. Mengembalikan `0` bila `s` bukan angka. |

## Array

| Fungsi | Keterangan |
|---|---|
| `len(a)` | Jumlah elemen. |
| `push(a, v)` | Tambahkan `v` (tipenya harus sama dengan tipe elemen array). |
| `pop(a)` | Hapus dan kembalikan elemen terakhir. `panic` pada array kosong. |

## Berkas dan input

| Fungsi | Keterangan |
|---|---|
| `read_file(path)` | Seluruh isi berkas sebagai `String`. Mengembalikan `""` bila berkas tidak bisa dibuka. |
| `write_file(path, s)` | Tulis `s` ke `path`, menimpa isi lama. Mengembalikan `true` bila berhasil. |
| `file_exists(path)` | `true` bila berkas bisa dibuka untuk dibaca. |
| `read_line()` | Satu baris dari stdin tanpa pemisah baris. Mengembalikan `""` saat input habis. |
| `args()` | Argumen program sebagai `[String]` (lihat [CLI](/id/guide/cli)). |

::: tip
`read_line()` mengembalikan `""` baik untuk baris kosong maupun saat input habis, sehingga loop seperti `while len(line) > 0` juga berhenti pada baris kosong pertama.
:::

## Kontrol program

| Fungsi | Keterangan |
|---|---|
| `exit(code)` | Hentikan program dengan exit code `code`. |
| `panic(msg)` | Cetak `panic: msg` ke stderr dan keluar dengan kode `101`. |

`exit` maupun `panic` tidak dihitung sebagai `return` pada pemeriksaan "return di semua jalur".
