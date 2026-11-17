# Modules and FFI

## import

```fenneth
import "geometry";        // loads geometry.fen or geometry.fenneth
```

- The path is relative to the file that contains the `import`.
- If the path has no extension, `.fen` is tried first, then `.fenneth`.
- Every file is loaded once. Import cycles are tolerated.
- All modules are merged into one program: a module's items come before those of the file that imports it, and every name lives in one shared namespace, so names must be unique across files.
- Error messages name the file they come from (`file:line:column`).

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

`extern fn` declares a function with no body that is implemented in C, or in Rust with a C ABI. The linker must be given the library that provides it (`--link`).

```fenneth
extern fn host_add(a: i32, b: i32) -> i32;
```

Allowed types in an `extern fn` signature: `i32`, `f64`, `bool` and `String` (and no return type). Arrays and structs are not allowed. `String` is a `const char*` on the C side, which is `*const c_char` in Rust.

## Using Rust from Fenneth

Create the Rust side as a static library:

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

Build the library and link it:

```sh
rustc --edition 2021 --crate-type staticlib -O host.rs -o libhost.a
fenneth run ffi.fen --link ./libhost.a --link -lpthread --link -ldl --link -lm
```

The system libraries a Rust static library needs differ per platform (on Windows they are different), so adjust the `--link` arguments. A function returning a `String` must return a NUL-terminated string that stays valid (for example a static one), because Fenneth never frees it.

## Using C from Fenneth

The same works with a C library. The C function must match the declared signature:

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
