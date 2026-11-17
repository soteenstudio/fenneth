# Functions

```fenneth
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

fn greet(name: String) {          // no `->`: returns nothing
    print("hello, " + name);
}
```

- Parameters need type annotations.
- Leaving out `-> T` means the function returns nothing. Calling it as part of a larger expression is a type error.
- Function names must be unique and cannot reuse a [built-in](/language/builtins) name.
- The compiler checks the number and types of arguments at every call. There are no default or variadic arguments.
- Recursion is allowed.

```fenneth
fn fib(n: i32) -> i32 {
    if n < 2 {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}
```

## Arguments are passed by value

`i32`, `f64`, `bool` and **structs** are copied when passed. **Arrays** share a reference, so a function can modify the caller's array. `String` values are immutable.

```fenneth
struct Counter { n: i32, }

fn bump(c: Counter) { c.n = c.n + 1; }       // changes a copy
fn append(xs: [i32]) { push(xs, 1); }        // changes the caller's array

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

`main` takes no parameters and returns `i32`. See [Basics](/language/basics#program-structure).

## extern fn

An `extern fn` declares a function implemented in C (or Rust). It has no body. See [Modules and FFI](/language/modules-ffi).

```fenneth
extern fn host_add(a: i32, b: i32) -> i32;
```
