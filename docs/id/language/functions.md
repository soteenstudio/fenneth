# Fungsi

```fenneth
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

fn greet(name: String) {          // tanpa `->`: tidak mengembalikan nilai
    print("halo, " + name);
}
```

- Parameter butuh anotasi tipe.
- Tanpa `-> T` berarti fungsi tidak mengembalikan nilai. Memanggilnya sebagai bagian dari ekspresi adalah error tipe.
- Nama fungsi harus unik dan tidak boleh memakai nama [fungsi bawaan](/id/language/builtins).
- Compiler memeriksa jumlah dan tipe argumen pada setiap pemanggilan. Tidak ada argumen default maupun variadic.
- Rekursi diperbolehkan.

```fenneth
fn fib(n: i32) -> i32 {
    if n < 2 {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}
```

## Argumen dilewatkan per nilai

`i32`, `f64`, `bool`, dan **struct** disalin saat dilewatkan. **Array** berbagi referensi, sehingga fungsi bisa mengubah array milik pemanggil. Nilai `String` tidak bisa diubah.

```fenneth
struct Counter { n: i32, }

fn bump(c: Counter) { c.n = c.n + 1; }       // mengubah salinan
fn append(xs: [i32]) { push(xs, 1); }        // mengubah array milik pemanggil

fn main() -> i32 {
    let c: Counter = Counter { n: 0 };
    bump(c);
    print(c.n);               // 0

    let xs: [i32] = [];
    append(xs);
    print(len(xs));          // 1
    return 0;
}
```

## main

`main` tidak menerima parameter dan mengembalikan `i32`. Lihat [Dasar-dasar](/id/language/basics#struktur-program).

## extern fn

`extern fn` mendeklarasikan fungsi yang diimplementasikan di C (atau Rust). Tidak punya badan. Lihat [Modul dan FFI](/id/language/modules-ffi).

```fenneth
extern fn host_add(a: i32, b: i32) -> i32;
```
