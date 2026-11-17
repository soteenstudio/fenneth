# Control flow

Conditions are written without required parentheses, and bodies always use braces. Conditions must be `bool`: there is no "truthy" `i32`.

## if / else if / else

```fenneth
fn sign(n: i32) -> String {
    if n < 0 {
        return "negative";
    } else if n == 0 {
        return "zero";
    } else {
        return "positive";
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

## for over a range

`for i in a..b` counts from `a` up to, but **not including**, `b`. Both bounds are `i32`. The loop variable has type `i32` and cannot be assigned to.

```fenneth
for i in 0..3 {      // 0, 1, 2
    print(i);
}
```

## for over an array

```fenneth
let names: [String] = ["ada", "lin", "kai"];
for n in names {
    print(n);
}
```

## break and continue

`break` leaves the innermost loop, `continue` jumps to its next iteration. Using either outside a loop is a compile error.

```fenneth
for i in 0..10 {
    if i % 2 == 0 { continue; }
    if i > 7 { break; }
    print(i);        // 1, 3, 5, 7
}
```

## return

`return value;` leaves a function with a value; `return;` leaves a function that has no return type. A function with a return type must `return` on **every path**. `panic()` and `exit()` do not count as a `return`.

```fenneth
fn abs(n: i32) -> i32 {
    if n < 0 {
        return -n;
    }
    return n;        // required: the compiler checks all paths
}
```
