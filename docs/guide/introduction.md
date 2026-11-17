# Introduction

Fenneth is a statically typed programming language with a modern syntax. A Fenneth program is compiled to **C99**, and then GCC or Clang compiles that C into a native executable. The compiler is written in Rust and has no external dependencies.

```
.fen / .fenneth → Lexer → Parser (AST) → Type checker → C code generator → .c → gcc/clang → executable
```

Source files use the extension **`.fen`** or **`.fenneth`**. They are equivalent everywhere, including in `import`.

## What it looks like

```fenneth
struct Point { x: i32, y: i32, }

fn dist2(a: Point, b: Point) -> i32 {
    let dx: i32 = a.x - b.x;
    let dy: i32 = a.y - b.y;
    return dx * dx + dy * dy;
}

fn main() -> i32 {
    let p: Point = Point { x: 3, y: 4 };
    print("distance squared = " + to_str(dist2(p, Point { x: 0, y: 0 })));
    return 0;
}
```

## Design in a nutshell

- **Explicit types.** Every variable has a type annotation and an initial value. There is no implicit conversion between `i32` and `f64`.
- **Small surface.** `struct`, arrays, `String`, `if` / `while` / `for`, functions, modules and FFI. No generics, enums, or closures yet.
- **Value structs, reference arrays.** Structs are copied on assignment; arrays share a reference.
- **Checked at runtime.** Out-of-bounds indexing, `pop` on an empty array, division by zero and bad `substr` ranges stop the program with `panic` (exit code 101).
- **Good diagnostics.** Errors carry `file:line:column`, and the type checker reports all of its errors together.

## What you need

| Task | Needs a C compiler? |
|---|---|
| `fenneth run`, `fenneth build` | Yes (GCC, Clang or `cc`) |
| `fenneth emit`, `fenneth check` | No |

Next: [install Fenneth](/guide/installation), then try the [code examples](/guide/examples).
