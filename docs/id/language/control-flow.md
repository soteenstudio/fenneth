# Alur kontrol

Kondisi ditulis tanpa kurung wajib, dan badan selalu memakai kurung kurawal. Kondisi harus `bool`: tidak ada `i32` yang dianggap "benar" atau "salah".

## if / else if / else

```fenneth
fn sign(n: i32) -> String {
    if n < 0 {
        return "negatif";
    } else if n == 0 {
        return "nol";
    } else {
        return "positif";
    }
}
```

## while

```fenneth
let i: i32 = 0;
while i < 3 {
    print(i);
    i += 1;
}
```

## for pada rentang

`for i in a..b` menghitung dari `a` sampai **sebelum** `b` (batas atas eksklusif). Kedua batas bertipe `i32`. Variabel loop bertipe `i32` dan tidak bisa di-assign.

```fenneth
for i in 0..3 {      // 0, 1, 2
    print(i);
}
```

## for pada array

```fenneth
let names: [String] = ["ada", "lin", "kai"];
for n in names {
    print(n);
}
```

## break dan continue

`break` keluar dari loop terdalam, `continue` melompat ke iterasi berikutnya. Memakai keduanya di luar loop adalah error kompilasi.

```fenneth
for i in 0..10 {
    if i % 2 == 0 { continue; }
    if i > 7 { break; }
    print(i);        // 1, 3, 5, 7
}
```

## return

`return nilai;` keluar dari fungsi dengan nilai; `return;` keluar dari fungsi tanpa tipe kembalian. Fungsi bertipe kembalian harus `return` di **semua jalur**. `panic()` dan `exit()` tidak dihitung sebagai `return`.

```fenneth
fn abs(n: i32) -> i32 {
    if n < 0 {
        return -n;
    }
    return n;        // wajib: compiler memeriksa semua jalur
}
```
