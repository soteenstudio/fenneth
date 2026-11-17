# Command line

```text
fenneth <file>                               Same as `fenneth run <file>`
fenneth run   <file> [options] [-- args...]  Build and run (args go to the program)
fenneth build <file> [-o out] [options]      Compile to a native executable
fenneth emit  <file> [-o out.c]              Compile to C (stdout by default)
fenneth check <file>                         Type-check only
fenneth --version | --help
```

## Commands

| Command | What it does | Needs C compiler |
|---|---|---|
| `run` (default) | Compiles to a temporary executable, runs it, and returns its exit code. | Yes |
| `build` | Produces a native executable. Default name is the file name without extension (`.exe` on Windows). | Yes |
| `emit` | Writes the generated C99 to stdout, or to the file given with `-o`. | No |
| `check` | Runs the lexer, parser and type checker. Prints `ok: <file>` on success. | No |

```sh
fenneth hello.fen                       # same as: fenneth run hello.fen
fenneth run app.fen -- arg1 arg2        # arguments after -- reach args()
fenneth build app.fen -o app            # native executable
fenneth emit app.fen -o app.c           # only generate C
fenneth check app.fen                   # type-check only
```

## Options for `build` and `run`

| Option | Meaning |
|---|---|
| `--cc <compiler>` | C compiler to use. Default: `$FENNETH_CC`, then `gcc`, `clang`, `cc`. |
| `--link <arg>` | Extra argument passed to the C compiler/linker. Repeatable (libraries, `-lm`, ...). |
| `--keep-c` | Keep the generated `.c` file next to the output. |
| `-o <file>` | Output path (`build`, `emit`). |

The C compiler is invoked with `-std=c99 -O2 -fwrapv -w`. If you compile the output of `emit` yourself, add `-fwrapv` to get the same wrap-around behaviour for `i32` overflow.

## File names

The extension may be omitted: `fenneth app` looks for `app.fen`, then `app.fenneth`. Only one input file is accepted on the command line; use [`import`](/language/modules-ffi) for more.

## Exit codes and errors

- A compile error prints every diagnostic as `file:line:column: error: message`, then `aborting: N errors`, and exits with code `1`. Lexer and parser errors stop at the first one; type checker errors are reported together.
- A running program exits with the value returned by `main`, or with the code given to `exit(code)`.
- A runtime check failure (`panic`) prints `panic: ...` to stderr and exits with code `101`.

## Environment

| Variable | Effect |
|---|---|
| `FENNETH_CC` | Default C compiler for `build` and `run`. |
