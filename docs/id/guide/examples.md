# Contoh kode

Simpan setiap program dalam file berakhiran `.fen` lalu jalankan dengan `fenneth file.fen`.

## Hello, world

```fenneth
fn main() -> i32 {
    print("Halo, Fenneth!");
    return 0;
}
```

## FizzBuzz

```fenneth
fn main() -> i32 {
    for i in 1..16 {                 // 1 sampai 15
        if i % 15 == 0 {
            print("FizzBuzz");
        } else if i % 3 == 0 {
            print("Fizz");
        } else if i % 5 == 0 {
            print("Buzz");
        } else {
            print(i);
        }
    }
    return 0;
}
```

## Rekursi

```fenneth
fn fib(n: i32) -> i32 {
    if n < 2 {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}

fn main() -> i32 {
    for i in 0..10 {
        write(to_str(fib(i)) + " ");
    }
    print("");                       // 0 1 1 2 3 5 8 13 21 34
    return 0;
}
```

## Struct dan array

```fenneth
struct Point { x: i32, y: i32, }

const LIMIT: i32 = 3;

fn dist2(a: Point, b: Point) -> i32 {
    let dx: i32 = a.x - b.x;
    let dy: i32 = a.y - b.y;
    return dx * dx + dy * dy;
}

fn main() -> i32 {
    let pts: [Point] = [];
    for i in 0..LIMIT {                  // 0, 1, 2
        push(pts, Point { x: i, y: i * 2 });
    }
    let total: i32 = 0;
    for p in pts {
        total += dist2(p, Point { x: 0, y: 0 });
    }
    print("total = " + to_str(total));   // total = 25
    return 0;
}
```

## String

```fenneth
fn reverse(s: String) -> String {
    let out: String = "";
    let i: i32 = len(s) - 1;
    while i >= 0 {
        out = out + char_str(str_at(s, i));
        i -= 1;
    }
    return out;
}

fn main() -> i32 {
    print(reverse("Fenneth"));           // htenneF
    return 0;
}
```

Program ini bekerja per byte, jadi cocok untuk teks ASCII.

## Argumen baris perintah

```fenneth
fn main() -> i32 {
    let a: [String] = args();
    print("menerima " + to_str(len(a)) + " argumen");
    for s in a {
        print(s);
    }
    return 0;
}
```

```sh
fenneth run args.fen -- satu dua
```

## Membaca input

```fenneth
fn main() -> i32 {
    let total: i32 = 0;
    let line: String = read_line();
    while len(line) > 0 {            // berhenti saat input habis atau baris kosong
        total += str_to_i32(line);
        line = read_line();
    }
    print("jumlah = " + to_str(total));
    return 0;
}
```

```sh
printf '10\n20\n12\n' | fenneth sum.fen     # jumlah = 42
```

## Berkas

```fenneth
fn main() -> i32 {
    if !write_file("note.txt", "disimpan oleh Fenneth\n") {
        eprint("tidak bisa menulis note.txt");
        return 1;
    }
    if file_exists("note.txt") {
        write(read_file("note.txt"));
    }
    return 0;
}
```

## Error dan exit code

```fenneth
fn main() -> i32 {
    let xs: [i32] = [1, 2, 3];
    print(xs[5]);        // panic: index 5 out of bounds (length 3), exit code 101
    return 0;
}
```

## Banyak file dan Rust

Lihat [Modul dan FFI](/id/language/modules-ffi) untuk `import` dan cara memanggil kode Rust atau C.
