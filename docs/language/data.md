# Structs, arrays and strings

## Structs

```fenneth
struct Point { x: i32, y: i32, }

fn main() -> i32 {
    let p: Point = Point { x: 1, y: 2 };
    p.x = 10;
    print(p.x + p.y);        // 12
    return 0;
}
```

- A struct name **must start with an uppercase letter** and cannot be a built-in type name.
- A struct needs at least one field. Field names must be unique.
- A literal must set **every** field exactly once; the order does not matter. A trailing comma is allowed.
- A struct cannot contain itself directly, but it can hold an array of itself, which is how you build trees:

```fenneth
struct Node { value: i32, children: [Node], }
```

**Structs are copied by value** when assigned and when passed to a function. Modifying the copy does not change the original.

## Arrays

`[T]` is a growable array. Arrays can be nested: `[[i32]]`.

```fenneth
let xs: [i32] = [1, 2, 3];
let empty: [String] = [];          // an annotation is required for []
let grid: [[i32]] = [[1, 2], [3, 4]];

push(xs, 4);                       // [1, 2, 3, 4]
let last: i32 = pop(xs);           // 4
xs[0] = 9;
print(len(xs));                    // 3
print(grid[1][0]);                 // 3
```

- All elements have the same type; a mixed literal is a compile error.
- The index must be `i32`. An out-of-bounds index panics (exit code 101).
- `pop` on an empty array panics.
- **Arrays share a reference**: `let b: [i32] = a;` makes `b` and `a` the same array.
- Iterate with `for x in xs { ... }`.

## Strings

`String` is an immutable sequence of bytes. Indexing and lengths count **bytes**, not Unicode characters.

```fenneth
let s: String = "hello";
let t: String = s + ", world";        // joining creates a new string
print(len(t));                        // 12
print(str_at(s, 0));                  // 104  (byte of 'h')
print(substr(s, 1, 4));               // ell  (bytes 1, 2, 3)
print(char_str(65));                  // A
print(s == "hello");                  // true
```

| Task | How |
|---|---|
| Join | `a + b` |
| Length (bytes) | `len(s)` |
| Byte at index | `str_at(s, i)` returns an `i32` in 0..255 |
| Slice `[a, b)` | `substr(s, a, b)` |
| One-character string | `char_str(code)` with `code` in 0..255 |
| Equality | `==`, `!=` |
| Number to string | `to_str(n)` |
| String to integer | `str_to_i32(s)` |

`s[i]` indexing on strings is not available; use `str_at`.
