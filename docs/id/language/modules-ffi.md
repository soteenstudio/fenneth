# Modul dan FFI

## import

```fenneth
import "geometry";        // memuat geometry.fen atau geometry.fenneth
```

- Path relatif terhadap file yang berisi `import`.
- Jika path tanpa ekstensi, `.fen` dicoba lebih dulu, lalu `.fenneth`.
- Setiap file dimuat satu kali. Siklus import ditoleransi.
- Semua modul digabung menjadi satu program: item modul ditempatkan sebelum item file yang meng-import-nya, dan semua nama berada di satu namespace bersama, jadi nama harus unik di seluruh file.
- Pesan error menyebut file asalnya (`file:baris:kolom`).
- Path yang diawali `std/` bukan file: ia memuat modul [pustaka standar](/id/language/stdlib) bawaan, misalnya `import "std/math";`.

**geometry.fen**

```fenneth
struct Point { x: i32, y: i32, }

fn dist2(a: Point, b: Point) -> i32 {
    let dx: i32 = a.x - b.x;
    let dy: i32 = a.y - b.y;
    return dx * dx + dy * dy;
}
```

**main.fen**

```fenneth
import "geometry";

fn main() -> i32 {
    print(dist2(Point { x: 0, y: 0 }, Point { x: 3, y: 4 }));    // 25
    return 0;
}
```

```sh
fenneth main.fen
```

## extern fn (FFI)

`extern fn` mendeklarasikan fungsi tanpa badan yang diimplementasikan di C, atau di Rust dengan C ABI. Linker harus diberi library yang menyediakannya (`--link`). (`fenneth run` dan `fenneth build` selalu me-link pustaka matematika C, `-lm`, di luar itu.)

```fenneth
extern fn host_add(a: i32, b: i32) -> i32;
```

Tipe yang boleh pada tanda tangan `extern fn`: `i32`, `f64`, `bool`, dan `String` (serta tanpa tipe kembalian). Array dan struct tidak boleh. `String` adalah `const char*` di sisi C, yaitu `*const c_char` di Rust.

## Memakai Rust dari Fenneth

Buat sisi Rust sebagai static library:

**host.rs**

```rust
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn host_add(a: i32, b: i32) -> i32 {
    a + b
}

#[no_mangle]
pub extern "C" fn host_half(x: f64) -> f64 {
    x / 2.0
}

#[no_mangle]
pub extern "C" fn host_banner() -> *const c_char {
    b"Hello from Rust, Fenneth!\0".as_ptr() as *const c_char
}
```

**ffi.fen**

```fenneth
extern fn host_add(a: i32, b: i32) -> i32;
extern fn host_half(x: f64) -> f64;
extern fn host_banner() -> String;

fn main() -> i32 {
    print(host_add(40, 2));      // 42
    print(host_half(3.0));       // 1.5
    print(host_banner());        // Hello from Rust, Fenneth!
    return 0;
}
```

Bangun library lalu tautkan:

```sh
rustc --edition 2021 --crate-type staticlib -O host.rs -o libhost.a
fenneth run ffi.fen --link ./libhost.a --link -lpthread --link -ldl --link -lm
```

Library sistem yang dibutuhkan static library Rust berbeda per platform (di Windows berbeda), jadi sesuaikan argumen `--link`. Fungsi yang mengembalikan `String` harus mengembalikan string berakhiran NUL yang tetap valid (misalnya string statis), karena Fenneth tidak pernah membebaskannya.

## Memakai C dari Fenneth

Cara yang sama berlaku untuk library C. Fungsi C harus cocok dengan tanda tangan yang dideklarasikan:

```c
// mathx.c
double mathx_square(double x) { return x * x; }
```

```fenneth
extern fn mathx_square(x: f64) -> f64;

fn main() -> i32 {
    print(mathx_square(1.5));    // 2.25
    return 0;
}
```

```sh
gcc -c mathx.c -o mathx.o
fenneth run main.fen --link ./mathx.o
```
