# Standard library

Fenneth ships with a small standard library. It is written in Fenneth itself and built into the compiler, so there is nothing to install and no file to find. Load a module with `import`:

```fenneth
import "std/math";
import "std/string";

fn main() -> i32 {
    print(math_sqrt(2.0));                       // 1.41421
    print(string_to_upper("hello"));             // HELLO
    print(string_fixed(MATH_PI, 2));             // 3.14
    return 0;
}
```

| Module | Contents |
|---|---|
| [`std/math`](#std-math) | Absolute value, min/max, powers, gcd, square root, trigonometry, rounding |
| [`std/string`](#std-string) | Searching, splitting, joining, trimming, replacing, padding, number parsing and formatting |
| [`std/char`](#std-char) | Classifying and converting single bytes |
| [`std/array`](#std-array) | Sorting, searching, slicing and summing arrays of `i32`, `f64` and `String` |
| [`std/fs`](#std-fs) | Directories, file metadata, copying, renaming, removing, and path helpers |
| [`std/http`](#std-http) | Fetching URLs over HTTP(S): GET, POST, downloads, URL encoding |
| [`std/zip`](#std-zip) | Reading, unpacking and creating `.zip` archives |

Run `fenneth std` to list the modules and `fenneth std math` to print the source of one.

## How it works

- **One shared namespace.** Fenneth has no namespaces, so every public name starts with the name of its module: `math_abs`, `string_split`, `array_sort_i32`. Names that start with an underscore (`_array_msort_i32`) are private helpers; do not call them.
- **The `std/` prefix is reserved.** `import "std/math";` always means the built-in module, even if a `std` folder exists next to your file.
- **Opt-in.** A program that does not import a module does not see its names, so your own `max` or `math_abs` stays yours. If you import a module and also define one of its names, the compiler reports `function '...' is defined more than once`.
- **Only what you use is compiled.** Std functions that your program never reaches are dropped from the generated C. Everything in an imported module is still type-checked. `std/fs`, `std/http` and `std/zip` also carry their own C, which is spliced into the output only when a function from that module survives.
- **Modules can import modules.** `std/array` imports `std/string`, so `string_compare` is available after `import "std/array";`.
- **C library.** `std/math` uses the C math library. `fenneth run` and `fenneth build` link it (`-lm`) on Linux, macOS and Termux. If you compile the output of `fenneth emit` yourself, add `-lm`.
- **Errors.** Misuse such as `array_min_i32` on an empty array stops the program with `panic: ...` and exit code 101, like the [runtime checks](/language/limitations).

## std/math

```fenneth
import "std/math";
```

Constants: `MATH_PI`, `MATH_TAU`, `MATH_E` (`f64`), `MATH_I32_MAX`, `MATH_I32_MIN` (`i32`).

**Integers (`i32`)** — arithmetic wraps on overflow, as everywhere in Fenneth.

| Function | Description |
|---|---|
| `math_abs(x)` | Absolute value. `math_abs(MATH_I32_MIN)` wraps and stays negative. |
| `math_min(a, b)`, `math_max(a, b)` | Smaller / larger of two numbers. |
| `math_clamp(x, lo, hi)` | Limit `x` to `lo..=hi`. Panics when `lo > hi`. |
| `math_sign(x)` | `-1`, `0` or `1`. |
| `math_mod(a, b)` | Modulo that is never negative for a positive `b`: `math_mod(-7, 3)` is `2`. Panics when `b` is 0. |
| `math_ipow(base, exp)` | `base` to the power `exp`. Panics when `exp < 0`. |
| `math_gcd(a, b)`, `math_lcm(a, b)` | Greatest common divisor, least common multiple. |

**Floating point (`f64`)**

| Function | Description |
|---|---|
| `math_absf(x)`, `math_minf(a, b)`, `math_maxf(a, b)`, `math_clampf(x, lo, hi)` | The `f64` twins of the functions above. |
| `math_is_nan(x)` | `true` for "not a number", for example `math_sqrt(-1.0)`. |
| `math_sqrt(x)`, `math_pow(x, y)`, `math_exp(x)` | Square root, power, `e` to the power `x`. |
| `math_log(x)`, `math_log10(x)` | Natural and base-10 logarithm. |
| `math_floor(x)`, `math_ceil(x)`, `math_trunc(x)`, `math_round(x)` | Rounding, giving an `f64`. `math_round` rounds halves away from zero. |
| `math_floor_i32(x)`, `math_ceil_i32(x)`, `math_round_i32(x)` | The same, giving an `i32` (the result must fit in an `i32`). |
| `math_sin(x)`, `math_cos(x)`, `math_tan(x)`, `math_atan(x)`, `math_atan2(y, x)` | Trigonometry in radians. |
| `math_to_radians(d)`, `math_to_degrees(r)` | Convert between degrees and radians. |
| `math_hypot(x, y)` | `sqrt(x*x + y*y)`. |

## std/string

```fenneth
import "std/string";
```

Strings are byte strings: lengths and indices count bytes, and the case functions only change ASCII letters. No function modifies its argument.

**Inspecting**

| Function | Description |
|---|---|
| `string_is_empty(s)` | `true` for `""`. |
| `string_compare(a, b)` | `-1`, `0` or `1`, comparing byte by byte (`"Z"` comes before `"a"`). |
| `string_starts_with(s, prefix)`, `string_ends_with(s, suffix)` | Prefix / suffix test. |
| `string_index_of(s, sub)` | Position of the first `sub`, or `-1`. |
| `string_index_from(s, sub, from)` | Same, starting at byte `from`. Panics when `from` is outside `0..=len(s)`. |
| `string_contains(s, sub)` | `true` if `sub` occurs in `s`. |
| `string_count(s, sub)` | Number of non-overlapping occurrences. |

**Transforming**

| Function | Description |
|---|---|
| `string_to_upper(s)`, `string_to_lower(s)` | Change ASCII letter case. |
| `string_reverse(s)` | Reverse the bytes. |
| `string_trim(s)`, `string_trim_start(s)`, `string_trim_end(s)` | Remove whitespace at both ends / the start / the end. |
| `string_repeat(s, n)` | `s` written `n` times. Panics when `n < 0`. |
| `string_replace(s, from, to)` | Replace every occurrence of `from`. |
| `string_pad_left(s, width, fill)`, `string_pad_right(s, width, fill)` | Pad to `width` bytes. `fill` must be a one-byte `String`. |

**Splitting and joining**

| Function | Description |
|---|---|
| `string_split(s, sep)` | `[String]`. `string_split("a,b,,c", ",")` is `["a", "b", "", "c"]`. An empty `sep` splits into single bytes. |
| `string_lines(s)` | The lines of `s`. Handles `\n` and `\r\n`; a final line ending does not add an empty line. |
| `string_join(parts, sep)` | Join a `[String]` with `sep`. |

**Numbers**

| Function | Description |
|---|---|
| `string_is_int(s)` | `true` if `s` is a decimal `i32` (optional sign). Check this before `str_to_i32`, which returns `0` for bad input. |
| `string_parse_f64(s)` | Parse a number such as `"3.25"` or `"-1e3"`. Gives `0.0` when `s` is not a number. |
| `string_fixed(x, decimals)` | Format with exactly `decimals` digits (0..40): `string_fixed(3.14159, 2)` is `"3.14"`. |

```fenneth
import "std/string";

fn main() -> i32 {
    let text: String = read_file("data.txt");
    for line in string_lines(text) {
        let fields: [String] = string_split(string_trim(line), ",");
        if len(fields) == 2 && string_is_int(fields[1]) {
            print(fields[0] + " = " + to_str(str_to_i32(fields[1])));
        }
    }
    return 0;
}
```

## std/char

```fenneth
import "std/char";
```

A "char" is a byte value as returned by `str_at` (0..255). Only ASCII is classified. Use `char_str` to turn a byte back into a `String`.

| Function | Description |
|---|---|
| `char_is_digit(c)`, `char_is_upper(c)`, `char_is_lower(c)` | `0-9`, `A-Z`, `a-z`. |
| `char_is_alpha(c)`, `char_is_alnum(c)` | Letters / letters and digits. |
| `char_is_space(c)` | Space, tab, newline, carriage return, vertical tab, form feed. |
| `char_to_upper(c)`, `char_to_lower(c)` | Change the case of an ASCII letter; other bytes are returned unchanged. |
| `char_digit_value(c)` | `'0'..'9'` is 0..9, letters are 10..35 (any case), anything else is `-1`. |

## std/array

```fenneth
import "std/array";
```

Fenneth has no generics, so each function exists once per element type, with the type as a suffix: `_i32`, `_f64` and `_str` (for `[String]`). Below, `T` stands for that suffix. Appending, removing and measuring are built in (`push`, `pop`, `len`).

Arrays are shared when assigned, so functions marked *in place* change the array you pass in; the others return a new array.

| Function | Description |
|---|---|
| `array_sort_T(a)` | Sort *in place*, smallest first. Stable, `O(n log n)`. Strings are ordered byte by byte. |
| `array_is_sorted_T(a)` | `true` if no element is smaller than the one before it. |
| `array_reverse_T(a)` | Reverse *in place*. |
| `array_copy_T(a)` | A new array with the same elements. |
| `array_slice_T(a, from, to)` | Elements `[from, to)` as a new array. Panics when the range is not inside `a`. |
| `array_concat_T(a, b)` | A new array: the elements of `a`, then those of `b`. |
| `array_fill_T(n, v)` | A new array of `n` copies of `v`. Panics when `n < 0`. |
| `array_contains_T(a, v)` | `true` if some element equals `v`. |
| `array_index_of_T(a, v)` | Index of the first element equal to `v`, or `-1`. |
| `array_min_T(a)`, `array_max_T(a)` | Smallest / largest element. Panic on an empty array. |
| `array_sum_i32(a)`, `array_sum_f64(a)` | Sum of the elements (`0` for an empty array). Only for numbers. |
| `array_range(start, end)` | `[i32]` holding `start, start+1, ..., end-1`. |

```fenneth
import "std/array";

fn main() -> i32 {
    let words: [String] = ["pear", "apple", "fig"];
    array_sort_str(words);
    print(words[0]);                          // apple
    print(array_sum_i32(array_range(1, 5)));  // 10
    return 0;
}
```

## std/fs

```fenneth
import "std/fs";
```

Files and directories. The built-ins `read_file`, `write_file` and `file_exists` still do the whole-file cases; this module adds directories, metadata, copying and path handling. Paths are ordinary `String`s. Functions that can fail return `false` (or `-1`), never a panic.

**Asking about a path**

| Function | Description |
|---|---|
| `fs_exists(path)` | `true` if something is there: file, directory or link. |
| `fs_is_dir(path)`, `fs_is_file(path)` | Is it a directory / a regular file? |
| `fs_size(path)` | Size in bytes, or `-1` when it is not a readable file. |

**Changing the file system**

| Function | Description |
|---|---|
| `fs_mkdir(path)` | Create one directory. `true` if created or already there. |
| `fs_mkdir_all(path)` | Create `path` and every missing parent: `fs_mkdir_all("a/b/c")`. |
| `fs_remove(path)` | Delete a file (or an empty directory). |
| `fs_remove_dir(path)` | Delete an empty directory. |
| `fs_rename(from, to)` | Move, replacing `to` when it exists. |
| `fs_copy(from, to)` | Copy the bytes of one file to another. |
| `fs_append(path, text)` | Append text, creating the file when needed. |

**Directories**

| Function | Description |
|---|---|
| `fs_list(path)` | Entry names as one line per name (`""` when it cannot be read). |
| `fs_entries(path)` | The same, already split into a `[String]`. |

**Places**

| Function | Description |
|---|---|
| `fs_cwd()` | The current working directory. |
| `fs_temp_dir()` | `$TMPDIR` / `%TEMP%`, or `/tmp`. |
| `fs_temp_path(prefix)` | A fresh unused path inside the temp directory (nothing is created). |

**Paths**

| Function | Description |
|---|---|
| `fs_separator()` | `"\\"` on Windows, `"/"` elsewhere. |
| `fs_join(a, b)` | Join with a separator unless `b` is absolute or `a` already ends with one. |
| `fs_basename(path)` | Last part: `"a/b/c.txt"` gives `"c.txt"`. |
| `fs_dirname(path)` | Everything but the last part: `"a/b/c.txt"` gives `"a/b"`. |
| `fs_extension(path)` | Text after the last dot, without the dot (`"gz"`). |

```fenneth
import "std/fs";

fn main() -> i32 {
    let dir: String = fs_join(fs_temp_dir(), "demo");
    fs_mkdir_all(fs_join(dir, "sub"));
    write_file(fs_join(dir, "a.txt"), "hello\n");
    for name in fs_entries(dir) {
        print(name + ": " + to_str(fs_size(fs_join(dir, name))));
    }
    return 0;
}
```

## std/http

```fenneth
import "std/http";
```

Fetch URLs over HTTP and HTTPS. Requests are run by the `curl` command (Windows 10+, macOS and most Linux systems have it); `wget` is used when curl is missing, and only with plain URLs. Nothing is linked, but the program needs one of them on `PATH` when it runs.

| Function | Description |
|---|---|
| `http_get(url)` | GET `url`; the response body (`""` when the request failed). |
| `http_post(url, content_type, body)` | POST `body`; the response body. |
| `http_post_json(url, json)` | POST with `Content-Type: application/json`. |
| `http_post_form(url, form)` | POST with `application/x-www-form-urlencoded`. |
| `http_download(url, path)` | Save the response to a file. `true` on success. Use this for binaries. |
| `http_last_status()` | Status code of the last request (`200`, `404`, ...), `0` on failure. |
| `http_last_ok()` | `true` when that code is 2xx. |
| `http_last_error()` | Why the last request failed (`""` when it worked). |
| `http_encode(s)`, `http_decode(s)` | Percent-encode / decode a value. `http_decode` also turns `+` into a space. |
| `http_form_encode(pairs)` | Encode the values of `"a=1&b=2"` and keep the keys. |
| `http_with_query(url, query)` | Append a query string with `?` or `&` as needed. |

A 404 is a normal response: you get the body and `http_last_status()` is `404`.

```fenneth
import "std/http";

fn main() -> i32 {
    let q: String = http_form_encode("q=hello world");
    let body: String = http_get(http_with_query("https://example.com/search", q));
    if !http_last_ok() {
        print("request failed: " + http_last_error());
        return 1;
    }
    print(len(body));
    return 0;
}
```

## std/zip

```fenneth
import "std/zip";
```

Read and write `.zip` archives. The reader handles any normal archive (stored and deflated entries, including the binaries inside); the writer produces a standard archive with stored (uncompressed) entries, so it is valid everywhere but bigger than a deflated one.

| Function | Description |
|---|---|
| `zip_list(archive)` | Entry names, one per line (`""` when the archive cannot be read). |
| `zip_entries(archive)` | The same as a `[String]`. |
| `zip_contains(archive, name)` | Is there an entry with this name? |
| `zip_entry_size(archive, name)` | Uncompressed size in bytes, or `-1`. |
| `zip_read(archive, name)` | One entry as a `String` (`""` when missing). |
| `zip_extract(archive, dest_dir)` | Unpack every entry below `dest_dir`, creating the folders it needs. |
| `zip_extract_file(archive, name, dest)` | Unpack one entry to an exact path. |
| `zip_create(archive, entries)` | Create an archive: `"name=path"` per line, `"name="` for an empty entry. |
| `zip_create_files(archive, names, sources)` | The same from two matching arrays. |
| `zip_last_error()` | Why the last call failed (`""` when it worked). |

```fenneth
import "std/zip";
import "std/fs";

fn main() -> i32 {
    let arc: String = fs_join(fs_temp_dir(), "out.zip");
    if !zip_create(arc, "VERSION=\nreadme.txt=README.md") {
        print(zip_last_error());
        return 1;
    }
    zip_extract(arc, fs_join(fs_temp_dir(), "unpacked"));
    print(zip_contains(arc, "readme.txt"));
    return 0;
}
```

## Limitations

- Without generics there are three copies of every array function, and there is no `array_sort` for arrays of structs. Write your own loop (or a merge sort like `fenneth std array` shows) for those.
- There are no hash maps and no Unicode-aware string functions yet.
- `std/http` needs the `curl` (or `wget`) command on `PATH` when the program runs; nothing extra is linked.
- `std/zip` reads stored and deflated entries (any normal archive) but writes stored, uncompressed entries.
