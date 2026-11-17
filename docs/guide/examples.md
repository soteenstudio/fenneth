# Code examples

Save each program in a file ending in `.fen` and run it with `fenneth file.fen`.

## Hello, world

```fenneth
fn main() -> i32 {
    print("Hello, Fenneth!");
    return 0;
}
```

## FizzBuzz

```fenneth
fn main() -> i32 {
    for i in 1..16 {                 // 1 to 15
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

## Recursion

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

## Structs and arrays

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

## Strings

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

This works on bytes, so it is meant for ASCII text.

## Command-line arguments

```fenneth
fn main() -> i32 {
    let a: [String] = args();
    print("received " + to_str(len(a)) + " argument(s)");
    for s in a {
        print(s);
    }
    return 0;
}
```

```sh
fenneth run args.fen -- one two
```

## Reading input

```fenneth
fn main() -> i32 {
    let total: i32 = 0;
    let line: String = read_line();
    while len(line) > 0 {            // stops at end of input or a blank line
        total += str_to_i32(line);
        line = read_line();
    }
    print("sum = " + to_str(total));
    return 0;
}
```

```sh
printf '10\n20\n12\n' | fenneth sum.fen     # sum = 42
```

## Files

```fenneth
fn main() -> i32 {
    if !write_file("note.txt", "saved by Fenneth\n") {
        eprint("could not write note.txt");
        return 1;
    }
    if file_exists("note.txt") {
        write(read_file("note.txt"));
    }
    return 0;
}
```

## Errors and exit codes

```fenneth
fn main() -> i32 {
    let xs: [i32] = [1, 2, 3];
    print(xs[5]);        // panic: index 5 out of bounds (length 3), exit code 101
    return 0;
}
```

## Multiple files and Rust

See [Modules and FFI](/language/modules-ffi) for `import` and for calling Rust or C code.
